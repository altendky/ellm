# ellm

A Rust CLI client and library for interacting with Claude AI and other LLMs via the Anthropic API.

## Features

- Library crate for embedding Claude API access in your Rust applications
- Command-line interface for quick interactions with Claude
- Support for multiple API key configuration methods
- Async-first design using tokio and reqwest

## Installation

```bash
cargo install ellm
```

Or build from source:

```bash
git clone https://github.com/altendky/ellm
cd ellm
cargo build --release
```

## Development

### Prerequisites

- Rust 1.91.0 or later (automatically configured via `rust-toolchain.toml`)
- dprint (`cargo install dprint`)
- [pre-commit](https://pre-commit.com/) for git hooks (optional but recommended)

### Setup

Install pre-commit hooks:

```bash
# Install pre-commit (if not already installed)
pipx install pre-commit

# Install the git hooks
pre-commit install
```

The pre-commit hooks will automatically run:

- `cargo fmt` - Format Rust code
- `cargo check` - Check for compilation errors
- `cargo clippy` - Lint Rust code
- `dprint` - Format YAML, TOML, Markdown, and JSON files
- `shfmt` - Format shell scripts
- `actionlint` - Lint GitHub Actions workflows
- Various file checks (YAML, TOML, trailing whitespace, etc.)

You can also run the hooks manually:

```bash
pre-commit run --all-files
```

### Running Tests

```bash
cargo test
```

## Configuration

The API key can be provided in three ways (in order of precedence):

1. Command-line argument: `--api-key YOUR_KEY`
2. Environment variable: `ANTHROPIC_API_KEY`
3. Configuration file: `~/.config/ellm/config.toml`

Example config file:

```toml
api_key = "your-api-key-here"
```

## Usage

### CLI

Send a message to Claude:

```bash
cargo run --bin ellm -- send "Hello, Claude!"
```

Specify API key directly:

```bash
cargo run --bin ellm -- --api-key YOUR_KEY send "Hello, Claude!"
```

### Library

```rust
use ellm::{Client, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let client = Client::new(config)?;

    let response = client.send_message("Hello, Claude!").await?;
    println!("Response: {}", response);

    Ok(())
}
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
