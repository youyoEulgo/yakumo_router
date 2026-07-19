use crate::config::Protocol;

pub fn build_upstream_uri(base_url: &str, path: &str, query: &str) -> String {
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

pub fn should_forward_header(name: &str) -> bool {
    !matches!(
        name.to_ascii_lowercase().as_str(),
        "host" | "authorization" | "content-length" | "connection"
    )
}

pub fn check_if_stream(body: &[u8]) -> bool {
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(body) {
        json.get("stream")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    } else {
        false
    }
}

pub fn extract_model_name(body: &[u8]) -> Option<String> {
    let json: serde_json::Value = serde_json::from_slice(body).ok()?;
    json.get("model")?.as_str().map(|s| s.to_string())
}

pub fn replace_model_in_json(body: &[u8], new_model: &str) -> Option<Vec<u8>> {
    let mut json: serde_json::Value = serde_json::from_slice(body).ok()?;
    *json.get_mut("model")? = serde_json::Value::String(new_model.to_string());
    serde_json::to_vec(&json).ok()
}

pub fn format_model_info(
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
    use super::build_upstream_uri;

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
}
