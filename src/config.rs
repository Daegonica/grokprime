//! # Daegonica Module: config
//!
//! **Purpose:** Application configuration and settings
//!
//! **Context:**
//! - Centralizes all hardcoded values and magic numbers
//! - Provides single source of truth for application settings
//! - Makes it easy to adjust behavior without code changes
//!
//! **Responsibilities:**
//! - Define configuration structures for each subsystem
//! - Provide sensible defaults for all settings
//! - (Future) Load from config file or environment variables
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-20
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use ratatui::style::Color;


/// # AppConfig
///
/// **Summary:**
/// Top-level application configuration containing all subsystem configs.
///
/// **Fields:**
/// - `grok`: Configuration for Grok API client
/// - `tui`: Configuration for terminal user interface
/// - `history`: Configuration for conversation history management
///
/// **Usage Example:**
/// ```rust
/// let config = AppConfig::default();
/// println!("Using model: {}", config.grok.model_name);
/// ```
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub grok: GrokConfig,
    pub tui: TuiConfig,
    pub history: HistoryConfig,
}

/// # GrokConfig
///
/// **Summary:**
/// Configuration for Grok API interactions.
///
/// **Fields:**
/// - `model_name`: The Grok model to use (e.g., "grok-4-fast")
/// - `default_temperature`: Default randomness for responses (0.0-1.0)
/// - `stream_enabled`: Whether to use streaming responses
///
/// **Usage Example:**
/// ```rust
/// let grok_config = GrokConfig::default();
/// let request = ChatRequest {
///     model: grok_config.model_name.clone(),
///     temperature: grok_config.default_temperature,
///     stream: grok_config.stream_enabled,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct GrokConfig {
    pub model_name: String,
    pub default_temperature: f32,
    pub stream_enabled: bool,
}

/// # TuiConfig
///
/// **Summary:**
/// Configuration for terminal user interface appearance and behavior.
///
/// **Fields:**
/// - `max_history_size`: Maximum messages to keep in memory
/// - `max_input_lines`: Maximum visible lines in input box
/// - `border_color`: RGB color for UI borders
/// - `user_message_color`: RGB color for user messages
/// - `scroll_step`: Lines to scroll per arrow key press
/// - `page_scroll_step`: Lines to scroll per page up/down
///
/// **Usage Example:**
/// ```rust
/// let tui_config = TuiConfig::default();
/// let border_style = Style::default().fg(tui_config.border_color);
/// ```
#[derive(Debug, Clone)]
pub struct TuiConfig {
    pub max_history_size: usize,
    pub max_input_lines: u16,
    pub border_color: Color,
    pub user_message_color: Color,
    pub scroll_step: u16,
    pub page_scroll_step: u16,
}

/// # HistoryConfig
///
/// **Summary:**
/// Configuration for conversation history persistence and management.
///
/// **Fields:**
/// - `enabled`: Whether to save/load history
/// - `auto_save`: Whether to save after each message
/// - `max_messages_before_summary`: Trigger summarization threshold
/// - `messages_to_keep_after_summary`: How many recent messages to keep
///
/// **Usage Example:**
/// ```rust
/// let history_config = HistoryConfig::default();
/// if history_config.enabled {
///     save_history_to_disk()?;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct HistoryConfig {
    pub enabled: bool,
    pub auto_save: bool,
    pub max_messages_before_summary: usize,
    pub messages_to_keep_after_summary: usize,
}

impl Default for GrokConfig {
    fn default() -> Self {
        Self {
            model_name: "grok-4-fast".to_string(),
            default_temperature: 0.7,
            stream_enabled: true,
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            max_history_size: 1000,
            max_input_lines: 20,
            border_color: Color::Rgb(255, 140, 0),
            user_message_color: Color::LightYellow,
            scroll_step: 1,
            page_scroll_step: 10,
        }
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_save: true,
            max_messages_before_summary: 20,
            messages_to_keep_after_summary: 12,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            grok: GrokConfig::default(),
            tui: TuiConfig::default(),
            history: HistoryConfig::default(),
        }
    }
}

use once_cell::sync::Lazy;

/// # GLOBAL_CONFIG
///
/// **Summary:**
/// Global application configuration singleton.
///
/// **Usage:**
/// ```rust
/// use crate::config::GLOBAL_CONFIG;
/// 
/// let model = &GLOBAL_CONFIG.grok.model_name;
/// ```
///
/// **Note:**
/// This is initialized once at program startup and is thread-safe.
/// Future versions may support runtime reloading from config file.
pub static GLOBAL_CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    AppConfig::default()
});