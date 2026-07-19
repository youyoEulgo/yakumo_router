use crate::AppState;
use crate::config::AppConfig;
use axum::{body::Body, http::StatusCode, response::Response};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;

pub async fn read_json_body<T: DeserializeOwned>(
    req: axum::http::Request<Body>,
) -> Result<T, StatusCode> {
    let body = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)
}

pub fn save_config(state: &AppState, config: &AppConfig) -> Result<(), StatusCode> {
    let config_text =
        toml::to_string_pretty(config).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    fs::write(&state.config_path, config_text).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn json_response<T: Serialize>(
    value: &T,
    status: StatusCode,
) -> Result<Response<Body>, StatusCode> {
    let body = serde_json::to_vec(value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Response::builder()
        .status(status)
        .header(
            axum::http::header::CONTENT_TYPE,
            "application/json; charset=utf-8",
        )
        .body(Body::from(body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
