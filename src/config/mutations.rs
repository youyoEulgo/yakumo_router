use crate::config::schema::{Protocol, ProtocolConfig, ProviderConfig, RouteRule, RouteTable};
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
        table.openai.retain(|route_id| route_id != id);
        table.anthropic.retain(|route_id| route_id != id);
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
        route_ids.retain(|route_id| !removed_ids.contains(route_id));
    }
}

pub fn upsert_provider(
    config: &mut ProtocolConfig,
    name: String,
    provider: ProviderConfig,
) -> bool {
    config.providers.insert(name, provider).is_some()
}

#[cfg(test)]
mod tests {
    use super::{delete_provider, upsert_route};
    use crate::config::schema::{MatchType, ProtocolConfig, ProviderConfig, RouteRule};
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
}
