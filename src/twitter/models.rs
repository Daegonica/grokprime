//! # Daegonica Module: twitter::models
//!
//! **Purpose:** Data structures for Twitter API requests and responses
//!
//! **Context:**
//! - Models for tweet creation and response handling
//! - Error response structures for API failures
//!
//! **Responsibilities:**
//! - Define serializable structures for Twitter API
//! - Handle tweet data and error responses
//! - Does NOT contain business logic (pure data structures)
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use serde::{Deserialize, Serialize};

/// # CreateTweetRequest
///
/// **Summary:**
/// Request payload for posting a new tweet to Twitter.
///
/// **Fields:**
/// - `text`: The tweet content (max 280 characters)
///
/// **Usage Example:**
/// ```rust
/// let request = CreateTweetRequest {
///     text: "Hello Twitter!".to_string(),
/// };
/// ```
#[derive(Serialize, Debug)]
pub struct CreateTweetRequest {
    pub text: String,
}

/// # TweetResponse
///
/// **Summary:**
/// Successful response from Twitter API after posting a tweet.
///
/// **Fields:**
/// - `data`: The tweet data returned by the API
///
/// **Usage Example:**
/// ```rust
/// let response: TweetResponse = serde_json::from_str(&json)?;
/// println!("Posted: {}", response.data.text);
/// ```
#[derive(Deserialize, Debug)]
pub struct TweetResponse {
    pub data: TweetData,
}

/// # TweetData
///
/// **Summary:**
/// Individual tweet data with ID and content.
///
/// **Fields:**
/// - `id`: Unique Twitter ID for the tweet
/// - `text`: The actual tweet content
///
/// **Usage Example:**
/// ```rust
/// println!("Tweet ID: {}", tweet_data.id);
/// ```
#[derive(Deserialize, Debug)]
pub struct TweetData {
    pub id: String,
    pub text: String,
}

/// # TwitterErrorResponse
///
/// **Summary:**
/// Error response from Twitter API when requests fail.
///
/// **Fields:**
/// - `errors`: Vector of individual error details
///
/// **Usage Example:**
/// ```rust
/// let error_response: TwitterErrorResponse = serde_json::from_str(&json)?;
/// for error in error_response.errors {
///     println!("Error: {}", error.message);
/// }
/// ```
#[derive(Deserialize, Debug)]
pub struct TwitterErrorResponse {
    pub errors: Vec<TwitterError>,
}

/// # TwitterError
///
/// **Summary:**
/// Individual error detail from Twitter API.
///
/// **Fields:**
/// - `message`: Human-readable error message
/// - `title`: Optional error title
///
/// **Usage Example:**
/// ```rust
/// println!("Twitter error: {}", error.message);
/// ```
#[derive(Deserialize, Debug)]
pub struct TwitterError {
    pub message: String,
    #[serde(default)]
    pub title: String,
}

