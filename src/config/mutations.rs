use crate::config::schema::{
    AppConfig, Protocol, ProtocolConfig, ProviderConfig, RouteRule, RouteTable, RouteTableEntry,
};
use std::collections::HashMap;

pub fn upsert_route(routes: &mut Vec<RouteRule>, route: RouteRule) -> bool {
    if let Some(existing) = routes.iter_mut().find(|existing| existing.id == route.id) {
        *existing = route;
        true
    } else {
        routes.push(route);
        false
    }
}

pub fn delete_route(routes: &mut Vec<RouteRule>, id: &str) -> bool {
    let old_len = routes.len();
    routes.retain(|route| route.id != id);
    routes.len() != old_len
}

pub fn delete_route_from_tables(route_tables: &mut HashMap<String, RouteTable>, id: &str) {
    for table in route_tables.values_mut() {
        table.openai.retain(|entry| entry.id != id);
        table.anthropic.retain(|entry| entry.id != id);
    }
}

pub fn delete_provider(config: &mut ProtocolConfig, name: &str) -> Option<usize> {
    config.providers.remove(name)?;
    let old_len = config.routes.len();
    config.routes.retain(|route| route.provider != name);
    Some(old_len - config.routes.len())
}

pub fn remove_provider_route_ids(
    route_tables: &mut HashMap<String, RouteTable>,
    protocol: Protocol,
    removed_ids: &[String],
) {
    for table in route_tables.values_mut() {
        let route_ids = protocol.table_routes_mut(table);
        route_ids.retain(|entry| !removed_ids.contains(&entry.id));
    }
}

pub fn add_route_table_entries(entries: &mut Vec<RouteTableEntry>, ids: &[String]) -> usize {
    let mut added = 0;

    for id in ids {
        if !entries.iter().any(|entry| &entry.id == id) {
            entries.push(RouteTableEntry {
                id: id.clone(),
                enabled: false,
            });
            added += 1;
        }
    }

    added
}

pub fn remove_route_table_entries(entries: &mut Vec<RouteTableEntry>, ids: &[String]) -> usize {
    let before = entries.len();
    entries.retain(|entry| !ids.contains(&entry.id));
    before - entries.len()
}

/// Rewrite references after a rule id changes, keeping each entry's position and
/// enabled flag untouched.
pub fn rename_route_in_tables(
    route_tables: &mut HashMap<String, RouteTable>,
    protocol: Protocol,
    old_id: &str,
    new_id: &str,
) {
    for table in route_tables.values_mut() {
        for entry in protocol.table_routes_mut(table).iter_mut() {
            if entry.id == old_id {
                entry.id = new_id.to_string();
            }
        }
    }
}

pub fn upsert_provider(
    config: &mut ProtocolConfig,
    name: String,
    provider: ProviderConfig,
) -> bool {
    config.providers.insert(name, provider).is_some()
}

#[derive(Debug, PartialEq, Eq)]
pub enum RenameRouteTableError {
    InvalidName,
    NotFound,
    Conflict,
}

