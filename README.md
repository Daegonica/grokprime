# Daegonica Software GrokPrime-Brain (Shadow)

An AI-powered conversational assistant with multiple persona support. This CLI/TUI application interfaces with Grok AI and includes Twitter integration for automated social media interaction.

## Features
- Multiple AI personas (Shadow, Friday, Historian)
- TUI mode with full terminal interface powered by ratatui
- CLI mode for scripting and automation
- Conversation history management
- Twitter integration for posting and drafting tweets
- Real-time streaming responses from Grok AI
- Persistent conversation storage
- Multi-agent support with independent contexts

## Tech
- Rust
- Grok API (xAI)
- Twitter API (OAuth 1.0)
- ratatui for TUI interface
- crossterm for terminal handling
- tokio for async runtime
- clap for CLI argument parsing

## Status
Active Development

## Modes

### TUI Mode (Default)
Interactive terminal interface with visual feedback and message history.
```bash
cargo run
# or explicitly
cargo run -- --tui
```

### CLI Mode
Simple text-based interface for automation and scripting.
```bash
cargo run -- --cli
```

## Commands (CLI Mode)

### Chat Commands
- Type any message to chat with the AI
- "quit" or "exit" - Exit the application

### Twitter Integration
- "tweet <text>"
    - Post a tweet directly to Twitter
- "draft <idea>"
    - Generate a tweet from an idea using AI

## TUI Controls
- Type your message and press Enter to send
- ESC - Exit application
- Navigate through conversation history

## Configuration
Requires a `.env` file with:
```
GROK_API_KEY=your_api_key_here
CONSUMER_KEY=your_twitter_consumer_key
CONSUMER_SECRET=your_twitter_consumer_secret
ACCESS_TOKEN=your_twitter_access_token
ACCESS_TOKEN_SECRET=your_twitter_access_token_secret
```

## Personas
Personas are defined in YAML files in the `personas/` directory:
- `shadow/` - Default AI persona
- `friday/` - Alternative persona
- `historian/` - Specialized persona

## How to Run
```bash
# TUI mode (default)
cargo run

# CLI mode
cargo run -- --cli

# Build for release
cargo build --release
```

## Project Structure
```
grokprime-brain/
├── personas/          # AI persona configurations
├── history/           # Conversation history storage
├── src/
│   ├── grok/         # Grok API integration
│   ├── twitter/      # Twitter API integration
│   ├── tui/          # Terminal UI components
│   ├── persona/      # Persona management
│   ├── user/         # User input handling
│   └── utilities/    # Helper functions
└── target/           # Build artifacts
```
