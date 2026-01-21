//! # Daegonica Module: error
//!
//! **Purpose:** Custom error types for GrokPrime-Brain
//!
//! **Context:**
//! - Provides type-safe error handling throughout the application
//! - Uses thiserror crate for automatic Display and Error trait implementations
//! - Replaces generic Box<dyn Error> with specific error variants
//!
//! **Responsibilities:**
//! - Define all possible error types in the application
//! - Provide clear error messages with context
//! - Enable pattern matching on specific error cases
//! - Automatic conversion from common error types
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-20
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use thiserror::Error;

/// # ShadowError
///
/// **Summary:**
/// Main error type for GrokPrime-Brain application.
///
/// **Variants:**
/// - API errors (network, authentication, rate limiting)
/// - File I/O errors (missing files, permission issues)
/// - Configuration errors (missing env vars, invalid settings)
/// - Parsing errors (JSON, YAML)
/// - Application logic errors
///
/// **Usage Example:**
/// ```rust
/// fn do_something() -> Result<(), ShadowError> {
///     let config = load_config()
///         .map_err(|e| ShadowError::ConfigError(format!("Failed to load: {}", e)))?;
///     Ok(())
/// }
/// ```
#[derive(Error, Debug)]
pub enum ShadowError {
    // API Errors
    #[error("Grok API error: {0}")]
    ApiError(String),
    
    #[error("API authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("API rate limit exceeded")]
    RateLimitError,
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    // File I/O Errors
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Permission denied accessing: {0}")]
    PermissionDenied(String),
    
    #[error("I/O error: {0}")]
    IoError(String),
    
    // Parsing Errors
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),
    
    #[error("Invalid YAML: {0}")]
    InvalidYaml(String),
    
    // Configuration Errors
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    // History/Persona Errors
    #[error("Persona not found: {0}")]
    PersonaNotFound(String),
    
    #[error("History corruption detected: {0}")]
    CorruptedHistory(String),
    
    #[error("Summarization failed: {0}")]
    SummarizationError(String),
    
    // Application Logic Errors
    #[error("No active agent")]
    NoActiveAgent,
    
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    // Channel/Async Errors
    #[error("Channel send failed")]
    ChannelSendError,
    
    #[error("Channel receive failed")]
    ChannelRecvError,
    
    // Generic fallback
    #[error("Unknown error: {0}")]
    Unknown(String),

}

impl From<std::io::Error> for ShadowError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => ShadowError::FileNotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => ShadowError::PermissionDenied(err.to_string()),
            _ => ShadowError::IoError(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for ShadowError {
    fn from(err: serde_json::Error) -> Self {
        ShadowError::InvalidJson(err.to_string())
    }
}

impl From<reqwest::Error> for ShadowError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ShadowError::NetworkError("Request timeout".to_string())
        } else if err.is_connect() {
            ShadowError::NetworkError("Connection failed".to_string())
        } else {
            ShadowError::NetworkError(err.to_string())
        }
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for ShadowError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        ShadowError::ChannelSendError
    }
}