# Shadow - The Ultimate Influencer Automation Hub

**Your AI Command Center for Content Creation & Multi-Platform Management**

Shadow is a powerful terminal-based application that unifies AI-powered content creation with platform automation. Built for content creators, streamers, and influencers who want to manage their entire digital presence from one place - no more juggling between dozens of apps.

## 🎯 Core Vision

Stop context-switching between OBS, Streamlabs, Twitter, Discord, ChatGPT, and a dozen other tools. Shadow brings everything together in one terminal interface where you can:
- Generate content with customizable AI personas
- Automate social media posting and trend tracking
- Control streaming platforms and chat moderation
- Manage your entire workflow without leaving the terminal

## ✨ Current Features (v0.1.0)

**AI Persona System**
- Multiple customizable personas for different content needs
- Real-time streaming responses from Grok AI
- Persistent conversation history with auto-summarization
- Context-aware multi-agent support

**Dual Interface Modes**
- **TUI Mode**: Full terminal interface with visual feedback (powered by ratatui)
- **CLI Mode**: Lightweight text interface for scripting and automation

**Smart History Management**
- Automatic conversation saving and loading
- Persona-specific history tracking
- Archive system for long conversations
- Timestamp-organized logging

**Architecture**
- Command Pattern for extensible features
- Modular design (client/conversation/history separation)
- Async streaming with Tokio
- Professional error handling with custom types

## 🚀 Coming Soon

**Twitter Integration** (Phase 2)
- Post tweets directly from Shadow
- Track trending topics in real-time
- Schedule content with AI assistance
- Monitor mentions and engagement

**Twitch Automation** (Phase 3)
- Integrated chatbot control
- Stream status monitoring
- Chat moderation via AI personas
- Stream management commands

**Stream Setup Automation** (Phase 4)
- OBS scene control via WebSocket
- One-command stream startup (title, game, alerts, scenes)
- Stream templates for different content types
- Full broadcast control from terminal

## 🛠️ Tech Stack

- **Language**: Rust 2024 Edition
- **AI**: Grok API (xAI) with streaming SSE
- **TUI**: ratatui 0.28 + crossterm
- **Async**: Tokio runtime with mpsc channels
- **Serialization**: serde + serde_json
- **Error Handling**: thiserror for custom types
- **Logging**: Custom dlog crate with timestamps

## 📊 Status

**Active Development** - v0.1.0 Foundation Complete  
Currently polishing for public release and building Twitter integration.


## 🎮 Usage

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/grokprime-brain.git
cd grokprime-brain

# Build and run
cargo run

# Or build for release
cargo build --release
./target/release/grokprime-brain
```

### Running Shadow

**TUI Mode (Default - Recommended)**
```bash
cargo run
# or explicitly
cargo run -- --tui
```

Interactive terminal interface with:
- Visual message history
- Multi-agent tabs
- Real-time streaming responses
- Easy persona switching

**CLI Mode (Automation & Scripting)**
```bash
cargo run -- --cli
```

Simple text-based interface for:
- Scripting automation
- Integration with other tools
- Lightweight resource usage


## ⌨️ Controls & Commands

### TUI Mode
- **Type & Enter**: Send message to active AI agent
- **Tab**: Switch between agents
- **Ctrl+N**: Create new agent
- **Ctrl+S**: Save conversation history
- **Ctrl+L**: Load history from file
- **ESC**: Exit application

### CLI Mode
- **Any text**: Chat with the AI
- **quit / exit**: Close application
- **save**: Save current conversation
- **new <persona>**: Start new conversation with persona

### Persona System

Personas are AI personalities with specific roles and behaviors. Define them in `personas/` directory as YAML files:

**Current Personas:**
- `shadow/` - Default technical assistant
- `friday/` - Friendly conversational AI
- `historian/` - Research and information specialist

**Example Persona Structure:**
```yaml
name: "ContentCreator"
role: "YouTube script writer"
temperature: 0.8
system_prompt: "You are an expert YouTube content creator..."
```


## ⚙️ Configuration

Create a `.env` file in the project root:

```env
# Required: Grok AI API
GROK_API_KEY=your_grok_api_key_here

# Optional: Twitter Integration (Phase 2)
CONSUMER_KEY=your_twitter_consumer_key
CONSUMER_SECRET=your_twitter_consumer_secret
ACCESS_TOKEN=your_twitter_access_token
ACCESS_TOKEN_SECRET=your_twitter_access_token_secret

# Optional: Twitch Integration (Phase 3)
TWITCH_CLIENT_ID=your_twitch_client_id
TWITCH_CLIENT_SECRET=your_twitch_client_secret

# Optional: OBS WebSocket (Phase 4)
OBS_WEBSOCKET_URL=ws://localhost:4455
OBS_WEBSOCKET_PASSWORD=your_obs_password
```

## 📁 Project Structure

```
grokprime-brain/
├── personas/           # AI persona YAML configurations
│   ├── shadow/
│   ├── friday/
│   └── historian/
├── history/            # Conversation history (persona-specific)
├── logs/              # Timestamped application logs
├── src/
│   ├── commands/      # Command Pattern implementations
│   ├── config.rs      # Centralized configuration
│   ├── error.rs       # Custom error types
│   ├── grok/          # Grok AI integration
│   │   ├── agent.rs       # Main coordinator
│   │   ├── client.rs      # HTTP/API layer
│   │   ├── conversation.rs # State management
│   │   └── history.rs     # File persistence
│   ├── persona/       # Persona loading and management
│   ├── tui/           # Terminal UI components
│   │   ├── app.rs         # Main TUI coordinator
│   │   ├── agent_pane.rs  # Agent display logic
│   │   └── widgets.rs     # Reusable UI components
│   ├── user/          # User input handling
│   ├── utilities/     # Helper functions
│   ├── lib.rs         # Library interface
│   ├── main.rs        # Entry point
│   └── prelude.rs     # Common imports
└── target/            # Build artifacts (gitignored)
```

## 🎓 Architecture Highlights

Shadow is built with professional software engineering patterns:

**Command Pattern**
- Each user action is a polymorphic `Command` trait object
- Easy to add new features without modifying core code
- Clean separation between TUI/CLI and business logic

**Single Responsibility Principle**
- Each module has one focused purpose
- `client.rs` = HTTP/API only
- `conversation.rs` = state management only
- `history.rs` = file persistence only

**Async Streaming Architecture**
- Tokio runtime with unbounded mpsc channels
- Server-Sent Events (SSE) for real-time AI responses
- Non-blocking I/O for responsive UI

**Centralized Configuration**
- Global lazy-static config with type safety
- No hardcoded values scattered in codebase
- Easy to modify behavior without code changes

## 🤝 Contributing

Shadow is being developed in public to help other content creators. Contributions welcome!

**Areas for Contribution:**
- New persona templates for specific content niches
- Additional platform integrations (YouTube, Discord, etc.)
- UI/UX improvements for TUI mode
- Documentation and tutorials
- Bug reports and feature requests

## 📝 License

MIT License - Use freely for personal or commercial projects

## 🔗 Links

- **GitHub**: [Shadow](https://github.com/Daegonica/grokprime-brain)
- **Issues**: [Report bugs or request features](https://github.com/Daegonica/grokprime-brain/issues)
- **Developer**: [Daegonica Software/X](https://twitter.com/Daegon89)

---

**Built with Rust 🦀 | Powered by Grok AI 🤖 | Designed for Creators 🎬**
