use crate::config::{AppConfig, MatchType, Protocol, ProtocolConfig, ProviderConfig, RouteRule};
use crate::proxy::request::extract_model_name;
use regex::RegexBuilder;

pub struct RouteTarget<'a> {
    pub protocol: Protocol,
    pub provider_name: &'a str,
    pub provider: &'a ProviderConfig,
    pub original_model: Option<String>,
    pub routed_model: Option<String>,
}

pub fn parse_protocol(protocol: &str) -> Option<Protocol> {
    match protocol.to_ascii_lowercase().as_str() {
        "openai" => Some(Protocol::OpenAi),
        "anthropic" => Some(Protocol::Anthropic),
        _ => None,
    }
}

pub fn detect_protocol(path: &str, body: &[u8]) -> Option<Protocol> {
    if path.starts_with("/v1/messages") {
        return Some(Protocol::Anthropic);
    }
    if path.starts_with("/v1/chat/completions") || path.starts_with("/v1/responses") {
        return Some(Protocol::OpenAi);
    }

    let json: serde_json::Value = serde_json::from_slice(body).ok()?;
    if json.get("system").is_some()
        && json
            .get("max_tokens")
            .or_else(|| json.get("max_tokens_to_sample"))
            .is_some()
    {
        Some(Protocol::Anthropic)
    } else if json.get("messages").is_some() || json.get("input").is_some() {
        Some(Protocol::OpenAi)
    } else {
        None
    }
}

pub fn route_request<'a>(
    config: &'a AppConfig,
    protocol: Protocol,
    body: &[u8],
) -> Option<RouteTarget<'a>> {
    let protocol_config = match protocol {
        Protocol::OpenAi => &config.openai,
        Protocol::Anthropic => &config.anthropic,
    };
    let original_model = extract_model_name(body);
    let matched = original_model
        .as_deref()
        .and_then(|model| find_route(config, protocol, protocol_config, model))?;

    let provider_name = matched.provider.as_str();
    let provider = protocol_config.providers.get(provider_name)?;
    let routed_model = if matched.forward_only {
        None
    } else {
        Some(matched.model.clone())
    };

    Some(RouteTarget {
        protocol,
        provider_name,
        provider,
        original_model,
        routed_model,
    })
}

fn find_route<'a>(
    config: &'a AppConfig,
    protocol: Protocol,
    protocol_config: &'a ProtocolConfig,
    model: &str,
) -> Option<&'a RouteRule> {
    let active_ids = config
        .active_route_table
        .as_deref()
        .and_then(|name| config.route_tables.get(name))
        .map(|table| match protocol {
            Protocol::OpenAi => table.openai.as_slice(),
            Protocol::Anthropic => table.anthropic.as_slice(),
        });

    if let Some(active_ids) = active_ids {
        return active_ids.iter().find_map(|id| {
            protocol_config
                .routes
                .iter()
                .find(|route| &route.id == id && route_matches(route, model))
        });
    }

    protocol_config
        .routes
        .iter()
        .find(|route| route_matches(route, model))
}

fn route_matches(route: &RouteRule, model: &str) -> bool {
    let lower = model.to_lowercase();
    let matcher = route.matcher.to_lowercase();
    match route.match_type {
        MatchType::Contains => lower.contains(&matcher),
        MatchType::Exact => lower == matcher,
        MatchType::Regex => RegexBuilder::new(&route.matcher)
            .case_insensitive(true)
            .build()
            .map(|regex| regex.is_match(model))
            .unwrap_or_else(|e| {
                eprintln!(
                    "[{}] Invalid regex in route {}: {}",
                    crate::ts(),
                    route.id,
                    e
                );
                false
            }),
    }
}

#[cfg(test)]
mod tests {
    use super::{find_route, route_request};
    use crate::config::{
        AppConfig, MatchType, Protocol, ProtocolConfig, ProviderConfig, RouteRule, RouteTable,
    };
    use std::collections::HashMap;

