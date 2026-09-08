use crate::config::schema::{AppConfig, RouteTableEntry};
use std::fs;
use std::path::{Path, PathBuf};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

const APP_DIR_NAME: &str = "yakumo_router";
const CONFIG_FILE_NAME: &str = "config.toml";

pub const DEFAULT_CONFIG: &str = r#"active_route_table = "default"

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

[[route_tables.default.openai]]
id = "openai-gpt"
enabled = true

[[route_tables.default.anthropic]]
id = "anthropic-sonnet"
enabled = true
"#;

pub const MINIMAL_CONFIG: &str = r#"[server]
host = "127.0.0.1"
port = 8989

[tls]
cert = "cert.pem"
key = "key.pem"
"#;

pub fn config_path(data_dir: &Path) -> PathBuf {
    data_dir.join(CONFIG_FILE_NAME)
}

pub fn load_config(config_path: &Path) -> Result<AppConfig, BoxError> {
    let config_text = fs::read_to_string(config_path)?;
    let mut config: AppConfig = toml::from_str(&config_text)?;
    migrate_legacy_route_tables(&mut config);
    Ok(config)
}

fn seed_missing_entries(entries: &mut Vec<RouteTableEntry>, known_ids: &[String]) {
    for id in known_ids {
        if !entries.iter().any(|entry| &entry.id == id) {
            entries.push(RouteTableEntry {
                id: id.clone(),
                enabled: false,
            });
        }
    }
}

/// Old configs stored only the enabled rule ids, so rules that were disabled are
/// absent from the file. Backfill them as disabled entries once, so a table keeps
/// showing the same rows it did before the format change.
pub fn migrate_legacy_route_tables(config: &mut AppConfig) {
    let openai_ids: Vec<String> = config
        .openai
        .routes
        .iter()
        .map(|route| route.id.clone())
        .collect();
    let anthropic_ids: Vec<String> = config
        .anthropic
        .routes
        .iter()
        .map(|route| route.id.clone())
        .collect();

    for table in config.route_tables.values_mut() {
        if table.legacy_openai {
            seed_missing_entries(&mut table.openai, &openai_ids);
            table.legacy_openai = false;
        }
        if table.legacy_anthropic {
            seed_missing_entries(&mut table.anthropic, &anthropic_ids);
            table.legacy_anthropic = false;
        }
    }
}

pub fn load_or_default_config(config_path: &Path) -> Result<AppConfig, BoxError> {
    if config_path.exists() {
        load_config(config_path)
    } else {
        Ok(AppConfig::default())
    }
}

/// Write a file that may contain secrets, restricting it to the owner on Unix.
pub fn write_private_file(path: &Path, contents: &str) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(contents.as_bytes())?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        Ok(())
    }

    #[cfg(not(unix))]
    {
        fs::write(path, contents)
    }
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

    write_private_file(&config_path, DEFAULT_CONFIG)?;
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

pub fn create_minimal_config(data_dir: &Path) -> Result<AppConfig, BoxError> {
    fs::create_dir_all(data_dir)?;

    let config_path = config_path(data_dir);
    let config = AppConfig::default();
    let minimal_text = MINIMAL_CONFIG;
    write_private_file(&config_path, minimal_text)?;
    Ok(config)
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

#[cfg(test)]
mod tests {
    use super::migrate_legacy_route_tables;
    use crate::config::schema::{AppConfig, MatchType, RouteRule, RouteTable, RouteTableEntry};

    fn route(id: &str) -> RouteRule {
        RouteRule {
            id: id.to_string(),
            matcher: id.to_string(),
            match_type: MatchType::Contains,
            provider: "provider".to_string(),
            model: "model".to_string(),
            forward_only: false,
        }
    }

    #[test]
    fn migration_seeds_missing_rules_as_disabled_entries() {
        let mut config = AppConfig::default();
        config.openai.routes = vec![route("a"), route("b"), route("c")];
        config.route_tables.insert(
            "main".to_string(),
            RouteTable {
                openai: vec![RouteTableEntry {
                    id: "b".to_string(),
                    enabled: true,
                }],
                legacy_openai: true,
                ..Default::default()
            },
        );

        migrate_legacy_route_tables(&mut config);

        let table = config.route_tables.get("main").expect("table exists");
        assert_eq!(table.openai.len(), 3);
        assert_eq!(table.openai[0].id, "b");
        assert!(table.openai[0].enabled);
        assert_eq!(table.openai[1].id, "a");
        assert!(!table.openai[1].enabled);
        assert_eq!(table.openai[2].id, "c");
        assert!(!table.openai[2].enabled);
        assert!(!table.legacy_openai);
    }
}
