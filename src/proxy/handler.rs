use crate::AppState;
use crate::config::Protocol;
use crate::proxy::request::{
    build_upstream_uri, check_if_stream, format_model_info, replace_model_in_json,
    should_forward_header,
};
use crate::proxy::routing::{detect_protocol, route_request};
use axum::{body::Body, extract::State, http::StatusCode, response::Response};
use bytes::Bytes;
use futures::StreamExt;
use std::sync::Arc;

const DEBUG_MOD: bool = false;

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
