use anyhow::{anyhow, Result};
use clap::Parser;
use ellm::{Client, Config, Messages};

mod cli;
use cli::{Cli, Commands};
use serde::Deserialize;

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
            match bool(cli, question).await?.answer {
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

    let response = client
        .send_message(Messages::new().push_user(message).clone(), None, None)
        .await?;

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

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BoolResponse {
    answer: bool,
    explanation: String,
}

async fn bool(cli: Cli, message: String) -> Result<BoolResponse> {
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

    let system = concat!(
        "consider the question or statement and answer with a true or false.",
        "\nwhen unable to assess as a question or statement, default to false and explain.",
        "\nencode the result to a json object.",
        "\nthe object should have a key 'answer' with a boolean value.",
        "\nthe object should have a key 'explanation' with a string value.",
    );

    let mut result: Option<BoolResponse> = None;
    let mut messages = Messages::new().push_user(message).clone();
    'retry: for _retry in 0..3 {
        // https://github.com/anthropics/claude-cookbooks/blob/main/misc/how_to_enable_json_mode.ipynb
        let lead = "{";
        let mut response = client
            .send_message(
                messages.clone(),
                Some(lead.into()),
                Some(system.to_string()),
            )
            .await?;
        response.insert_str(0, lead);

        println!("{}", response);
        if let Err(error) = json::parse(&response) {
            println!("{}", error);
            messages.push_assistant(response);
            messages.push_user(error.to_string());
            continue 'retry;
        }
        match serde_json::from_str::<BoolResponse>(&response) {
            Ok(r) => {
                result = Some(r);
                break 'retry;
            }
            Err(error) => {
                println!("{}", error);
                messages.push_assistant(response);
                messages.push_user(format!("response did not match schema: {}", error));
                continue 'retry;
            }
        }
    }

    match result {
        Some(r) => Ok(r),
        None => Err(anyhow!("failed despite retries")),
    }
}
