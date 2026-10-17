use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_models_dir")]
    pub models_dir: String,

    #[serde(default = "default_target_lang")]
    pub default_target_lang: String,
}

fn default_models_dir() -> String {
    "~/.cache/sub-llama/models".to_string()
}

fn default_target_lang() -> String {
    "pt".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            models_dir: default_models_dir(),
            default_target_lang: default_target_lang(),
        }
    }
}

pub fn get_config_dir() -> Result<PathBuf> {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(config_home).join("sub-llama"));
    }
    let home = std::env::var("HOME").context("HOME environment variable not found")?;
    Ok(PathBuf::from(home).join(".config").join("sub-llama"))
}

pub fn load_or_create_config() -> Result<AppConfig> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.toml");

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;
        let config: AppConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML in {}", config_path.display()))?;
        return Ok(config);
    }

    // Criar configuração padrão se não existir
    let default_config = AppConfig::default();
    let toml_str = toml::to_string_pretty(&default_config)?;
    let _ = fs::create_dir_all(&config_dir);
    let _ = fs::write(&config_path, toml_str);

    Ok(default_config)
}

pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();
    if let Ok(stripped) = p.strip_prefix("~") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    p.to_path_buf()
}
