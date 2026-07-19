use crate::config::{ProviderConfig, RouteRule, RouteTable};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct RouteTables {
    pub openai: Vec<RouteRule>,
    pub anthropic: Vec<RouteRule>,
}

#[derive(Serialize)]
pub struct ProviderTables {
    pub openai: HashMap<String, ProviderConfig>,
    pub anthropic: HashMap<String, ProviderConfig>,
}

#[derive(Serialize)]
pub struct RouteTableList {
    pub active: Option<String>,
    pub tables: HashMap<String, RouteTable>,
}

#[derive(Serialize)]
pub struct UpsertRouteResult {
    pub updated: bool,
    pub route: RouteRule,
}

#[derive(Serialize)]
pub struct UpsertProviderResult {
    pub updated: bool,
    pub name: String,
    pub provider: ProviderConfig,
}

#[derive(Serialize)]
pub struct DeleteProviderResult {
    pub name: String,
    pub removed_routes: usize,
}

#[derive(Serialize)]
pub struct DeleteRouteResult {
    pub id: String,
}

#[derive(Serialize)]
pub struct DeleteRouteTableResult {
    pub name: String,
}

#[derive(Serialize)]
pub struct UpsertRouteTableResult {
    pub updated: bool,
    pub name: String,
    pub table: RouteTable,
}

#[derive(Serialize)]
pub struct ActiveRouteTableResult {
    pub active: Option<String>,
}

#[derive(Serialize)]
pub struct ConfigFileStatus {
    pub exists: bool,
}

#[derive(Serialize)]
pub struct CreateConfigResult {
    pub created: bool,
}
