use axum::{
    Router, body::Body, extract::State, http::StatusCode, response::Response, routing::any,
};
use axum_server::tls_rustls::RustlsConfig;
use bytes::Bytes;
use futures::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const APP_DIR_NAME: &str = "yakumo_switch";
const CONFIG_FILE_NAME: &str = "config.toml";
const DEFAULT_CONFIG: &str = r#"[server]
host = "127.0.0.1"
port = 8989

[tls]
cert = "cert.pem"
key = "key.pem"

[openai]
default_provider = "openrouter"

[openai.providers.openrouter]
base_url = "https://openrouter.ai/api/v1"
api_key = "sk-your-openrouter-key"

[[openai.routes]]
match = "gpt"
provider = "openrouter"
model = "openai/gpt-4.1"

[anthropic]
default_provider = "deepseek"

[anthropic.providers.deepseek]
base_url = "https://api.deepseek.com/anthropic"
api_key = "sk-your-deepseek-key"

[[anthropic.routes]]
match = "sonnet"
provider = "deepseek"
model = "deepseek-v4-pro"

[[anthropic.routes]]
match = "haiku"
provider = "deepseek"
model = "deepseek-v4-flash"
"#;

const DEBUG_MOD: bool = false;

#[derive(Debug, Deserialize)]
struct AppConfig {
    #[serde(default)]
    server: ServerConfig,
    #[serde(default)]
    tls: TlsConfig,
    openai: ProtocolConfig,
    anthropic: ProtocolConfig,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    #[serde(default = "default_host")]
    host: IpAddr,
    #[serde(default = "default_port")]
    port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct TlsConfig {
    #[serde(default = "default_cert_file")]
    cert: PathBuf,
    #[serde(default = "default_key_file")]
    key: PathBuf,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            cert: default_cert_file(),
            key: default_key_file(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ProtocolConfig {
    default_provider: String,
    providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    routes: Vec<RouteRule>,
}

#[derive(Debug, Deserialize)]
struct ProviderConfig {
    base_url: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct RouteRule {
    #[serde(rename = "match")]
    matcher: String,
    provider: String,
    model: String,
}

#[derive(Clone, Copy, Debug)]
enum Protocol {
    OpenAi,
    Anthropic,
}

struct RouteTarget<'a> {
    protocol: Protocol,
    provider_name: &'a str,
    provider: &'a ProviderConfig,
    original_model: Option<String>,
    routed_model: Option<String>,
}

fn default_host() -> IpAddr {
    IpAddr::from([127, 0, 0, 1])
}

fn default_port() -> u16 {
    8443
}

fn default_cert_file() -> PathBuf {
    PathBuf::from("cert.pem")
}

fn default_key_file() -> PathBuf {
    PathBuf::from("key.pem")
}

fn ts() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let hours = (secs / 3600 + 9) % 24; // JST
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let data_dir = data_dir()?;

    match args.as_slice() {
        [] => run_server(data_dir).await,
        [cmd] if cmd == "init" => init_config(data_dir),
        [cmd] if cmd == "--help" || cmd == "-h" || cmd == "help" => {
            print_help(&data_dir);
            Ok(())
        }
        _ => {
            eprintln!("Unknown command.");
            print_help(&data_dir);
            Err("invalid command".into())
        }
    }
}

async fn run_server(data_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = data_dir.join(CONFIG_FILE_NAME);
    if !config_path.exists() {
        return Err(format!(
            "config not found at {}; run `yakumo_switch init` first",
            config_path.display()
        )
        .into());
    }

    let config_text = fs::read_to_string(&config_path)?;
    let config: AppConfig = toml::from_str(&config_text)?;
    let config = Arc::new(config);

    let client = Client::builder()
        .danger_accept_invalid_certs(false)
        .build()?;

    let addr = SocketAddr::from((config.server.host, config.server.port));
    let app = Router::new()
        .route("/{*path}", any(proxy_handler))
        .with_state((client, Arc::clone(&config)));

    let cert_path = resolve_config_path(&data_dir, &config.tls.cert);
    let key_path = resolve_config_path(&data_dir, &config.tls.key);
    let has_cert = cert_path.exists() && key_path.exists();

    println!("[{}] Config loaded from {}", ts(), config_path.display());
    if has_cert {
        let tls_config = RustlsConfig::from_pem_file(&cert_path, &key_path).await?;
        println!("[{}] HTTPS proxy listening on https://{}", ts(), addr);
        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service())
            .await?;
    } else {
        println!(
            "[{}] HTTP proxy listening on http://{} (no cert in {}, skipping TLS)",
            ts(),
            addr,
            data_dir.display()
        );
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}

fn init_config(data_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(&data_dir)?;

    let config_path = data_dir.join(CONFIG_FILE_NAME);
    if config_path.exists() {
        println!(
            "[{}] Config already exists at {}",
            ts(),
            config_path.display()
        );
        return Ok(());
    }

    fs::write(&config_path, DEFAULT_CONFIG)?;
    println!(
        "[{}] Created default config at {}",
        ts(),
        config_path.display()
    );
    println!("[{}] Edit api_key/base_url/routes before running.", ts());
    Ok(())
}

fn print_help(data_dir: &std::path::Path) {
    println!("Yakumo Switch");
    println!();
    println!("Usage:");
    println!("  yakumo_switch init    Create config.toml template");
    println!("  yakumo_switch         Run proxy server");
    println!();
    println!("Config directory:");
    println!("  {}", data_dir.display());
}

async fn proxy_handler(
    State((client, config)): State<(Client, Arc<AppConfig>)>,
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
    let target = route_request(&config, protocol, &body_bytes).ok_or(StatusCode::BAD_REQUEST)?;

    let out_body = if let Some(ref routed_model) = target.routed_model {
        replace_model_in_json(&body_bytes, routed_model).unwrap_or_else(|| body_bytes.to_vec())
    } else {
        body_bytes.to_vec()
    };

    let upstream_uri = build_upstream_uri(&target.provider.base_url, &path, &query);
    let is_stream = check_if_stream(&body_bytes);

    let mut upstream_req = client.request(method.clone(), &upstream_uri).header(
        "Authorization",
        format!("Bearer {}", target.provider.api_key),
    );

    for (name, value) in &headers {
        if should_forward_header(name.as_str()) {
            upstream_req = upstream_req.header(name, value);
        }
    }

    if matches!(target.protocol, Protocol::Anthropic) {
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
        eprintln!("[{}] Upstream error: {}", ts(), e);
        StatusCode::BAD_GATEWAY
    })?;

    let status = resp.status();
    let resp_headers = resp.headers().clone();
    let model_info = format_model_info(
        target.protocol,
        target.provider_name,
        target.original_model.as_deref(),
        target.routed_model.as_deref(),
    );

    println!(
        "[{}] {:?} {} {} {} [{}]",
        ts(),
        target.protocol,
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

fn data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return Ok(PathBuf::from(appdata).join(APP_DIR_NAME));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join(APP_DIR_NAME));
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(xdg_data_home) = std::env::var_os("XDG_DATA_HOME") {
            return Ok(PathBuf::from(xdg_data_home).join(APP_DIR_NAME));
        }
        if let Some(home) = std::env::var_os("HOME") {
            return Ok(PathBuf::from(home)
                .join(".local")
                .join("share")
                .join(APP_DIR_NAME));
        }
    }

    Err("could not determine user data directory".into())
}

fn resolve_config_path(data_dir: &std::path::Path, path: &std::path::Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        data_dir.join(path)
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
        .and_then(|model| find_route(protocol_config, model));

    let provider_name = matched
        .map(|route| route.provider.as_str())
        .unwrap_or(protocol_config.default_provider.as_str());
    let provider = protocol_config.providers.get(provider_name)?;
    let routed_model = matched.map(|route| route.model.clone());

    Some(RouteTarget {
        protocol,
        provider_name,
        provider,
        original_model,
        routed_model,
    })
}

fn find_route<'a>(config: &'a ProtocolConfig, model: &str) -> Option<&'a RouteRule> {
    let lower = model.to_lowercase();
    config
        .routes
        .iter()
        .find(|route| lower.contains(&route.matcher.to_lowercase()))
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
