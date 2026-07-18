mod config;
mod proxy;
mod ui;
mod watcher;

use crate::config::{
    AppConfig, BoxError, config_path, data_dir, init_config, load_config, resolve_config_path,
};
use crate::proxy::proxy_handler;
use crate::ui::{
    activate_route_table_handler, delete_provider_handler, delete_route_handler,
    delete_route_table_handler, list_providers_handler, list_route_tables_handler,
    list_routes_handler, ui_asset_handler, ui_index_handler, upsert_provider_handler,
    upsert_route_handler, upsert_route_table_handler,
};
use crate::watcher::spawn_config_watcher;
use axum::{
    Router,
    routing::{any, get, put},
};
use axum_server::tls_rustls::RustlsConfig;
use reqwest::Client;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

pub struct AppState {
    pub client: Client,
    pub config: Arc<RwLock<AppConfig>>,
    pub config_path: PathBuf,
}

pub fn ts() -> String {
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
async fn main() -> Result<(), BoxError> {
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

async fn run_server(data_dir: PathBuf) -> Result<(), BoxError> {
    let config_path = config_path(&data_dir);
    if !config_path.exists() {
        return Err(format!(
            "config not found at {}; run `yakumo init` first",
            config_path.display()
        )
        .into());
    }

    let config = load_config(&config_path)?;
    let config = Arc::new(RwLock::new(config));

    let client = Client::builder()
        .danger_accept_invalid_certs(false)
        .build()?;

    let startup_config = config.read().await;
    let addr = SocketAddr::from((startup_config.server.host, startup_config.server.port));
    let cert_path = resolve_config_path(&data_dir, &startup_config.tls.cert);
    let key_path = resolve_config_path(&data_dir, &startup_config.tls.key);
    drop(startup_config);

    spawn_config_watcher(config_path.clone(), Arc::clone(&config));

    let app = Router::new()
        .route("/_ui", get(ui_index_handler))
        .route("/_ui/", get(ui_index_handler))
        .route("/_ui/api/providers", get(list_providers_handler))
        .route(
            "/_ui/api/providers/{protocol}/{name}",
            put(upsert_provider_handler).delete(delete_provider_handler),
        )
        .route("/_ui/api/route-tables", get(list_route_tables_handler))
        .route(
            "/_ui/api/route-tables/{name}",
            put(upsert_route_table_handler).delete(delete_route_table_handler),
        )
        .route(
            "/_ui/api/active-route-table/{name}",
            put(activate_route_table_handler),
        )
        .route("/_ui/api/routes", get(list_routes_handler))
        .route("/_ui/api/routes/{protocol}", put(upsert_route_handler))
        .route(
            "/_ui/api/routes/{protocol}/{id}",
            axum::routing::delete(delete_route_handler),
        )
        .route("/_ui/{*path}", get(ui_asset_handler))
        .route("/{*path}", any(proxy_handler))
        .with_state(Arc::new(AppState {
            client,
            config,
            config_path: config_path.clone(),
        }));

    let has_cert = cert_path.exists() && key_path.exists();

    println!("[{}] Config loaded from {}", ts(), config_path.display());
    if has_cert {
        let tls_config = RustlsConfig::from_pem_file(&cert_path, &key_path).await?;
        println!("[{}] HTTPS proxy listening on https://{}", ts(), addr);
        println!("[{}] Web UI available at https://{}/_ui/", ts(), addr);
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
        println!("[{}] Web UI available at http://{}/_ui/", ts(), addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}

fn print_help(data_dir: &std::path::Path) {
    println!("Yakumo Router");
    println!();
    println!("Usage:");
    println!("  yakumo init    Create config.toml template");
    println!("  yakumo         Run proxy server");
    println!();
    println!("Config directory:");
    println!("  {}", data_dir.display());
}
