use crate::prelude::*;
use crate::twitter::models::*;
use oauth1_request as oauth;

pub struct TwitterConnection {
    api_key: String,
    api_secret: String,
    access_token: String,
    access_token_secret: String,
    client: Client,
    output: SharedOutput,
}
#[derive(oauth::Request)]
struct EmptyRequest{

}

impl TwitterConnection {
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