use crate::error::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the Claude API client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// API key for authentication
    pub api_key: String,

    /// Base URL for the API (defaults to Anthropic's API)
    #[serde(default = "default_base_url")]
    pub base_url: String,

    /// Model to use (defaults to claude-sonnet-4-5)
    #[serde(default = "default_model")]
    pub model: String,

    /// Maximum tokens to generate
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_base_url() -> String {
    "https://api.anthropic.com/v1".to_string()
}

fn default_model() -> String {
    "claude-sonnet-4-5-20250929".to_string()
}

fn default_max_tokens() -> u32 {
    4096
}

impl Config {
    /// Create a new Config with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: default_base_url(),
            model: default_model(),
            max_tokens: default_max_tokens(),
        }
    }

    /// Load configuration from multiple sources with priority:
    /// 1. Provided api_key argument
    /// 2. Environment variable
    /// 3. Config file
    pub fn load(api_key: Option<String>) -> Result<Self> {
        // Priority 1: Provided API key
        if let Some(key) = api_key {
            return Ok(Self::new(key));
        }

        // Priority 2: Environment variable
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            return Ok(Self::new(key));
        }

        // Priority 3: Config file
        if let Ok(config) = Self::from_file() {
            return Ok(config);
        }

        Err(ConfigError::ApiKeyNotFound.into())
    }

    /// Load configuration from environment variables only
    pub fn from_env() -> Result<Self> {
        let api_key =
            std::env::var("ANTHROPIC_API_KEY").map_err(|_| ConfigError::ApiKeyNotFound)?;
        Ok(Self::new(api_key))
    }

    /// Load configuration from file
    pub fn from_file() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Err(ConfigError::FileNotFound(config_path.display().to_string()).into());
        }

        let contents = std::fs::read_to_string(&config_path)?;
        let config: Config =
            toml::from_str(&contents).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Ok(config)
    }

    /// Get the default config file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            ConfigError::ParseError("Could not determine config directory".to_string())
        })?;

        Ok(config_dir.join("ellm").join("config.toml"))
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(ConfigError::InvalidApiKey.into());
        }

        // Basic validation: API keys should start with "sk-ant-"
        if !self.api_key.starts_with("sk-ant-") {
            eprintln!("Warning: API key does not start with 'sk-ant-'. This may be invalid.");
        }

        Ok(())
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the maximum tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Build a Client from CLI arguments
    /// This is a convenience method that:
    /// 1. Loads config from multiple sources (CLI arg > env var > config file)
    /// 2. Applies CLI overrides for model and max_tokens
    /// 3. Creates and returns a Client
    pub fn build_from_cli(
        api_key: Option<String>,
        model: Option<String>,
        max_tokens: u32,
    ) -> Result<crate::Client> {
        let mut config = Self::load(api_key)?;

        // Apply CLI overrides
        if let Some(model) = model {
            config = config.with_model(model);
        }
        config = config.with_max_tokens(max_tokens);

        crate::Client::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config() {
        let config = Config::new("test-key");
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.base_url, "https://api.anthropic.com/v1");
        assert_eq!(config.model, "claude-sonnet-4-5-20250929");
        assert_eq!(config.max_tokens, 4096);
    }

    #[test]
    fn test_with_model() {
        let config = Config::new("test-key").with_model("claude-opus-4");
        assert_eq!(config.model, "claude-opus-4");
    }

    #[test]
    fn test_with_max_tokens() {
        let config = Config::new("test-key").with_max_tokens(1000);
        assert_eq!(config.max_tokens, 1000);
    }

    #[test]
    fn test_validate_empty_key() {
        let config = Config::new("");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_valid_key() {
        let config = Config::new("sk-ant-test-key");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::new("sk-ant-test-key");
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("api_key"));
        assert!(toml_str.contains("sk-ant-test-key"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            api_key = "sk-ant-test-key"
            base_url = "https://api.anthropic.com/v1"
            model = "claude-sonnet-4-5-20250929"
            max_tokens = 4096
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.api_key, "sk-ant-test-key");
    }
}
