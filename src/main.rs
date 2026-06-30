use axum::{
    Router, body::Body, extract::State, http::StatusCode, response::Response, routing::any,
};
use axum_server::tls_rustls::RustlsConfig;
use bytes::Bytes;
use futures::StreamExt;
use reqwest::Client;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const DEBUG_MOD: bool = false;

struct AppConfig {
    upstream_base_url: String,
    api_key: String,
    model_opus: String,
    model_sonnet: String,
    model_haiku: String,
    port: u16,
}

impl AppConfig {
    fn from_env() -> Self {
        Self {
            upstream_base_url: std::env::var("UPSTREAM_BASE_URL")
                .expect("UPSTREAM_BASE_URL not set"),
            api_key: std::env::var("API_KEY").expect("API_KEY not set"),
            model_opus: std::env::var("MODEL_OPUS").expect("MODEL_OPUS not set"),
            model_sonnet: std::env::var("MODEL_SONNET").expect("MODEL_SONNET not set"),
            model_haiku: std::env::var("MODEL_HAIKU").expect("MODEL_HAIKU not set"),
            port: std::env::var("PROXY_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8443),
        }
    }
}

fn ts() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let hours = (secs / 3600 + 8) % 24; // UTC+8
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config = Arc::new(AppConfig::from_env());

    let client = Client::builder()
        .danger_accept_invalid_certs(false)
        .build()?;

    let port = config.port;
    let app = Router::new()
        .route("/{*path}", any(proxy_handler))
        .with_state((client, config));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let has_cert = std::path::Path::new("cert.pem").exists()
        && std::path::Path::new("key.pem").exists();

    if has_cert {
        let tls_config = RustlsConfig::from_pem_file("cert.pem", "key.pem").await?;
        println!("[{}] HTTPS proxy listening on https://{}", ts(), addr);
        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service())
            .await?;
    } else {
        println!("[{}] HTTP proxy listening on http://{} (no cert, skipping TLS)", ts(), addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}

async fn proxy_handler(
    State((client, config)): State<(Client, Arc<AppConfig>)>,
    req: axum::http::Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let query = req.uri().query().unwrap_or("").to_string();

    // 使用 .env 中配置的 API Key，忽略客户端传来的认证头
    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let anthropic_version = req
        .headers()
        .get("anthropic-version")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 读取完整 body
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // 判断是否需要改写模型名（仅 POST /v1/messages 且非 count_tokens）
    let is_messages_api = method == axum::http::Method::POST
        && path.starts_with("/v1/messages")
        && !path.contains("count_tokens");

    let is_stream = if is_messages_api {
        check_if_stream(&body_bytes)
    } else {
        false
    };

    let (out_body, model_info) = if is_messages_api {
        let orig_model = extract_model_name(&body_bytes).unwrap_or("?".into());
        let modified = match modify_model_in_json(&config, &body_bytes) {
            Some(b) => b,
            None => body_bytes.to_vec(),
        };
        let routed_model = extract_model_name(&modified).unwrap_or("?".into());

        if DEBUG_MOD {
            println!(
                "[{}] [{}] {} → {}",
                ts(),
                if is_stream { "Stream" } else { "Batch" },
                orig_model,
                routed_model
            );
        }

        (modified, format!("{} → {}", orig_model, routed_model))
    } else {
        if DEBUG_MOD {
            println!("[{}] [Pass] {} {}", ts(), method, path);
        }
        (body_bytes.to_vec(), String::new())
    };

    // 构建上游 URI
    let upstream_uri = if query.is_empty() {
        format!("{}{}", config.upstream_base_url, path)
    } else {
        format!("{}{}?{}", config.upstream_base_url, path, query)
    };

    // 构建上游请求
    let mut upstream_req = client
        .request(method.clone(), &upstream_uri)
        .header("Authorization", format!("Bearer {}", config.api_key));

    if let Some(ref ct) = content_type {
        upstream_req = upstream_req.header("Content-Type", ct);
    }
    if let Some(ref av) = anthropic_version {
        upstream_req = upstream_req.header("anthropic-version", av);
    }
    if is_messages_api {
        upstream_req = upstream_req.header(
            "Accept",
            if is_stream {
                "text/event-stream"
            } else {
                "application/json"
            },
        );
    }

    upstream_req = upstream_req.body(out_body);

    let resp = upstream_req.send().await.map_err(|e| {
        eprintln!("[{}] Upstream error: {}", ts(), e);
        StatusCode::BAD_GATEWAY
    })?;

    let status = resp.status();
    let headers = resp.headers().clone();

    // 打印日志
    if !model_info.is_empty() {
        println!(
            "[{}] {} {} {} [{}]",
            ts(),
            if is_stream { "[Stream]" } else { "[Batch]" },
            status,
            path,
            model_info
        );
    } else {
        println!("[{}] {} {} {}", ts(), method, status, path);
    }

    if is_stream {
        if DEBUG_MOD {
            // 调试模式：收集全部流数据并打印
            let all_chunks: Vec<Bytes> = resp
                .bytes_stream()
                .filter_map(|r| async { r.ok().map(Bytes::from) })
                .collect()
                .await;

            println!("========== Stream Content ==========");
            for chunk in &all_chunks {
                print!("{}", String::from_utf8_lossy(chunk));
            }
            println!("\n====================================");

            let stream = tokio_stream::iter(all_chunks.into_iter().map(Ok::<_, axum::Error>));
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
        }
    } else {
        let body = resp.bytes().await.unwrap_or_default();

        if DEBUG_MOD {
            println!("========== Response ==========");
            println!("Headers: {:?}", headers);
            println!("Body: {}", String::from_utf8_lossy(&body));
            println!("==============================");
        }

        let mut response = Response::new(Body::from(body));
        *response.status_mut() = status;
        *response.headers_mut() = headers;
        Ok(response)
    }
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

fn modify_model_in_json(config: &AppConfig, body: &[u8]) -> Option<Vec<u8>> {
    let mut json: serde_json::Value = serde_json::from_slice(body).ok()?;
    if let Some(model) = json.get_mut("model") {
        if let Some(new_model) = route_model(config, model.as_str()?) {
            *model = serde_json::Value::String(new_model);
        }
    }
    serde_json::to_vec(&json).ok()
}

/// Maps incoming model names to upstream model names by model family.
/// Returns `None` if the model doesn't need routing.
fn route_model(config: &AppConfig, model: &str) -> Option<String> {
    let lower = model.to_lowercase();
    if lower.contains("sonnet") {
        Some(config.model_sonnet.clone())
    } else if lower.contains("opus") {
        Some(config.model_opus.clone())
    } else if lower.contains("haiku") {
        Some(config.model_haiku.clone())
    } else {
        None
    }
}
