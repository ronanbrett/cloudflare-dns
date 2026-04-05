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
    /// 1. ~/.config/cloudflaredns/config.yaml
    /// 2. ./.env (current directory)
    /// 3. Environment variables
    pub fn load() -> Result<Self> {
        // Try config file first
        if let Ok(config) = Self::load_from_file() {
            config.validate()?;
            return Ok(config);
        }

        // Try .env file
        dotenvy::dotenv().ok();

        // Fall back to environment variables
        let config = Self::load_from_env()?;
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values.
    fn validate(&self) -> Result<()> {
        if self.cloudflare_api_token.is_empty() {
            anyhow::bail!("Cloudflare API token cannot be empty");
        }

        if self.cloudflare_zone_id.is_empty() {
            anyhow::bail!("Cloudflare zone ID cannot be empty");
        }

        // Basic format validation - zone IDs are typically 32 character hex strings
        if !self
            .cloudflare_zone_id
            .chars()
            .all(|c| c.is_ascii_hexdigit() || c == '-')
        {
            anyhow::bail!("Cloudflare zone ID appears to be in an invalid format");
        }

        // API tokens should be reasonably long
        if self.cloudflare_api_token.len() < 20 {
            anyhow::bail!("Cloudflare API token appears to be in an invalid format (too short)");
        }

        Ok(())
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

    /// Load configuration from environment variables only.
    /// Useful for testing or when no config file is available.
    pub fn load_from_env() -> Result<Self> {
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
            .join("cloudflaredns")
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn create_temp_config_file(
        api_token: &str,
        zone_id: &str,
    ) -> (tempfile::TempDir, std::path::PathBuf) {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let config_dir = temp_dir.path().join(".config").join("cloudflaredns");
        fs::create_dir_all(&config_dir).expect("Failed to create config dir");

        let config_path = config_dir.join("config.yaml");
        let content = format!(
            r#"cloudflare:
  api_token: {}
  zone_id: {}
"#,
            api_token, zone_id
        );

        let mut file = fs::File::create(&config_path).expect("Failed to create config file");
        file.write_all(content.as_bytes())
            .expect("Failed to write config");

        (temp_dir, config_path)
    }

    #[test]
    fn test_config_from_env() {
        // Set environment variables for testing
        unsafe {
            std::env::set_var("CLOUDFLARE_API_TOKEN", "test_api_token_1234567890abcdef");
            std::env::set_var("CLOUDFLARE_ZONE_ID", "0123456789abcdef0123456789abcdef");
        }

        let config = Config::load_from_env().expect("Failed to load config from env");
        assert_eq!(
            config.cloudflare_api_token,
            "test_api_token_1234567890abcdef"
        );
        assert_eq!(
            config.cloudflare_zone_id,
            "0123456789abcdef0123456789abcdef"
        );

        // Clean up
        unsafe {
            std::env::remove_var("CLOUDFLARE_API_TOKEN");
            std::env::remove_var("CLOUDFLARE_ZONE_ID");
        }
    }

    #[test]
    fn test_config_from_env_missing_token() {
        unsafe {
            std::env::set_var("CLOUDFLARE_ZONE_ID", "0123456789abcdef0123456789abcdef");
            std::env::remove_var("CLOUDFLARE_API_TOKEN");
        }

        let result = Config::load_from_env();
        assert!(result.is_err());

        unsafe {
            std::env::remove_var("CLOUDFLARE_ZONE_ID");
        }
    }

    #[test]
    fn test_config_from_env_missing_zone_id() {
        unsafe {
            std::env::set_var("CLOUDFLARE_API_TOKEN", "test_api_token_1234567890abcdef");
            std::env::remove_var("CLOUDFLARE_ZONE_ID");
        }

        let result = Config::load_from_env();
        assert!(result.is_err());

        unsafe {
            std::env::remove_var("CLOUDFLARE_API_TOKEN");
        }
    }

    #[test]
    fn test_config_path_generation() {
        let config_path = Config::config_path();
        // Should end with ~/.config/cloudflaredns/config.yaml
        let path_str = config_path.to_string_lossy();
        assert!(path_str.contains(".config/cloudflaredns/config.yaml"));
    }

    #[test]
    fn test_config_file_parsing() {
        let (temp_dir, config_path) = create_temp_config_file("file_token_abc", "file_zone_xyz");

        // Read and parse the file manually to test the parsing logic
        let content = fs::read_to_string(&config_path).expect("Failed to read config file");
        let config_file: ConfigFile =
            serde_yaml::from_str(&content).expect("Failed to parse config");

        assert_eq!(config_file.cloudflare.api_token, "file_token_abc");
        assert_eq!(config_file.cloudflare.zone_id, "file_zone_xyz");

        // Prevent temp dir from being dropped until end of test
        drop(temp_dir);
    }

    #[test]
    fn test_config_file_not_found() {
        // Use a unique temp dir that definitely has no config
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let fake_config_dir = temp_dir.path().join("nonexistent").join("cloudflaredns");

        // Try to load from a path that doesn't exist
        let result = fs::read_to_string(fake_config_dir.join("config.yaml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_yaml_invalid_format() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let config_dir = temp_dir
            .path()
            .join("test_config_invalid")
            .join("cloudflaredns");
        fs::create_dir_all(&config_dir).expect("Failed to create config dir");
        let config_path = config_dir.join("config.yaml");

        // Write invalid YAML
        let mut file = fs::File::create(&config_path).expect("Failed to create config file");
        file.write_all(b"invalid: yaml: : content: [")
            .expect("Failed to write invalid YAML");

        // Read and parse directly - should fail on invalid YAML
        let content = fs::read_to_string(&config_path).expect("Failed to read config file");
        let result: Result<ConfigFile, _> = serde_yaml::from_str(&content);
        assert!(result.is_err());
    }
}
