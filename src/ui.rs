use crate::AppState;
use crate::config::{
    Protocol, ProviderConfig, RouteRule, RouteTable, delete_provider, delete_route,
    delete_route_from_tables, remove_provider_route_ids, upsert_provider, upsert_route,
};
use crate::proxy::parse_protocol;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::Response,
};
use rust_embed::RustEmbed;
use serde::Serialize;
use std::fs;
use std::sync::Arc;

#[derive(RustEmbed)]
#[folder = "ui/dist"]
struct UiAssets;

pub async fn ui_index_handler() -> Result<Response<Body>, StatusCode> {
    embedded_asset_response("index.html")
}

pub async fn ui_asset_handler(Path(path): Path<String>) -> Result<Response<Body>, StatusCode> {
    embedded_asset_response(&path)
}

pub async fn list_routes_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Response<Body>, StatusCode> {
    let config = state.config.read().await;
    let routes = RouteTables {
        openai: config.openai.routes.clone(),
        anthropic: config.anthropic.routes.clone(),
    };

    json_response(&routes, StatusCode::OK)
}

pub async fn list_route_tables_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Response<Body>, StatusCode> {
    let config = state.config.read().await;
    let tables = RouteTableList {
        active: config.active_route_table.clone(),
        tables: config.route_tables.clone(),
    };

    json_response(&tables, StatusCode::OK)
}

pub async fn list_providers_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Response<Body>, StatusCode> {
    let config = state.config.read().await;
    let providers = ProviderTables {
        openai: config.openai.providers.clone(),
        anthropic: config.anthropic.providers.clone(),
    };

    json_response(&providers, StatusCode::OK)
}

pub async fn upsert_provider_handler(
    State(state): State<Arc<AppState>>,
    Path((protocol, name)): Path<(String, String)>,
    req: axum::http::Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    if name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let body = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let provider: ProviderConfig =
        serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    if provider.base_url.trim().is_empty() || provider.api_key.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let protocol = parse_protocol(&protocol).ok_or(StatusCode::BAD_REQUEST)?;
    let mut config = state.config.write().await;
    let protocol_config = match protocol {
        Protocol::OpenAi => &mut config.openai,
        Protocol::Anthropic => &mut config.anthropic,
    };

    let updated = upsert_provider(protocol_config, name.clone(), provider.clone());
    write_config(&state, &config)?;

    let result = UpsertProviderResult {
        updated,
        name,
        provider,
    };
    json_response(&result, StatusCode::OK)
}

pub async fn delete_provider_handler(
    State(state): State<Arc<AppState>>,
    Path((protocol, name)): Path<(String, String)>,
) -> Result<Response<Body>, StatusCode> {
    let protocol = parse_protocol(&protocol).ok_or(StatusCode::BAD_REQUEST)?;
    let mut config = state.config.write().await;
    let protocol_config = match protocol {
        Protocol::OpenAi => &mut config.openai,
        Protocol::Anthropic => &mut config.anthropic,
    };

    let removed_ids: Vec<String> = protocol_config
        .routes
        .iter()
        .filter(|route| route.provider == name)
        .map(|route| route.id.clone())
        .collect();
    let removed_routes = delete_provider(protocol_config, &name).ok_or(StatusCode::NOT_FOUND)?;
    remove_provider_route_ids(&mut config.route_tables, protocol, &removed_ids);
    write_config(&state, &config)?;

    let result = DeleteProviderResult {
        name,
        removed_routes,
    };
    json_response(&result, StatusCode::OK)
}

pub async fn upsert_route_handler(
    State(state): State<Arc<AppState>>,
    Path(protocol): Path<String>,
    req: axum::http::Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let body = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let route: RouteRule = serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    if route.id.trim().is_empty()
        || route.matcher.trim().is_empty()
        || route.provider.trim().is_empty()
        || (!route.forward_only && route.model.trim().is_empty())
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let protocol = parse_protocol(&protocol).ok_or(StatusCode::BAD_REQUEST)?;
    let mut config = state.config.write().await;
    let route_exists_in_other_protocol = match protocol {
        Protocol::OpenAi => route_id_exists(&config.anthropic.routes, &route.id),
        Protocol::Anthropic => route_id_exists(&config.openai.routes, &route.id),
    };
    if route_exists_in_other_protocol {
        return Err(StatusCode::BAD_REQUEST);
    }

    let protocol_config = match protocol {
        Protocol::OpenAi => &mut config.openai,
        Protocol::Anthropic => &mut config.anthropic,
    };
    if !protocol_config.providers.contains_key(&route.provider) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let routes = &mut protocol_config.routes;
    let updated = upsert_route(routes, route.clone());
    write_config(&state, &config)?;

    let result = UpsertRouteResult { updated, route };
    json_response(&result, StatusCode::OK)
}

