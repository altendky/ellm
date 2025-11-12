use std::collections::HashMap;

use anyhow::{anyhow, Result};
use clap::Parser;
use ellm::{Client, Config, Messages};

mod cli;
use cli::{Cli, Commands};
use schemars::JsonSchema;
use serde::Deserialize;

/// Helper function to build a Client from Cli struct
fn build_client(cli: &Cli) -> Result<Client> {
    Ok(Config::build_from_cli(
        cli.api_key.clone(),
        cli.model.clone(),
        cli.max_tokens,
    )?)
}

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
        Commands::Book { message } => {
            book(cli, message).await?;
        }
    }

    Ok(())
}

async fn send_message(cli: Cli, message: String) -> Result<()> {
    let client = build_client(&cli)?;

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

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct BoolResponse {
    /// when unable to assess the input clearly, default to false
    answer: bool,
    /// provide an explanation of how you reached the answer
    explanation: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct Book {
    title: String,
    authors: Vec<String>,
    /// score as 1 if no indication given
    #[schemars(range(min = -2, max = 2))]
    score: i8,
    themes: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct Series {
    title: String,
    authors: Vec<String>,
    /// score as null if no indication given
    #[schemars(range(min = -1, max = 1))]
    score: Option<i8>,
    books: Vec<Book>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct BookResponse {
    books: Vec<Book>,
    series: Vec<Series>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct RecommendationResponse {
    books: Vec<Book>,
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
    T: serde::de::DeserializeOwned + JsonSchema,
{
    let schema = schemars::schema_for!(T);
    let schema_json = serde_json::to_string_pretty(&schema)?;
    let jsonschema_system = format!(
        "encode the result to a json object that matches the following JSON schema:\n\n{}",
        schema_json
    );
    let system = if let Some(system) = system {
        format!("{}\n\n{}", system, jsonschema_system)
    } else {
        jsonschema_system
    };

    let mut result: Option<T> = None;

    'retry: for _retry in 0..max_retries {
        // https://github.com/anthropics/claude-cookbooks/blob/main/misc/how_to_enable_json_mode.ipynb
        let lead = "{";
        let mut response = client
            .send_message(messages.clone(), Some(lead.into()), Some(system.clone()))
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
    let client = build_client(&cli)?;

    println!("Sending message to Claude...\n");

    let system = "consider the question or statement and answer with a true or false.".into();

    let messages = Messages::new().push_user(message).clone();

    send_with_json_retry::<BoolResponse>(&client, messages, Some(system), 3).await
}

async fn book(cli: Cli, message: String) -> Result<()> {
    let client = Config::build_from_cli(cli.api_key, cli.model, cli.max_tokens)?;

    let response = parse_book_preferences(message, &client).await?;

    let mut themes: HashMap<String, i16> = HashMap::new();
    for book in response.books.iter().chain(
        response
            .series
            .iter()
            .flat_map(|series| series.books.iter()),
    ) {
        println!("{:?}", book.title);
        for theme in book.themes.iter() {
            let value = themes.entry(theme.clone()).or_insert(0);
            *value += book.score as i16;
        }
    }

    let mut collected: Vec<(&String, &i16)> = themes.iter().collect();
    collected.sort_by_key(|&(_theme, score)| -score);
    for (theme, score) in collected.iter() {
        println!("{}: {}", score, theme)
    }

    let top_count = collected.len().min(5);
    let selected_themes: Vec<&str> = collected[..top_count]
        .iter()
        .map(|&(theme, _score)| theme.as_str())
        .collect();

    suggest_books(client, selected_themes).await?;

    Ok(())
}

async fn parse_book_preferences(
    message: String,
    client: &Client,
) -> Result<BookResponse, anyhow::Error> {
    let system = "\
    interpret the user input to collect the information described below.
    if only a title is provided and no author, attempt to identify the author yourself.
    if a series is mentioned, report all the books in the series.
    ";
    let messages = Messages::new().push_user(message).clone();
    send_with_json_retry::<BookResponse>(client, messages, Some(system.to_string()), 3).await
}

async fn suggest_books(
    client: Client,
    selected_themes: Vec<&str>,
) -> Result<RecommendationResponse, anyhow::Error> {
    let system = "\
    provide five recommended books for the given themes.
    ";
    let messages = Messages::new()
        .push_user(selected_themes.join(", "))
        .clone();

    send_with_json_retry::<RecommendationResponse>(&client, messages, Some(system.to_string()), 3)
        .await
}
