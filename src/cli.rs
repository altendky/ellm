use clap::{Parser, Subcommand};

/// Claude CLI - Interact with Claude AI from the command line
#[derive(Parser, Debug)]
#[command(name = "ellm")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// API key for authentication (overrides environment and config file)
    #[arg(long, global = true)]
    pub api_key: Option<String>,

    /// Model to use
    #[arg(long, default_value = "claude-sonnet-4-5-20250929", global = true)]
    pub model: Option<String>,

    /// Maximum tokens to generate
    #[arg(long, default_value_t = 4096, global = true)]
    pub max_tokens: u32,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Send a message to Claude
    Send {
        /// The message to send
        message: String,
    },

    /// Show current configuration
    Config,

    /// Ask Claude a yes/no question and get a boolean response
    Bool {
        /// The question or prompt to ask
        question: String,
    },

    /// Book subcommand
    Book {
        /// The message to send
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_send() {
        let args = vec!["ellm", "send", "Hello, Claude!"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Send { message } => {
                assert_eq!(message, "Hello, Claude!");
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_cli_parse_with_api_key() {
        let args = vec!["ellm", "--api-key", "sk-ant-test", "send", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.api_key, Some("sk-ant-test".to_string()));
    }

    #[test]
    fn test_cli_parse_with_model() {
        let args = vec!["ellm", "--model", "claude-opus-4", "send", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.model, Some("claude-opus-4".to_string()));
    }

    #[test]
    fn test_cli_parse_with_max_tokens() {
        let args = vec!["ellm", "--max-tokens", "1000", "send", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.max_tokens, 1000);
    }

    #[test]
    fn test_cli_parse_config_command() {
        let args = vec!["ellm", "config"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(matches!(cli.command, Commands::Config));
    }

    #[test]
    fn test_cli_parse_bool() {
        let args = vec!["ellm", "bool", "Is Rust a systems programming language?"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Bool { question } => {
                assert_eq!(question, "Is Rust a systems programming language?");
            }
            _ => panic!("Expected Bool command"),
        }
    }

    #[test]
    fn test_cli_parse_bool_with_options() {
        let args = vec![
            "ellm",
            "--api-key",
            "sk-ant-test",
            "--max-tokens",
            "10",
            "bool",
            "Is the sky blue?",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.api_key, Some("sk-ant-test".to_string()));
        assert_eq!(cli.max_tokens, 10);

        match cli.command {
            Commands::Bool { question } => {
                assert_eq!(question, "Is the sky blue?");
            }
            _ => panic!("Expected Bool command"),
        }
    }
}
