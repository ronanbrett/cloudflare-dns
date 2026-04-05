use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub cloudflare_api_token: String,
    pub cloudflare_zone_id: String,
}

impl Config {
    /// Load configuration with fallback chain:
    /// 1. ~/.config/cloudflare-dns/config.yaml
    /// 2. ./.env (current directory)
    /// 3. Environment variables
    pub fn load() -> Result<Self> {
        // Try config file first
        if let Ok(config) = Self::load_from_file() {
            return Ok(config);
        }

        // Try .env file
        dotenvy::dotenv().ok();

        // Fall back to environment variables
        Self::load_from_env()
    }

    fn load_from_file() -> Result<Self> {
        let config_dir = dirs::home_dir()
            .context("Could not determine home directory")?
            .join(".config")
            .join("cloudflaredns");

        let config_path = config_dir.join("config.yaml");

        if !config_path.exists() {
            println!("Config file not found at {}", config_path.display());
            anyhow::bail!("Config file not found at {}", config_path.display());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let config: ConfigFile = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

        Ok(Config {
            cloudflare_api_token: config.cloudflare.api_token,
            cloudflare_zone_id: config.cloudflare.zone_id,
        })
    }

    fn load_from_env() -> Result<Self> {
        let api_token = env::var("CLOUDFLARE_API_TOKEN").context(
            "CLOUDFLARE_API_TOKEN not set in config file, .env, or environment variables",
        )?;

        let zone_id = env::var("CLOUDFLARE_ZONE_ID")
            .context("CLOUDFLARE_ZONE_ID not set in config file, .env, or environment variables")?;

        Ok(Config {
            cloudflare_api_token: api_token,
            cloudflare_zone_id: zone_id,
        })
    }

    /// Get the expected config file path for display purposes
    pub fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".config")
            .join("cloudflare-dns")
            .join("config.yaml")
    }
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    cloudflare: CloudflareConfig,
}

#[derive(Debug, Deserialize)]
struct CloudflareConfig {
    api_token: String,
    zone_id: String,
}
