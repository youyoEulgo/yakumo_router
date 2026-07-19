use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "ui/dist"]
struct UiAssets;

pub async fn ui_index_handler() -> Result<Response<Body>, StatusCode> {
    embedded_asset_response("index.html")
}

pub async fn ui_asset_handler(Path(path): Path<String>) -> Result<Response<Body>, StatusCode> {
    embedded_asset_response(&path)
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
