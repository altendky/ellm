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
    let client = Config::build_from_cli(cli.api_key, cli.model, cli.max_tokens)?;

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

/// Sends a message to the Claude API with retry logic for JSON responses.
///
/// This function attempts to get a valid JSON response of type `T` from the API,
/// retrying up to `max_retries` times if parsing fails. Each failed attempt
/// includes the error in the conversation to help the model correct its response.
///
/// # Arguments
/// * `client` - The API client to use for sending messages
/// * `messages` - The conversation messages to send
/// * `system` - Optional system prompt to guide the model's behavior
/// * `max_retries` - Maximum number of retry attempts (default: 3)
///
/// # Returns
/// * `Ok(T)` - Successfully parsed response of type T
/// * `Err` - If all retry attempts fail or an API error occurs
async fn send_with_json_retry<T>(
    client: &Client,
    mut messages: Messages,
    system: Option<String>,
    max_retries: usize,
) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let mut result: Option<T> = None;

    'retry: for _retry in 0..max_retries {
        // https://github.com/anthropics/claude-cookbooks/blob/main/misc/how_to_enable_json_mode.ipynb
        let lead = "{";
        let mut response = client
            .send_message(messages.clone(), Some(lead.into()), system.clone())
            .await?;
        response.insert_str(0, lead);

        println!("{}", response);

        // First validate as generic JSON
        if let Err(error) = json::parse(&response) {
            println!("{}", error);
            messages.push_assistant(response);
            messages.push_user(error.to_string());
            continue 'retry;
        }

        // Then try to parse into the specific type
        match serde_json::from_str::<T>(&response) {
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

    result.ok_or_else(|| anyhow!("failed to get valid response despite retries"))
}

async fn bool(cli: Cli, message: String) -> Result<BoolResponse> {
    let client = Config::build_from_cli(cli.api_key, cli.model, cli.max_tokens)?;

    println!("Sending message to Claude...\n");

    let system = "consider the question or statement and answer with a true or false.
when unable to assess as a question or statement, default to false and explain.
encode the result to a json object.
the object should have a key 'answer' with a boolean value.
the object should have a key 'explanation' with a string value.";

    let messages = Messages::new().push_user(message).clone();

    send_with_json_retry::<BoolResponse>(&client, messages, Some(system.to_string()), 3).await
}
