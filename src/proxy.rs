use crate::AppState;
use crate::config::{AppConfig, MatchType, Protocol, ProtocolConfig, ProviderConfig, RouteRule};
use axum::{body::Body, extract::State, http::StatusCode, response::Response};
use bytes::Bytes;
use futures::StreamExt;
use regex::RegexBuilder;
use std::sync::Arc;

const DEBUG_MOD: bool = false;

struct RouteTarget<'a> {
    protocol: Protocol,
    provider_name: &'a str,
    provider: &'a ProviderConfig,
    original_model: Option<String>,
    routed_model: Option<String>,
}

pub async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    req: axum::http::Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let query = req.uri().query().unwrap_or("").to_string();
    let headers = req.headers().clone();

    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let protocol = detect_protocol(&path, &body_bytes).ok_or(StatusCode::BAD_REQUEST)?;
    let config = state.config.read().await;
    let target = route_request(&config, protocol, &body_bytes).ok_or(StatusCode::BAD_REQUEST)?;

    let out_body = if let Some(ref routed_model) = target.routed_model {
        replace_model_in_json(&body_bytes, routed_model).unwrap_or_else(|| body_bytes.to_vec())
    } else {
        body_bytes.to_vec()
    };

    let upstream_uri = build_upstream_uri(&target.provider.base_url, &path, &query);
    let is_stream = check_if_stream(&body_bytes);
    let api_key = target.provider.api_key.clone();
    let target_protocol = target.protocol;
    let model_info = format_model_info(
        target.protocol,
        target.provider_name,
        target.original_model.as_deref(),
        target.routed_model.as_deref(),
    );
    drop(config);

    let mut upstream_req = state
        .client
        .request(method.clone(), &upstream_uri)
        .header("Authorization", format!("Bearer {}", api_key));

    for (name, value) in &headers {
        if should_forward_header(name.as_str()) {
            upstream_req = upstream_req.header(name, value);
        }
    }

    if matches!(target_protocol, Protocol::Anthropic) {
        upstream_req = upstream_req.header(
            "Accept",
            if is_stream {
                "text/event-stream"
            } else {
                "application/json"
            },
        );
    }

    let resp = upstream_req.body(out_body).send().await.map_err(|e| {
        eprintln!("[{}] Upstream error: {}", crate::ts(), e);
        StatusCode::BAD_GATEWAY
    })?;

    let status = resp.status();
    let resp_headers = resp.headers().clone();
    println!(
        "[{}] {:?} {} {} {} [{}]",
        crate::ts(),
        target_protocol,
        method,
        status,
        path,
        model_info
    );

    if is_stream {
        let stream = resp
            .bytes_stream()
            .map(|result| result.map_err(|e| axum::Error::new(e)))
            .map(|result| result.map(Bytes::from));

        let body = Body::from_stream(stream);
        let mut response = Response::new(body);
        *response.status_mut() = status;
        response
            .headers_mut()
            .insert("content-type", "text/event-stream".parse().unwrap());
        response
            .headers_mut()
            .insert("cache-control", "no-cache".parse().unwrap());
        response
            .headers_mut()
            .insert("x-content-type-options", "nosniff".parse().unwrap());
        Ok(response)
    } else {
        let body = resp.bytes().await.unwrap_or_default();

        if DEBUG_MOD {
            println!("========== Response ==========");
            println!("Headers: {:?}", resp_headers);
            println!("Body: {}", String::from_utf8_lossy(&body));
            println!("==============================");
        }

        let mut response = Response::new(Body::from(body));
        *response.status_mut() = status;
        *response.headers_mut() = resp_headers;
        Ok(response)
    }
}

pub fn parse_protocol(protocol: &str) -> Option<Protocol> {
    match protocol.to_ascii_lowercase().as_str() {
        "openai" => Some(Protocol::OpenAi),
        "anthropic" => Some(Protocol::Anthropic),
        _ => None,
    }
}

fn detect_protocol(path: &str, body: &[u8]) -> Option<Protocol> {
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

fn route_request<'a>(
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

fn build_upstream_uri(base_url: &str, path: &str, query: &str) -> String {
    let base_url = base_url.trim_end_matches('/');
    let path = if base_url.ends_with("/v1") && path.starts_with("/v1/") {
        &path[3..]
    } else {
        path
    };

    if query.is_empty() {
        format!("{}{}", base_url, path)
    } else {
        format!("{}{}?{}", base_url, path, query)
    }
}

fn should_forward_header(name: &str) -> bool {
    !matches!(
        name.to_ascii_lowercase().as_str(),
        "host" | "authorization" | "content-length" | "connection"
    )
}

fn check_if_stream(body: &[u8]) -> bool {
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(body) {
        json.get("stream")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    } else {
        false
    }
}

fn extract_model_name(body: &[u8]) -> Option<String> {
    let json: serde_json::Value = serde_json::from_slice(body).ok()?;
    json.get("model")?.as_str().map(|s| s.to_string())
}

fn replace_model_in_json(body: &[u8], new_model: &str) -> Option<Vec<u8>> {
    let mut json: serde_json::Value = serde_json::from_slice(body).ok()?;
    *json.get_mut("model")? = serde_json::Value::String(new_model.to_string());
    serde_json::to_vec(&json).ok()
}

fn format_model_info(
    protocol: Protocol,
    provider: &str,
    original: Option<&str>,
    routed: Option<&str>,
) -> String {
    match (original, routed) {
        (Some(original), Some(routed)) => {
            format!("{:?}/{}: {} -> {}", protocol, provider, original, routed)
        }
        (Some(original), None) => format!("{:?}/{}: {} passthrough", protocol, provider, original),
        _ => format!("{:?}/{}: no model", protocol, provider),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_upstream_uri, find_route, route_request};
    use crate::config::{
        AppConfig, MatchType, Protocol, ProtocolConfig, ProviderConfig, RouteRule, RouteTable,
    };
    use std::collections::HashMap;

    #[test]
    fn joins_base_url_and_path() {
        assert_eq!(
            build_upstream_uri("https://api.deepseek.com/anthropic", "/v1/messages", ""),
            "https://api.deepseek.com/anthropic/v1/messages"
        );
    }

    #[test]
    fn avoids_duplicate_v1_prefix() {
        assert_eq!(
            build_upstream_uri("https://openrouter.ai/api/v1", "/v1/chat/completions", ""),
            "https://openrouter.ai/api/v1/chat/completions"
        );
    }

    #[test]
    fn preserves_query_string() {
        assert_eq!(
            build_upstream_uri("https://example.com/v1", "/v1/models", "limit=10"),
            "https://example.com/v1/models?limit=10"
        );
    }

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
