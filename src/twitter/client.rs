//! # Daegonica Module: twitter::client
//!
//! **Purpose:** Twitter API client with OAuth 1.0a authentication
//!
//! **Context:**
//! - Handles authenticated requests to Twitter API v2
//! - Used for posting tweets from the application
//!
//! **Responsibilities:**
//! - Authenticate with Twitter using OAuth 1.0a
//! - Post tweets via API
//! - Handle API errors gracefully
//! - Display success/failure messages
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use crate::prelude::*;
use crate::twitter::models::*;
use oauth1_request as oauth;

/// # TwitterConnection
///
/// **Summary:**
/// Client for interacting with the Twitter API v2 with OAuth 1.0a authentication.
///
/// **Fields:**
/// - `api_key`: Twitter API consumer key (from env)
/// - `api_secret`: Twitter API consumer secret (from env)
/// - `access_token`: User access token (from env)
/// - `access_token_secret`: User access token secret (from env)
/// - `client`: HTTP client for making requests
/// - `output`: Shared output handler for displaying results
///
/// **Usage Example:**
/// ```rust
/// let twitter = TwitterConnection::new(Arc::clone(&output));
/// twitter.post_tweet("Hello Twitter!").await?;
/// ```
pub struct TwitterConnection {
    api_key: String,
    api_secret: String,
    access_token: String,
    access_token_secret: String,
    client: Client,
    output: SharedOutput,
}
/// # EmptyRequest
///
/// **Summary:**
/// Empty request struct for OAuth signature generation (oauth1_request requirement).
#[derive(oauth::Request)]
struct EmptyRequest{

}

impl TwitterConnection {
    /// # new
    ///
    /// **Purpose:**
    /// Creates a new TwitterConnection with credentials from environment variables.
    ///
    /// **Parameters:**
    /// - `output`: Shared output handler for displaying messages
    ///
    /// **Returns:**
    /// Initialized TwitterConnection ready for API calls
    ///
    /// **Errors / Failures:**
    /// - Panics if required environment variables are not set:
    ///   - TWITTER_API_KEY
    ///   - TWITTER_API_SECRET
    ///   - TWITTER_ACCESS_TOKEN
    ///   - TWITTER_ACCESS_TOKEN_SECRET
    ///
    /// **Examples:**
    /// ```rust
    /// let twitter = TwitterConnection::new(Arc::clone(&output));
    /// ```
    pub fn new(output: SharedOutput) -> Self {
        dotenv().ok();
    
        let api_key = env::var("TWITTER_API_KEY")
            .expect("TWITTER_API_KEY not set in .env");
        let api_secret = env::var("TWITTER_API_SECRET")
            .expect("TWITTER_API_SECRET not set in .env");
        let access_token = env::var("TWITTER_ACCESS_TOKEN")
            .expect("TWITTER_ACCESS_TOKEN not set in .env");
        let access_token_secret = env::var("TWITTER_ACCESS_TOKEN_SECRET")
            .expect("TWITTER_ACCESS_TOKEN_SECRET not set in .env");
    
        TwitterConnection {
            api_key,
            api_secret,
            access_token,
            access_token_secret,
            client: Client::new(),
            output,
        }
    }

    /// # post_tweet
    ///
    /// **Purpose:**
    /// Posts a tweet to Twitter using the configured API credentials.
    ///
    /// **Parameters:**
    /// - `text`: The tweet content (max 280 characters)
    ///
    /// **Returns:**
    /// `Result<TweetData, Box<dyn std::error::Error>>` - Tweet data on success or error
    ///
    /// **Errors / Failures:**
    /// - Network connectivity issues
    /// - Authentication failures
    /// - Rate limiting
    /// - Tweet content violations (too long, duplicate, etc.)
    /// - API response parsing errors
    ///
    /// **Examples:**
    /// ```rust
    /// match twitter.post_tweet("Hello world!").await {
    ///     Ok(data) => println!("Posted: {}", data.id),
    ///     Err(e) => eprintln!("Failed: {}", e),
    /// }
    /// ```
    pub async fn post_tweet(&self, text: &str) -> Result<TweetData, Box<dyn std::error::Error>> {
        let url = "https://api.twitter.com/2/tweets";

        let body = CreateTweetRequest {
            text: text.to_string(),
        };

        let json_body = serde_json::to_string(&body)?;

        let token = oauth::Token::from_parts(
            &self.api_key,
            &self.api_secret,
            &self.access_token,
            &self.access_token_secret,
        );
    
        let empty_req = EmptyRequest{};
    
        let auth_header = oauth::post(url, &empty_req, &token, oauth::HMAC_SHA1);
    
        let response = self.client
            .post(url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .body(json_body)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;

        if status.is_success() {
            match serde_json::from_str::<TweetResponse>(&text) {
                Ok(tweet_response) => {
                    self.output.display(format!(
                        "✓ Tweet posted! ID: {} | Text: '{}'",
                        tweet_response.data.id,
                        tweet_response.data.text,
                    ));
                    Ok(tweet_response.data)
                }
                Err(e) => {
                    Err(format!("Failed to parse Twitter response: {}", e).into())
                }
            }
        } else {
            match serde_json::from_str::<TwitterErrorResponse>(&text) {
                Ok(error_body) => {
                    let error_msg = error_body.errors
                        .iter()
                        .map(|e| e.message.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    Err(format!("Twitter API Error: {}", error_msg).into())
                }
                Err(_) => {
                    Err(format!("Request failed ({}): {}", status, text).into())
                }
            }
        }
    }
}