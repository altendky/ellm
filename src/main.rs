use anyhow::Result;
use clap::Parser;
use ellm::{Client, Config};

mod cli;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.clone() {
        Commands::Send { message } => {
            send_message(cli, message).await?;
        }
        Commands::Config => {
            show_config(cli)?;
        }
        Commands::Bool { question } => {
            match bool(cli, question).await? {
                true => (),
                // TODO: is this actually kind with tokio?
                false => std::process::exit(1),
            };
        }
    }

    Ok(())
}

async fn send_message(cli: Cli, message: String) -> Result<()> {
    // Load configuration with priority: CLI arg > env var > config file
    let mut config = Config::load(cli.api_key)?;

    // Apply CLI overrides
    if let Some(model) = cli.model {
        config = config.with_model(model);
    }
    config = config.with_max_tokens(cli.max_tokens);

    // Create client and send message
    let client = Client::new(config)?;

    println!("Sending message to Claude...\n");

    let response = client.send_message(message, None).await?;

    println!("{}", response);

    Ok(())
}

fn show_config(cli: Cli) -> Result<()> {
    let config = Config::load(cli.api_key)?;

    println!("Current Configuration:");
    println!(
        "  API Key: {}***",
        &config.api_key[..10.min(config.api_key.len())]
    );
    println!("  Base URL: {}", config.base_url);
    println!("  Model: {}", config.model);
    println!("  Max Tokens: {}", config.max_tokens);

    if let Ok(config_path) = Config::config_path() {
        println!("\nConfig file location: {}", config_path.display());
        if config_path.exists() {
            println!("  Status: Found");
        } else {
            println!("  Status: Not found");
        }
    }

    Ok(())
}

async fn bool(cli: Cli, message: String) -> Result<bool> {
    // Load configuration with priority: CLI arg > env var > config file
    let mut config = Config::load(cli.api_key)?;

    // Apply CLI overrides
    if let Some(model) = cli.model {
        config = config.with_model(model);
    }
    config = config.with_max_tokens(1);

    // Create client and send message
    let client = Client::new(config)?;

    println!("Sending message to Claude...\n");

    let response = client
        .send_message(
            message,
            Some("answer with a json-serialized boolean and no markup".to_string()),
        )
        .await?;
    let result = serde_json::from_str(&response)?;
    println!("{}", response);

    Ok(result)
}
