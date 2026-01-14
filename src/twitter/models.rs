use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct CreateTweetRequest {
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct TweetResponse {
    pub data: TweetData,
}

#[derive(Deserialize, Debug)]
pub struct TweetData {
    pub id: String,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct TwitterErrorResponse {
    pub errors: Vec<TwitterError>,
}

#[derive(Deserialize, Debug)]
pub struct TwitterError {
    pub message: String,
    #[serde(default)]
    pub title: String,
}