pub async fn delete_route_handler(
    State(state): State<Arc<AppState>>,
    Path((protocol, id)): Path<(String, String)>,
) -> Result<Response<Body>, StatusCode> {
    let protocol = parse_protocol(&protocol).ok_or(StatusCode::BAD_REQUEST)?;
    let mut config = state.config.write().await;
    let protocol_config = match protocol {
        Protocol::OpenAi => &mut config.openai,
        Protocol::Anthropic => &mut config.anthropic,
    };

    if !delete_route(&mut protocol_config.routes, &id) {
        return Err(StatusCode::NOT_FOUND);
    }
    delete_route_from_tables(&mut config.route_tables, &id);
    write_config(&state, &config)?;

    let result = DeleteRouteResult { id };
    json_response(&result, StatusCode::OK)
}

pub async fn upsert_route_table_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    req: axum::http::Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    if name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let body = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let table: RouteTable = serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut config = state.config.write().await;
    if !route_table_ids_exist(&config.openai.routes, &table.openai)
        || !route_table_ids_exist(&config.anthropic.routes, &table.anthropic)
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let updated = config
        .route_tables
        .insert(name.clone(), table.clone())
        .is_some();
    write_config(&state, &config)?;

    let result = UpsertRouteTableResult {
        updated,
        name,
        table,
    };
    json_response(&result, StatusCode::OK)
}

pub async fn delete_route_table_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    let mut config = state.config.write().await;
    if config.route_tables.remove(&name).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    if config.active_route_table.as_deref() == Some(&name) {
        config.active_route_table = None;
    }
    write_config(&state, &config)?;

    let result = DeleteRouteTableResult { name };
    json_response(&result, StatusCode::OK)
}

pub async fn activate_route_table_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    let mut config = state.config.write().await;
    if !config.route_tables.contains_key(&name) {
        return Err(StatusCode::NOT_FOUND);
    }

    config.active_route_table = Some(name.clone());
    write_config(&state, &config)?;

    let result = ActiveRouteTableResult { active: Some(name) };
    json_response(&result, StatusCode::OK)
}

fn embedded_asset_response(path: &str) -> Result<Response<Body>, StatusCode> {
    let asset = UiAssets::get(path).ok_or(StatusCode::NOT_FOUND)?;
    let content_type = content_type_for_path(path);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(asset.data.into_owned()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Serialize)]
struct RouteTables {
    openai: Vec<RouteRule>,
    anthropic: Vec<RouteRule>,
}

#[derive(Serialize)]
struct ProviderTables {
    openai: std::collections::HashMap<String, ProviderConfig>,
    anthropic: std::collections::HashMap<String, ProviderConfig>,
}

#[derive(Serialize)]
struct RouteTableList {
    active: Option<String>,
    tables: std::collections::HashMap<String, RouteTable>,
}

#[derive(Serialize)]
struct UpsertRouteResult {
    updated: bool,
    route: RouteRule,
}

#[derive(Serialize)]
struct UpsertProviderResult {
    updated: bool,
    name: String,
    provider: ProviderConfig,
}

#[derive(Serialize)]
struct DeleteProviderResult {
    name: String,
    removed_routes: usize,
}

#[derive(Serialize)]
struct DeleteRouteResult {
    id: String,
}

#[derive(Serialize)]
struct DeleteRouteTableResult {
    name: String,
}

#[derive(Serialize)]
struct UpsertRouteTableResult {
    updated: bool,
    name: String,
    table: RouteTable,
}

#[derive(Serialize)]
struct ActiveRouteTableResult {
    active: Option<String>,
}

fn write_config(state: &AppState, config: &crate::config::AppConfig) -> Result<(), StatusCode> {
    let config_text =
        toml::to_string_pretty(config).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    fs::write(&state.config_path, config_text).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn route_table_ids_exist(routes: &[RouteRule], ids: &[String]) -> bool {
    ids.iter()
        .all(|id| routes.iter().any(|route| &route.id == id))
}

fn route_id_exists(routes: &[RouteRule], id: &str) -> bool {
    routes.iter().any(|route| route.id == id)
}

fn json_response<T: Serialize>(
    value: &T,
    status: StatusCode,
) -> Result<Response<Body>, StatusCode> {
    let body = serde_json::to_vec(value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .body(Body::from(body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn content_type_for_path(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "text/javascript; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".json") {
        "application/json; charset=utf-8"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".webp") {
        "image/webp"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else {
        "application/octet-stream"
    }
}