/// Rename a route table in place, moving the active pointer with it.
pub fn rename_route_table(
    config: &mut AppConfig,
    old_name: &str,
    new_name: &str,
) -> Result<(), RenameRouteTableError> {
    let new_name = new_name.trim();
    if new_name.is_empty() {
        return Err(RenameRouteTableError::InvalidName);
    }
    if !config.route_tables.contains_key(old_name) {
        return Err(RenameRouteTableError::NotFound);
    }
    if new_name == old_name {
        return Ok(());
    }
    if config.route_tables.contains_key(new_name) {
        return Err(RenameRouteTableError::Conflict);
    }

    let table = config
        .route_tables
        .remove(old_name)
        .ok_or(RenameRouteTableError::NotFound)?;
    config.route_tables.insert(new_name.to_string(), table);
    if config.active_route_table.as_deref() == Some(old_name) {
        config.active_route_table = Some(new_name.to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        RenameRouteTableError, add_route_table_entries, delete_provider,
        remove_route_table_entries, rename_route_in_tables, rename_route_table, upsert_route,
    };
    use crate::config::schema::{
        AppConfig, MatchType, Protocol, ProtocolConfig, ProviderConfig, RouteRule, RouteTable,
        RouteTableEntry,
    };
    use std::collections::HashMap;

    #[test]
    fn upsert_route_updates_existing_id() {
        let mut routes = vec![RouteRule {
            id: "route-1".to_string(),
            matcher: "old".to_string(),
            match_type: MatchType::Contains,
            provider: "provider-a".to_string(),
            model: "model-a".to_string(),
            forward_only: false,
        }];

        let updated = upsert_route(
            &mut routes,
            RouteRule {
                id: "route-1".to_string(),
                matcher: "new".to_string(),
                match_type: MatchType::Contains,
                provider: "provider-b".to_string(),
                model: "model-b".to_string(),
                forward_only: false,
            },
        );

        assert!(updated);
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].matcher, "new");
        assert_eq!(routes[0].provider, "provider-b");
        assert_eq!(routes[0].model, "model-b");
    }

    #[test]
    fn upsert_route_appends_new_id() {
        let mut routes = Vec::new();

        let updated = upsert_route(
            &mut routes,
            RouteRule {
                id: "route-1".to_string(),
                matcher: "gpt".to_string(),
                match_type: MatchType::Contains,
                provider: "openrouter".to_string(),
                model: "openai/gpt-4.1".to_string(),
                forward_only: false,
            },
        );

        assert!(!updated);
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].id, "route-1");
    }

    #[test]
    fn delete_provider_removes_referencing_routes() {
        let mut providers = HashMap::new();
        providers.insert(
            "openrouter".to_string(),
            ProviderConfig {
                base_url: "https://openrouter.ai/api/v1".to_string(),
                api_key: "key".to_string(),
            },
        );
        providers.insert(
            "deepseek".to_string(),
            ProviderConfig {
                base_url: "https://api.deepseek.com/anthropic".to_string(),
                api_key: "key".to_string(),
            },
        );
        let mut config = ProtocolConfig {
            providers,
            routes: vec![
                RouteRule {
                    id: "route-1".to_string(),
                    matcher: "gpt".to_string(),
                    match_type: MatchType::Contains,
                    provider: "openrouter".to_string(),
                    model: "openai/gpt-4.1".to_string(),
                    forward_only: false,
                },
                RouteRule {
                    id: "route-2".to_string(),
                    matcher: "sonnet".to_string(),
                    match_type: MatchType::Contains,
                    provider: "deepseek".to_string(),
                    model: "deepseek-v4-pro".to_string(),
                    forward_only: false,
                },
            ],
        };

        let removed_routes = delete_provider(&mut config, "openrouter");

        assert_eq!(removed_routes, Some(1));
        assert!(!config.providers.contains_key("openrouter"));
        assert_eq!(config.routes.len(), 1);
        assert_eq!(config.routes[0].provider, "deepseek");
    }

    #[test]
    fn add_route_table_entries_appends_disabled_and_skips_existing() {
        let mut entries = vec![RouteTableEntry {
            id: "route-a".to_string(),
            enabled: true,
        }];

        let added = add_route_table_entries(
            &mut entries,
            &[
                "route-a".to_string(),
                "route-b".to_string(),
                "route-c".to_string(),
            ],
        );

        assert_eq!(added, 2);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].id, "route-a");
        assert!(entries[0].enabled);
        assert_eq!(entries[1].id, "route-b");
        assert!(!entries[1].enabled);
        assert_eq!(entries[2].id, "route-c");
        assert!(!entries[2].enabled);
    }

    #[test]
    fn remove_route_table_entries_is_idempotent() {
        let mut entries = vec![
            RouteTableEntry {
                id: "route-a".to_string(),
                enabled: true,
            },
            RouteTableEntry {
                id: "route-b".to_string(),
                enabled: false,
            },
        ];

        let removed = remove_route_table_entries(
            &mut entries,
            &["route-b".to_string(), "missing".to_string()],
        );

        assert_eq!(removed, 1);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "route-a");
    }

    #[test]
    fn rename_route_in_tables_rewrites_ids_in_place() {
        let mut tables = HashMap::new();
        tables.insert(
            "main".to_string(),
            RouteTable {
                openai: vec![
                    RouteTableEntry {
                        id: "old".to_string(),
                        enabled: true,
                    },
                    RouteTableEntry {
                        id: "other".to_string(),
                        enabled: false,
                    },
                ],
                ..Default::default()
            },
        );

        rename_route_in_tables(&mut tables, Protocol::OpenAi, "old", "new");

        let entries = &tables.get("main").expect("table exists").openai;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "new");
        assert!(entries[0].enabled);
        assert_eq!(entries[1].id, "other");
        assert!(!entries[1].enabled);
    }

    #[test]
    fn rename_route_table_moves_table_and_active_pointer() {
        let mut config = AppConfig::default();
        config.route_tables.insert(
            "old".to_string(),
            RouteTable {
                openai: vec![RouteTableEntry {
                    id: "route-a".to_string(),
                    enabled: true,
                }],
                ..Default::default()
            },
        );
        config.active_route_table = Some("old".to_string());

        rename_route_table(&mut config, "old", "new").expect("rename succeeds");

        assert!(!config.route_tables.contains_key("old"));
        let table = config
            .route_tables
            .get("new")
            .expect("renamed table exists");
        assert_eq!(table.openai.len(), 1);
        assert_eq!(table.openai[0].id, "route-a");
        assert!(table.openai[0].enabled);
        assert_eq!(config.active_route_table.as_deref(), Some("new"));
    }

    #[test]
    fn rename_route_table_rejects_conflicts_and_missing() {
        let mut config = AppConfig::default();
        config
            .route_tables
            .insert("a".to_string(), RouteTable::default());
        config
            .route_tables
            .insert("b".to_string(), RouteTable::default());

        assert_eq!(
            rename_route_table(&mut config, "a", "b"),
            Err(RenameRouteTableError::Conflict)
        );
        assert_eq!(
            rename_route_table(&mut config, "missing", "c"),
            Err(RenameRouteTableError::NotFound)
        );
        assert_eq!(
            rename_route_table(&mut config, "a", "   "),
            Err(RenameRouteTableError::InvalidName)
        );
        assert!(config.route_tables.contains_key("a"));
        assert!(config.route_tables.contains_key("b"));
    }
}
