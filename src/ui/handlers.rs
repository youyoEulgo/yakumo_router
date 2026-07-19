use crate::AppState;
use crate::config::{
    Protocol, ProviderConfig, RouteRule, RouteTable, create_minimal_config, delete_provider,
    delete_route, delete_route_from_tables, remove_provider_route_ids, upsert_provider,
    upsert_route,
};
use crate::proxy::parse_protocol;
use crate::ui::dto::{
    ActiveRouteTableResult, ConfigFileStatus, CreateConfigResult, DeleteProviderResult,
    DeleteRouteResult, DeleteRouteTableResult, ProviderTables, RouteTableList, RouteTables,
    UpsertProviderResult, UpsertRouteResult, UpsertRouteTableResult,
};
use crate::ui::response::{json_response, read_json_body, save_config};
use axum::{body::Body, extract::Path, extract::State, http::StatusCode, response::Response};
use std::sync::Arc;

pub async fn get_config_status_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Response<Body>, StatusCode> {
    let status = ConfigFileStatus {
        exists: state.config_path.exists(),
    };

    json_response(&status, StatusCode::OK)
}

pub async fn create_config_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Response<Body>, StatusCode> {
    if state.config_path.exists() {
        return Err(StatusCode::CONFLICT);
    }

    let data_dir = state
        .config_path
        .parent()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let config = create_minimal_config(data_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    *state.config.write().await = config;

    let result = CreateConfigResult { created: true };
    json_response(&result, StatusCode::OK)
}

pub async fn list_routes_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Response<Body>, StatusCode> {
    let config = state.config.read().await;
    let routes = RouteTables {
        openai: Protocol::OpenAi.config(&config).routes.clone(),
        anthropic: Protocol::Anthropic.config(&config).routes.clone(),
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
        openai: Protocol::OpenAi.config(&config).providers.clone(),
        anthropic: Protocol::Anthropic.config(&config).providers.clone(),
    };

    json_response(&providers, StatusCode::OK)
}

pub async fn upsert_provider_handler(
    State(state): State<Arc<AppState>>,
    Path((protocol, name)): Path<(String, String)>,
    req: axum::http::Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    ensure_config_file_exists(&state)?;

    if name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let provider: ProviderConfig = read_json_body(req).await?;
    validate_provider(&provider)?;

    let protocol = parse_protocol(&protocol).ok_or(StatusCode::BAD_REQUEST)?;
    let mut config = state.config.write().await;
    let protocol_config = protocol.config_mut(&mut config);

    let updated = upsert_provider(protocol_config, name.clone(), provider.clone());
    save_config(&state, &config)?;

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
    ensure_config_file_exists(&state)?;

    let protocol = parse_protocol(&protocol).ok_or(StatusCode::BAD_REQUEST)?;
    let mut config = state.config.write().await;
    let protocol_config = protocol.config_mut(&mut config);

    let removed_ids: Vec<String> = protocol_config
        .routes
        .iter()
        .filter(|route| route.provider == name)
        .map(|route| route.id.clone())
        .collect();
    let removed_routes = delete_provider(protocol_config, &name).ok_or(StatusCode::NOT_FOUND)?;
    remove_provider_route_ids(&mut config.route_tables, protocol, &removed_ids);
    save_config(&state, &config)?;

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
    ensure_config_file_exists(&state)?;

    let route: RouteRule = read_json_body(req).await?;
    validate_route(&route)?;

    let protocol = parse_protocol(&protocol).ok_or(StatusCode::BAD_REQUEST)?;
    let mut config = state.config.write().await;
    let route_exists_in_other_protocol = match protocol {
        Protocol::OpenAi => route_id_exists(&config.anthropic.routes, &route.id),
        Protocol::Anthropic => route_id_exists(&config.openai.routes, &route.id),
    };
    if route_exists_in_other_protocol {
        return Err(StatusCode::BAD_REQUEST);
    }

    let protocol_config = protocol.config_mut(&mut config);
    if !protocol_config.providers.contains_key(&route.provider) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let routes = &mut protocol_config.routes;
    let updated = upsert_route(routes, route.clone());
    save_config(&state, &config)?;

    let result = UpsertRouteResult { updated, route };
    json_response(&result, StatusCode::OK)
}

pub async fn delete_route_handler(
    State(state): State<Arc<AppState>>,
    Path((protocol, id)): Path<(String, String)>,
) -> Result<Response<Body>, StatusCode> {
    ensure_config_file_exists(&state)?;

    let protocol = parse_protocol(&protocol).ok_or(StatusCode::BAD_REQUEST)?;
    let mut config = state.config.write().await;
    let protocol_config = protocol.config_mut(&mut config);

    if !delete_route(&mut protocol_config.routes, &id) {
        return Err(StatusCode::NOT_FOUND);
    }
    delete_route_from_tables(&mut config.route_tables, &id);
    save_config(&state, &config)?;

    let result = DeleteRouteResult { id };
    json_response(&result, StatusCode::OK)
}

pub async fn upsert_route_table_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    req: axum::http::Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    ensure_config_file_exists(&state)?;

    if name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let table: RouteTable = read_json_body(req).await?;

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
    save_config(&state, &config)?;

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
    ensure_config_file_exists(&state)?;

    let mut config = state.config.write().await;
    if config.route_tables.remove(&name).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    if config.active_route_table.as_deref() == Some(&name) {
        config.active_route_table = None;
    }
    save_config(&state, &config)?;

    let result = DeleteRouteTableResult { name };
    json_response(&result, StatusCode::OK)
}

pub async fn activate_route_table_handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    ensure_config_file_exists(&state)?;

    let mut config = state.config.write().await;
    if !config.route_tables.contains_key(&name) {
        return Err(StatusCode::NOT_FOUND);
    }

    config.active_route_table = Some(name.clone());
    save_config(&state, &config)?;

    let result = ActiveRouteTableResult { active: Some(name) };
    json_response(&result, StatusCode::OK)
}

fn validate_provider(provider: &ProviderConfig) -> Result<(), StatusCode> {
    if provider.base_url.trim().is_empty() || provider.api_key.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(())
}

fn ensure_config_file_exists(state: &AppState) -> Result<(), StatusCode> {
    if state.config_path.exists() {
        Ok(())
    } else {
        Err(StatusCode::CONFLICT)
    }
}

fn validate_route(route: &RouteRule) -> Result<(), StatusCode> {
    if route.id.trim().is_empty()
        || route.matcher.trim().is_empty()
        || route.provider.trim().is_empty()
        || (!route.forward_only && route.model.trim().is_empty())
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(())
}

fn route_table_ids_exist(routes: &[RouteRule], ids: &[String]) -> bool {
    ids.iter()
        .all(|id| routes.iter().any(|route| &route.id == id))
}

fn route_id_exists(routes: &[RouteRule], id: &str) -> bool {
    routes.iter().any(|route| route.id == id)
}
