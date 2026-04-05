use cloudflaredns::config::Config;
use std::fs;
use std::io::Write;
use std::path::{Component, PathBuf};

/// Helper to create a temporary config file at a specific path
fn setup_config_file(config_dir: &std::path::Path, content: &str) -> PathBuf {
    fs::create_dir_all(config_dir).expect("Failed to create config dir");
    let config_path = config_dir.join("config.yaml");
    let mut file = fs::File::create(&config_path).expect("Failed to create config file");
    file.write_all(content.as_bytes())
        .expect("Failed to write config");
    config_path
}

// ─── YAML config file parsing ────────────────────────────────────────────────

#[test]
fn test_config_yaml_valid_single_line() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join("config_single");
    let content = r#"cloudflare:
  api_token: token_abc123
  zone_id: zone_xyz789
"#;
    setup_config_file(&config_dir, content);

    // Parse directly to test the YAML structure
    let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(content);
    assert!(parsed.is_ok());
    let value = parsed.unwrap();
    assert_eq!(value["cloudflare"]["api_token"], "token_abc123");
    assert_eq!(value["cloudflare"]["zone_id"], "zone_xyz789");
}

#[test]
fn test_config_yaml_valid_with_comments() {
    let content = r#"# Cloudflare DNS Manager Configuration
cloudflare:
  # Your Cloudflare API token
  api_token: my-api-token-here
  # The zone ID for your domain
  zone_id: my-zone-id-here
"#;

    let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(content);
    assert!(parsed.is_ok());
    let value = parsed.unwrap();
    assert_eq!(value["cloudflare"]["api_token"], "my-api-token-here");
    assert_eq!(value["cloudflare"]["zone_id"], "my-zone-id-here");
}

#[test]
fn test_config_yaml_invalid_missing_cloudflare_key() {
    let content = r#"api_token: token_abc
zone_id: zone_xyz
"#;

    let parsed: Result<cloudflaredns::config::Config, _> = serde_yaml::from_str(content);
    assert!(parsed.is_err());
}

#[test]
fn test_config_yaml_invalid_malformed() {
    let content = "this: is: not: valid: yaml: [";

    let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(content);
    assert!(parsed.is_err());
}

// ─── Environment variable config ─────────────────────────────────────────────

#[test]
#[serial_test::serial(env_test)]
fn test_config_from_env_both_set() {
    unsafe {
        std::env::set_var("CLOUDFLARE_API_TOKEN", "env_token_123");
        std::env::set_var("CLOUDFLARE_ZONE_ID", "env_zone_456");
    }

    let config = Config::load_from_env().expect("Should load from env");
    assert_eq!(config.cloudflare_api_token, "env_token_123");
    assert_eq!(config.cloudflare_zone_id, "env_zone_456");

    unsafe {
        std::env::remove_var("CLOUDFLARE_API_TOKEN");
        std::env::remove_var("CLOUDFLARE_ZONE_ID");
    }
}

#[test]
#[serial_test::serial(env_test)]
fn test_config_from_env_empty_token() {
    unsafe {
        std::env::set_var("CLOUDFLARE_ZONE_ID", "env_zone_456");
        std::env::remove_var("CLOUDFLARE_API_TOKEN");
    }

    let result = Config::load_from_env();
    assert!(result.is_err());

    unsafe {
        std::env::remove_var("CLOUDFLARE_ZONE_ID");
    }
}

#[test]
#[serial_test::serial(env_test)]
fn test_config_from_env_empty_zone() {
    unsafe {
        std::env::set_var("CLOUDFLARE_API_TOKEN", "env_token_123");
        std::env::remove_var("CLOUDFLARE_ZONE_ID");
    }

    let result = Config::load_from_env();
    assert!(result.is_err());

    unsafe {
        std::env::remove_var("CLOUDFLARE_API_TOKEN");
    }
}

#[test]
#[serial_test::serial(env_test)]
fn test_config_from_env_neither_set() {
    unsafe {
        std::env::remove_var("CLOUDFLARE_API_TOKEN");
        std::env::remove_var("CLOUDFLARE_ZONE_ID");
    }

    let result = Config::load_from_env();
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("CLOUDFLARE_API_TOKEN"));
}

// ─── Config path generation ──────────────────────────────────────────────────

#[test]
fn test_config_path_is_in_home_directory() {
    let config_path = Config::config_path();
    let home = dirs::home_dir().expect("Should have home directory");

    assert!(config_path.starts_with(&home));
    assert!(config_path.ends_with(".config/cloudflaredns/config.yaml"));
}

#[test]
fn test_config_path_components() {
    let config_path = Config::config_path();
    let components: Vec<Component<'_>> = config_path.components().collect();

    // Should end with: .config / cloudflaredns / config.yaml
    assert!(components.iter().any(|c| c.as_os_str() == ".config"));
    assert!(components.iter().any(|c| c.as_os_str() == "cloudflaredns"));
    assert!(components.iter().any(|c| c.as_os_str() == "config.yaml"));
}

// ─── Config struct properties ────────────────────────────────────────────────

#[test]
fn test_config_struct_fields_accessible() {
    let config = Config {
        cloudflare_api_token: "test_token".to_string(),
        cloudflare_zone_id: "test_zone".to_string(),
    };

    assert_eq!(config.cloudflare_api_token, "test_token");
    assert_eq!(config.cloudflare_zone_id, "test_zone");
}

#[test]
fn test_config_debug_format() {
    let config = Config {
        cloudflare_api_token: "secret_token".to_string(),
        cloudflare_zone_id: "zone_123".to_string(),
    };

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("Config"));
    assert!(debug_str.contains("cloudflare_api_token"));
    assert!(debug_str.contains("cloudflare_zone_id"));
}
