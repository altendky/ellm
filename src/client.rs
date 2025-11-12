use crate::config::Config;
use crate::error::{ApiError, Result};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

/// Claude API client
pub struct Client {
    http_client: HttpClient,
    config: Config,
}

/// Request structure for the Messages API
#[derive(Debug, Serialize)]
struct MessageRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    messages: Vec<Message>,
}

/// Message structure for API requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

// TODO: do i really want Clone?
#[derive(Clone, Debug, Serialize)]
pub struct Messages {
    _messages: Vec<Message>,
}

impl Default for Messages {
    fn default() -> Self {
        Self::new()
    }
}

impl Messages {
    pub fn new() -> Messages {
        Messages { _messages: vec![] }
    }

    pub fn push_user(&mut self, content: String) -> &mut Self {
        self._messages.push(Message {
            role: "user".into(),
            content,
        });

        self
    }

    pub fn push_assistant(&mut self, content: String) -> &mut Self {
        self._messages.push(Message {
            role: "assistant".into(),
            content,
        });

        self
    }
}

impl From<Messages> for Vec<Message> {
    fn from(value: Messages) -> Self {
        value._messages
    }
}

/// Response structure from the Messages API
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MessageResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ContentBlock>,
    model: String,
    stop_reason: Option<String>,
    usage: Usage,
}

/// Content block in the response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: String,
}

/// Usage statistics from the API
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Error response from the API
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ErrorResponse {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

impl Client {
    /// Create a new Claude API client
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;

        let http_client = HttpClient::builder()
            .build()
            .map_err(|e| ApiError::InvalidRequest(e.to_string()))?;

        Ok(Self {
            http_client,
            config,
        })
    }

    /// Send a message to Claude and get a response
    pub async fn send_message(
        &self,
        mut messages: Messages,
        lead: Option<String>,
        system: Option<String>,
    ) -> Result<String> {
        if let Some(lead) = lead {
            messages.push_assistant(lead);
        };

        let request = MessageRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            system,
            temperature: Some(0f32),
            messages: messages.into(),
        };

        let url = format!("{}/messages", self.config.base_url);

        let request = self
            .http_client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request);

        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            // Try to parse as error response
            if let Ok(error_resp) = serde_json::from_str::<ErrorResponse>(&body) {
                return match status.as_u16() {
                    401 => Err(ApiError::AuthenticationFailed(error_resp.message).into()),
                    429 => Err(ApiError::RateLimitExceeded.into()),
                    _ => Err(ApiError::ApiError {
                        status: status.as_u16(),
                        message: error_resp.message,
                    }
                    .into()),
                };
            }

            return Err(ApiError::ApiError {
                status: status.as_u16(),
                message: body,
            }
            .into());
        }

        let message_response: MessageResponse =
            serde_json::from_str(&body).map_err(|e| ApiError::UnexpectedResponse(e.to_string()))?;

        // Extract the text from the first content block
        let text = message_response
            .content
            .first()
            .map(|block| block.text.clone())
            .ok_or_else(|| ApiError::UnexpectedResponse("No content in response".to_string()))?;

        Ok(text)
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let message = Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        assert_eq!(message.role, "user");
        assert_eq!(message.content, "Hello");
    }

    #[test]
    fn test_message_request_serialization() {
        let request = MessageRequest {
            model: "claude-sonnet-4-5-20250929".to_string(),
            max_tokens: 1024,
            system: None,
            temperature: None,
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("claude-sonnet-4-5-20250929"));
        assert!(json.contains("Hello"));
        assert!(json.contains("1024"));
    }

    #[test]
    fn test_client_creation_with_valid_config() {
        let config = Config::new("sk-ant-test-key");
        let client = Client::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_creation_with_invalid_config() {
        let config = Config::new("");
        let client = Client::new(config);
        assert!(client.is_err());
    }

    // Note: We don't test actual API calls here to avoid requiring real API keys
    // Integration tests with mocking would be in the tests/ directory
}
