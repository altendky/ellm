//! # ellm
//!
//! A Rust library and CLI for interacting with Claude AI and other LLMs via the Anthropic API.
//!
//! ## Features
//!
//! - Async-first design using tokio and reqwest
//! - Multiple API key configuration methods
//! - Type-safe error handling
//! - Easy-to-use client interface
//!
//! ## Example
//!
//! ```no_run
//! use ellm::{Client, Config};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load config from environment or file
//!     let config = Config::load(None)?;
//!
//!     // Create a client
//!     let client = Client::new(config)?;
//!
//!     // Send a message
//!     let response = client.send_message("Hello, Claude!", None, None).await?;
//!     println!("Response: {}", response);
//!
//!     Ok(())
//! }
//! ```

mod client;
mod config;
mod error;

// Re-export main types
pub use client::{Client, Message};
pub use config::Config;
pub use error::{ApiError, ClaudeError, ConfigError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports() {
        // Ensure all main types are accessible
        let config = Config::new("sk-ant-test-key");
        assert_eq!(config.api_key, "sk-ant-test-key");
    }
}
