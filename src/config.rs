use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

const APP_DIR_NAME: &str = "yakumo_router";
const CONFIG_FILE_NAME: &str = "config.toml";
const DEFAULT_CONFIG: &str = r#"active_route_table = "default"

[server]
host = "127.0.0.1"
port = 8989

[tls]
cert = "cert.pem"
key = "key.pem"

[openai.providers.openrouter]
base_url = "https://openrouter.ai/api/v1"
api_key = "sk-your-openrouter-key"

[[openai.routes]]
id = "openai-gpt"
match = ".*"
match_type = "regex"
provider = "openrouter"
model = "openai/gpt-4.1"
forward_only = true

[anthropic.providers.deepseek]
base_url = "https://api.deepseek.com/anthropic"
api_key = "sk-your-deepseek-key"

[[anthropic.routes]]
id = "anthropic-sonnet"
match = "sonnet"
match_type = "contains"
provider = "deepseek"
model = "deepseek-v4-pro"
forward_only = false

[route_tables.default]
openai = ["openai-gpt"]
anthropic = ["anthropic-sonnet"]
"#;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub tls: TlsConfig,
    pub openai: ProtocolConfig,
    pub anthropic: ProtocolConfig,
    #[serde(default)]
    pub active_route_table: Option<String>,
    #[serde(default)]
    pub route_tables: HashMap<String, RouteTable>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TlsConfig {
    #[serde(default = "default_cert_file")]
    pub cert: PathBuf,
    #[serde(default = "default_key_file")]
    pub key: PathBuf,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            cert: default_cert_file(),
            key: default_key_file(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProtocolConfig {
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    pub routes: Vec<RouteRule>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub base_url: String,
    pub api_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteRule {
    pub id: String,
    #[serde(rename = "match")]
    pub matcher: String,
    #[serde(default)]
    pub match_type: MatchType,
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub forward_only: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RouteTable {
    #[serde(default)]
    pub openai: Vec<String>,
    #[serde(default)]
    pub anthropic: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub enum Protocol {
    OpenAi,
    Anthropic,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    #[default]
    Contains,
    Exact,
    Regex,
}

fn default_host() -> IpAddr {
    IpAddr::from([127, 0, 0, 1])
}

fn default_port() -> u16 {
    8989
}

fn default_cert_file() -> PathBuf {
    PathBuf::from("cert.pem")
}

fn default_key_file() -> PathBuf {
    PathBuf::from("key.pem")
}

pub fn config_path(data_dir: &Path) -> PathBuf {
    data_dir.join(CONFIG_FILE_NAME)
}

pub fn load_config(config_path: &Path) -> Result<AppConfig, BoxError> {
    let config_text = fs::read_to_string(config_path)?;
    Ok(toml::from_str(&config_text)?)
}

pub fn init_config(data_dir: PathBuf) -> Result<(), BoxError> {
    fs::create_dir_all(&data_dir)?;

    let config_path = config_path(&data_dir);
    if config_path.exists() {
        println!(
            "[{}] Config already exists at {}",
            crate::ts(),
            config_path.display()
        );
        return Ok(());
    }

    fs::write(&config_path, DEFAULT_CONFIG)?;
    println!(
        "[{}] Created default config at {}",
        crate::ts(),
        config_path.display()
    );
    println!(
        "[{}] Edit api_key/base_url/routes before running.",
        crate::ts()
    );
    Ok(())
}

pub fn data_dir() -> Result<PathBuf, BoxError> {
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

pub fn resolve_config_path(data_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        data_dir.join(path)
    }
}

pub fn upsert_route(routes: &mut Vec<RouteRule>, route: RouteRule) -> bool {
    if let Some(existing) = routes.iter_mut().find(|existing| existing.id == route.id) {
        *existing = route;
        true
    } else {
        routes.push(route);
        false
    }
}

pub fn delete_route(routes: &mut Vec<RouteRule>, id: &str) -> bool {
    let old_len = routes.len();
    routes.retain(|route| route.id != id);
    routes.len() != old_len
}

pub fn delete_route_from_tables(route_tables: &mut HashMap<String, RouteTable>, id: &str) {
    for table in route_tables.values_mut() {
        table.openai.retain(|route_id| route_id != id);
        table.anthropic.retain(|route_id| route_id != id);
    }
}

pub fn delete_provider(config: &mut ProtocolConfig, name: &str) -> Option<usize> {
    config.providers.remove(name)?;
    let old_len = config.routes.len();
    config.routes.retain(|route| route.provider != name);
    Some(old_len - config.routes.len())
}

pub fn remove_provider_route_ids(
    route_tables: &mut HashMap<String, RouteTable>,
    protocol: Protocol,
    removed_ids: &[String],
) {
    for table in route_tables.values_mut() {
        let route_ids = match protocol {
            Protocol::OpenAi => &mut table.openai,
            Protocol::Anthropic => &mut table.anthropic,
        };
        route_ids.retain(|route_id| !removed_ids.contains(route_id));
    }
}

pub fn upsert_provider(
    config: &mut ProtocolConfig,
    name: String,
    provider: ProviderConfig,
) -> bool {
    config.providers.insert(name, provider).is_some()
}

#[cfg(test)]
mod tests {
    use super::{
        AppConfig, DEFAULT_CONFIG, MatchType, ProtocolConfig, ProviderConfig, RouteRule,
        delete_provider, upsert_route,
    };
    use std::collections::HashMap;

    #[test]
    fn default_config_has_active_route_table_at_root() {
        let config: AppConfig = toml::from_str(DEFAULT_CONFIG).expect("default config parses");

        assert_eq!(config.active_route_table.as_deref(), Some("default"));
        assert!(config.route_tables.contains_key("default"));
    }

    #[test]
    fn upsert_route_updates_existing_id() {
        let mut routes = vec![RouteRule {
            id: "route-1".to_string(),
            matcher: "old".to_string(),
            match_type: MatchType::Contains,
            provider: "provider-a".to_string(),
            model: "model-a".to_string(),
            forward_only: false,
        }];

        let updated = upsert_route(
            &mut routes,
            RouteRule {
                id: "route-1".to_string(),
                matcher: "new".to_string(),
                match_type: MatchType::Contains,
                provider: "provider-b".to_string(),
                model: "model-b".to_string(),
                forward_only: false,
            },
        );

        assert!(updated);
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].matcher, "new");
        assert_eq!(routes[0].provider, "provider-b");
        assert_eq!(routes[0].model, "model-b");
    }

    #[test]
    fn upsert_route_appends_new_id() {
        let mut routes = Vec::new();

        let updated = upsert_route(
            &mut routes,
            RouteRule {
                id: "route-1".to_string(),
                matcher: "gpt".to_string(),
                match_type: MatchType::Contains,
                provider: "openrouter".to_string(),
                model: "openai/gpt-4.1".to_string(),
                forward_only: false,
            },
        );

        assert!(!updated);
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].id, "route-1");
    }

    #[test]
    fn delete_provider_removes_referencing_routes() {
        let mut providers = HashMap::new();
        providers.insert(
            "openrouter".to_string(),
            ProviderConfig {
                base_url: "https://openrouter.ai/api/v1".to_string(),
                api_key: "key".to_string(),
            },
        );
        providers.insert(
            "deepseek".to_string(),
            ProviderConfig {
                base_url: "https://api.deepseek.com/anthropic".to_string(),
                api_key: "key".to_string(),
            },
        );
        let mut config = ProtocolConfig {
            providers,
            routes: vec![
                RouteRule {
                    id: "route-1".to_string(),
                    matcher: "gpt".to_string(),
                    match_type: MatchType::Contains,
                    provider: "openrouter".to_string(),
                    model: "openai/gpt-4.1".to_string(),
                    forward_only: false,
                },
                RouteRule {
                    id: "route-2".to_string(),
                    matcher: "sonnet".to_string(),
                    match_type: MatchType::Contains,
                    provider: "deepseek".to_string(),
                    model: "deepseek-v4-pro".to_string(),
                    forward_only: false,
                },
            ],
        };

        let removed_routes = delete_provider(&mut config, "openrouter");

        assert_eq!(removed_routes, Some(1));
        assert!(!config.providers.contains_key("openrouter"));
        assert_eq!(config.routes.len(), 1);
        assert_eq!(config.routes[0].provider, "deepseek");
    }
}
