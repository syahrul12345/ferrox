use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct CompletionRequest {
    /// ID of the model to use
    pub model: String,
    /// A list of messages comprising the conversation so far
    pub messages: Vec<Message>,
    /// What sampling temperature to use, between 0 and 2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Tool choice - can be "none", "auto" or a specific tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    /// An alternative to sampling with temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// How many chat completion choices to generate for each input message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,
    /// Whether to stream back partial progress
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Up to 4 sequences where the API will stop generating further tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// The maximum number of tokens to generate in the chat completion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on
    /// whether they appear in the text so far
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their
    /// existing frequency in the text so far
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// Modify the likelihood of specified tokens appearing in the completion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<std::collections::HashMap<String, f32>>,
    /// A unique identifier representing your end-user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl Default for CompletionRequest {
    fn default() -> Self {
        Self {
            model: Model::OpenAI(OpenAIModel::GPT35Turbo).as_str().to_string(),
            messages: Vec::new(),
            temperature: None,
            tool_choice: Some("auto".to_string()),
            top_p: None,
            n: None,
            stream: None,
            stop: None,
            max_tokens: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: String,
    pub index: i32,
}

#[derive(Debug, Clone)]
pub enum Model {
    OpenAI(OpenAIModel),
    Anthropic(AnthropicModel),
}

#[derive(Debug, Clone)]
pub enum OpenAIModel {
    GPT4,
    GPT4Turbo,
    GPT4Mini,
    GPT4RealTimePreview,
    GPT40,
    GPT35Turbo,
}

#[derive(Debug, Clone)]
pub enum AnthropicModel {
    Claude3Opus,
    Claude3Sonnet,
}

impl Model {
    pub fn as_str(&self) -> &'static str {
        match self {
            Model::OpenAI(model) => model.as_str(),
            Model::Anthropic(model) => model.as_str(),
        }
    }
}

impl OpenAIModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            OpenAIModel::GPT4 => "gpt-4",
            OpenAIModel::GPT4Turbo => "gpt-4-turbo",
            OpenAIModel::GPT4Mini => "gpt-4-mini",
            OpenAIModel::GPT4RealTimePreview => "gpt-4-realtime-preview",
            OpenAIModel::GPT40 => "gpt-4o",
            OpenAIModel::GPT35Turbo => "gpt-3.5-turbo",
        }
    }
}

impl AnthropicModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnthropicModel::Claude3Opus => "claude-3-opus",
            AnthropicModel::Claude3Sonnet => "claude-3-sonnet",
        }
    }
}
