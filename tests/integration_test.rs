use ellm::{Client, Config};

#[test]
fn test_config_creation() {
    let config = Config::new("sk-ant-test-key-12345");
    assert_eq!(config.api_key, "sk-ant-test-key-12345");
    assert_eq!(config.model, "claude-sonnet-4-5-20250929");
}

#[test]
fn test_config_with_custom_model() {
    let config = Config::new("sk-ant-test-key")
        .with_model("claude-opus-4")
        .with_max_tokens(2000);

    assert_eq!(config.model, "claude-opus-4");
    assert_eq!(config.max_tokens, 2000);
}

#[test]
fn test_client_creation() {
    let config = Config::new("sk-ant-test-key-12345");
    let result = Client::new(config);

    // Should succeed with a properly formatted key
    assert!(result.is_ok());
}

#[test]
fn test_client_creation_fails_with_empty_key() {
    let config = Config::new("");
    let result = Client::new(config);

    // Should fail with empty key
    assert!(result.is_err());
}

#[test]
fn test_config_load_priority() {
    // Test that we can create a config from an explicit API key
    let config = Config::load(Some("sk-ant-explicit-key".to_string()));
    assert!(config.is_ok());
    assert_eq!(config.unwrap().api_key, "sk-ant-explicit-key");
}

// Note: We don't test actual API calls in integration tests without mocking
// to avoid requiring real API keys and making actual API requests during testing.
// For real API testing, you would:
// 1. Use a mocking library like `mockito` or `wiremock`
// 2. Set up test fixtures with mock responses
// 3. Test error handling paths

#[cfg(feature = "live_api_tests")]
mod live_tests {
    use super::*;

    // These tests only run when explicitly enabled with --features live_api_tests
    // and require a valid API key in the environment

    #[tokio::test]
    async fn test_real_api_call() {
        let config = Config::from_env().expect("ANTHROPIC_API_KEY must be set for live tests");
        let client = Client::new(config).expect("Failed to create client");

        let response = client
            .send_message("Say 'Hello' and nothing else.", None, None)
            .await
            .expect("API call failed");

        assert!(!response.is_empty());
        println!("API Response: {}", response);
    }
}
