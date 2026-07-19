use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;

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

impl Protocol {
    pub fn config<'a>(&self, config: &'a AppConfig) -> &'a ProtocolConfig {
        match self {
            Protocol::OpenAi => &config.openai,
            Protocol::Anthropic => &config.anthropic,
        }
    }

    pub fn config_mut<'a>(&self, config: &'a mut AppConfig) -> &'a mut ProtocolConfig {
        match self {
            Protocol::OpenAi => &mut config.openai,
            Protocol::Anthropic => &mut config.anthropic,
        }
    }

    pub fn table_routes_mut<'a>(&self, table: &'a mut RouteTable) -> &'a mut Vec<String> {
        match self {
            Protocol::OpenAi => &mut table.openai,
            Protocol::Anthropic => &mut table.anthropic,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::AppConfig;
    use crate::config::storage::DEFAULT_CONFIG;

    #[test]
    fn default_config_has_active_route_table_at_root() {
        let config: AppConfig = toml::from_str(DEFAULT_CONFIG).expect("default config parses");

        assert_eq!(config.active_route_table.as_deref(), Some("default"));
        assert!(config.route_tables.contains_key("default"));
    }
}
