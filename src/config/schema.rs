use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub tls: TlsConfig,
    #[serde(default)]
    pub openai: ProtocolConfig,
    #[serde(default)]
    pub anthropic: ProtocolConfig,
    #[serde(default)]
    pub active_route_table: Option<String>,
    #[serde(default)]
    pub route_tables: HashMap<String, RouteTable>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            tls: TlsConfig::default(),
            openai: ProtocolConfig::default(),
            anthropic: ProtocolConfig::default(),
            active_route_table: None,
            route_tables: HashMap::new(),
        }
    }
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

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            providers: HashMap::new(),
            routes: Vec::new(),
        }
    }
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteTableEntry {
    pub id: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct RouteTable {
    pub openai: Vec<RouteTableEntry>,
    pub anthropic: Vec<RouteTableEntry>,
    #[serde(skip)]
    pub legacy_openai: bool,
    #[serde(skip)]
    pub legacy_anthropic: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawRouteTableEntry {
    Id(String),
    Detailed {
        id: String,
        #[serde(default)]
        enabled: bool,
    },
}

impl<'de> Deserialize<'de> for RouteTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawRouteTable {
            #[serde(default)]
            openai: Vec<RawRouteTableEntry>,
            #[serde(default)]
            anthropic: Vec<RawRouteTableEntry>,
        }

        fn convert(entries: Vec<RawRouteTableEntry>) -> (Vec<RouteTableEntry>, bool) {
            let mut legacy = false;
            let converted = entries
                .into_iter()
                .map(|entry| match entry {
                    RawRouteTableEntry::Id(id) => {
                        legacy = true;
                        RouteTableEntry { id, enabled: true }
                    }
                    RawRouteTableEntry::Detailed { id, enabled } => RouteTableEntry { id, enabled },
                })
                .collect();

            (converted, legacy)
        }

        let raw = RawRouteTable::deserialize(deserializer)?;
        let (openai, legacy_openai) = convert(raw.openai);
        let (anthropic, legacy_anthropic) = convert(raw.anthropic);

        Ok(RouteTable {
            openai,
            anthropic,
            legacy_openai,
            legacy_anthropic,
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
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

    pub fn table_routes_mut<'a>(&self, table: &'a mut RouteTable) -> &'a mut Vec<RouteTableEntry> {
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
    use super::{AppConfig, RouteTable, RouteTableEntry};
    use crate::config::storage::{DEFAULT_CONFIG, MINIMAL_CONFIG};

    #[test]
    fn default_config_has_active_route_table_at_root() {
        let config: AppConfig = toml::from_str(DEFAULT_CONFIG).expect("default config parses");

        assert_eq!(config.active_route_table.as_deref(), Some("default"));
        assert!(config.route_tables.contains_key("default"));
    }

    #[test]
    fn minimal_config_defaults_routes_and_providers() {
        let config: AppConfig = toml::from_str(MINIMAL_CONFIG).expect("minimal config parses");

        assert!(config.openai.providers.is_empty());
        assert!(config.openai.routes.is_empty());
        assert!(config.anthropic.providers.is_empty());
        assert!(config.anthropic.routes.is_empty());
        assert!(config.route_tables.is_empty());
        assert_eq!(config.active_route_table, None);
    }

    #[test]
    fn legacy_route_table_ids_deserialize_as_enabled_entries() {
        let config: AppConfig = toml::from_str(
            r#"
[route_tables.main]
openai = ["route-a", "route-b"]
anthropic = ["route-c"]
"#,
        )
        .expect("legacy config parses");

        let table = config.route_tables.get("main").expect("table exists");
        assert_eq!(table.openai.len(), 2);
        assert!(table.openai.iter().all(|entry| entry.enabled));
        assert!(table.legacy_openai);
        assert!(table.legacy_anthropic);
    }

    #[test]
    fn detailed_route_table_entries_preserve_enabled_flag() {
        let config: AppConfig = toml::from_str(
            r#"
[route_tables.main]
openai = [
    { id = "route-a", enabled = true },
    { id = "route-b", enabled = false },
]
"#,
        )
        .expect("detailed config parses");

        let table = config.route_tables.get("main").expect("table exists");
        assert_eq!(table.openai.len(), 2);
        assert!(table.openai[0].enabled);
        assert!(!table.openai[1].enabled);
        assert!(!table.legacy_openai);
    }

    #[test]
    fn route_table_entries_round_trip_through_toml() {
        let mut config = AppConfig::default();
        config.route_tables.insert(
            "main".to_string(),
            RouteTable {
                openai: vec![
                    RouteTableEntry {
                        id: "route-a".to_string(),
                        enabled: true,
                    },
                    RouteTableEntry {
                        id: "route-b".to_string(),
                        enabled: false,
                    },
                ],
                ..Default::default()
            },
        );

        let text = toml::to_string_pretty(&config).expect("config serializes");
        println!("--- serialized ---\n{text}--- end ---");

        let parsed: AppConfig = toml::from_str(&text).expect("config re-parses");
        let table = parsed.route_tables.get("main").expect("table exists");
        assert_eq!(table.openai.len(), 2);
        assert!(table.openai[0].enabled);
        assert!(!table.openai[1].enabled);
        assert!(!table.legacy_openai);
    }
}
