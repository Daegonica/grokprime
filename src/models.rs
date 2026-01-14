use serde::{Serialize, Deserialize};

// Response handling
#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}
#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub input: Vec<Message>,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
}


#[derive(Deserialize, Debug)]
pub struct ResponsesApiResponse{
    pub id: String,
    pub object: String,
    #[serde(rename = "created_at")]
    pub created_at: u64,
    pub model: String,
    pub output: Vec<OutputMessage>,
    #[serde(default)]
    pub usage: Option<Usage>,
}
#[derive(Deserialize, Debug)]
pub struct OutputMessage {
    pub id: String,
    pub role: String,
    #[serde(rename= "type")]
    pub type_: String,
    pub status: String,
    pub content: Vec<ContentBlock>,
}
#[derive(Deserialize, Debug)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub type_: String,
    pub text: String
}
#[derive(Deserialize, Debug)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}



// Error response from the API
#[derive(Deserialize, Debug)]
pub struct ApiErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub code: Option<String>,
}
#[derive(Deserialize, Debug)]
pub struct ApiErrorResponse {
    pub error: ApiErrorDetail,
}
#[derive(Debug)]
pub enum InputAction {
    Quit,
    DoNothing,
    ContinueNoSend(String),
    SendAsMessage(String),
}
