use crate::config::schema::AppConfig;
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
openai = ["openai-gpt"]
anthropic = ["anthropic-sonnet"]
"#;

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
