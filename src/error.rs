use thiserror::Error;

/// Main error type for the ellm library
#[derive(Error, Debug)]
pub enum ClaudeError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// API-related errors
    #[error("API error: {0}")]
    Api(#[from] ApiError),

    /// Network/HTTP errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Configuration-specific errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("API key not found. Please set ANTHROPIC_API_KEY environment variable, provide --api-key argument, or create a config file at ~/.config/ellm/config.toml")]
    ApiKeyNotFound,

    #[error("Invalid API key format")]
    InvalidApiKey,

    #[error("Failed to parse config file: {0}")]
    ParseError(String),

    #[error("Config file not found at: {0}")]
    FileNotFound(String),
}

/// API-specific errors
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("API returned error {status}: {message}")]
    ApiError { status: u16, message: String },

    #[error("Unexpected response format: {0}")]
    UnexpectedResponse(String),
}

/// Type alias for Results using ClaudeError
pub type Result<T> = std::result::Result<T, ClaudeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ClaudeError::Config(ConfigError::ApiKeyNotFound);
        assert!(err.to_string().contains("API key not found"));
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::ApiError {
            status: 401,
            message: "Unauthorized".to_string(),
        };
        assert!(err.to_string().contains("401"));
        assert!(err.to_string().contains("Unauthorized"));
    }

    #[test]
    fn test_config_error_from() {
        let config_err = ConfigError::ApiKeyNotFound;
        let claude_err: ClaudeError = config_err.into();
        assert!(matches!(claude_err, ClaudeError::Config(_)));
    }
}