    #[test]
    fn exact_match_requires_full_model_name() {
        let config = app_config(vec![RouteRule {
            id: "exact".to_string(),
            matcher: "gpt-test".to_string(),
            match_type: MatchType::Exact,
            provider: "provider".to_string(),
            model: "upstream".to_string(),
            forward_only: false,
        }]);

        assert!(find_route(&config, Protocol::OpenAi, &config.openai, "gpt-test").is_some());
        assert!(find_route(&config, Protocol::OpenAi, &config.openai, "gpt-test-extra").is_none());
    }

    #[test]
    fn regex_match_supports_patterns() {
        let config = app_config(vec![RouteRule {
            id: "regex".to_string(),
            matcher: "^claude-(sonnet|opus)-\\d+$".to_string(),
            match_type: MatchType::Regex,
            provider: "provider".to_string(),
            model: "upstream".to_string(),
            forward_only: false,
        }]);

        assert!(find_route(&config, Protocol::OpenAi, &config.openai, "claude-sonnet-4").is_some());
        assert!(find_route(&config, Protocol::OpenAi, &config.openai, "claude-haiku-4").is_none());
    }

    #[test]
    fn invalid_regex_does_not_match() {
        let config = app_config(vec![RouteRule {
            id: "bad-regex".to_string(),
            matcher: "(".to_string(),
            match_type: MatchType::Regex,
            provider: "provider".to_string(),
            model: "upstream".to_string(),
            forward_only: false,
        }]);

        assert!(find_route(&config, Protocol::OpenAi, &config.openai, "anything").is_none());
    }

    #[test]
    fn forward_only_route_does_not_rewrite_model() {
        let config = AppConfig {
            server: Default::default(),
            tls: Default::default(),
            openai: protocol_config(vec![RouteRule {
                id: "forward".to_string(),
                matcher: "gpt-test".to_string(),
                match_type: MatchType::Exact,
                provider: "provider".to_string(),
                model: "ignored-upstream-model".to_string(),
                forward_only: true,
            }]),
            anthropic: protocol_config(Vec::new()),
            active_route_table: None,
            route_tables: HashMap::new(),
        };
        let body = br#"{"model":"gpt-test"}"#;

        let target = route_request(&config, Protocol::OpenAi, body).expect("route should match");

        assert_eq!(target.provider_name, "provider");
        assert_eq!(target.routed_model, None);
    }

    #[test]
    fn active_route_table_order_controls_match_priority() {
        let mut config = app_config(vec![
            RouteRule {
                id: "broad".to_string(),
                matcher: "gpt".to_string(),
                match_type: MatchType::Contains,
                provider: "provider".to_string(),
                model: "broad-upstream".to_string(),
                forward_only: false,
            },
            RouteRule {
                id: "specific".to_string(),
                matcher: "gpt-4.1".to_string(),
                match_type: MatchType::Exact,
                provider: "provider".to_string(),
                model: "specific-upstream".to_string(),
                forward_only: false,
            },
        ]);
        config.active_route_table = Some("main".to_string());
        config.route_tables.insert(
            "main".to_string(),
            RouteTable {
                openai: vec!["specific".to_string(), "broad".to_string()],
                anthropic: Vec::new(),
            },
        );

        let matched = find_route(&config, Protocol::OpenAi, &config.openai, "gpt-4.1")
            .expect("route should match");

        assert_eq!(matched.id, "specific");
    }

    fn app_config(routes: Vec<RouteRule>) -> AppConfig {
        AppConfig {
            server: Default::default(),
            tls: Default::default(),
            openai: protocol_config(routes),
            anthropic: protocol_config(Vec::new()),
            active_route_table: None,
            route_tables: HashMap::new(),
        }
    }

    fn protocol_config(routes: Vec<RouteRule>) -> ProtocolConfig {
        let mut providers = HashMap::new();
        providers.insert(
            "provider".to_string(),
            ProviderConfig {
                base_url: "https://example.com/v1".to_string(),
                api_key: "key".to_string(),
            },
        );

        ProtocolConfig { providers, routes }
    }
}
