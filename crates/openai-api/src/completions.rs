use crate::models::{CompletionRequest, CompletionResponse, Message, Model};
use anyhow::Result;

pub struct Client {
    api_key: String,
    model: Model,
    client: reqwest::Client,
    base_url: Option<String>,
}

impl Client {
    pub fn new(api_key: String, model: Model) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
            base_url: None,
        }
    }

    pub fn with_model(mut self, model: Model) -> Self {
        self.model = model;
        self
    }

    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    fn get_base_url(&self) -> String {
        if let Some(url) = &self.base_url {
            url.clone()
        } else {
            match self.model {
                Model::OpenAI(_) => "https://api.openai.com".to_string(),
                Model::Anthropic(_) => "https://api.anthropic.com".to_string(),
            }
        }
    }

    pub async fn send_prompt(&self, prompt: String, mut history: Vec<Message>) -> Result<String> {
        // Add the user's prompt to the message history
        history.push(Message {
            role: "user".to_string(),
            content: prompt,
        });

        let request = CompletionRequest {
            model: self.model.as_str().to_string(),
            messages: history,
            temperature: Some(0.7),
            tool_choice: Some("auto".to_string()),
            ..Default::default()
        };

        let endpoint = match self.model {
            Model::OpenAI(_) => "/v1/chat/completions",
            Model::Anthropic(_) => "/v1/messages",
        };

        let response = self
            .client
            .post(format!("{}{}", self.get_base_url(), endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header(
                "anthropic-version",
                if matches!(self.model, Model::Anthropic(_)) {
                    "2023-06-01"
                } else {
                    ""
                },
            )
            .json(&request)
            .send()
            .await?;

        let completion: CompletionResponse = response.json().await?;

        Ok(completion
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AnthropicModel, OpenAIModel};
    use mockito;
    use serde_json::json;

    fn setup_mock_response() -> mockito::Mock {
        let mut mock = mockito::Server::new();
        mock.mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": "chatcmpl-123",
                    "object": "chat.completion",
                    "created": 1677652288,
                    "choices": [{
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": "Hello! How can I help you today?"
                        },
                        "finish_reason": "stop"
                    }]
                })
                .to_string(),
            )
            .create()
    }

    #[tokio::test]
    async fn test_new_client() {
        let client = Client::new(
            "test-key".to_string(),
            Model::OpenAI(OpenAIModel::GPT35Turbo),
        );
        assert_eq!(client.api_key, "test-key");
        assert!(matches!(
            client.model,
            Model::OpenAI(OpenAIModel::GPT35Turbo)
        ));
    }

    #[tokio::test]
    async fn test_with_model() {
        let client = Client::new(
            "test-key".to_string(),
            Model::OpenAI(OpenAIModel::GPT35Turbo),
        )
        .with_model(Model::Anthropic(AnthropicModel::Claude3Sonnet));

        assert!(matches!(
            client.model,
            Model::Anthropic(AnthropicModel::Claude3Sonnet)
        ));
    }

    #[tokio::test]
    async fn test_send_prompt() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": "chatcmpl-123",
                    "object": "chat.completion",
                    "created": 1677652288,
                    "choices": [{
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": "Hello! How can I help you today?"
                        },
                        "finish_reason": "stop"
                    }]
                })
                .to_string(),
            )
            .create();

        let client = Client::new(
            "test-key".to_string(),
            Model::OpenAI(OpenAIModel::GPT35Turbo),
        )
        .with_base_url(url);

        let history = vec![Message {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        }];

        let result = client
            .send_prompt("Hello!".to_string(), history)
            .await
            .unwrap();

        assert_eq!(result, "Hello! How can I help you today?");
        mock.assert();
    }

    #[tokio::test]
    async fn test_model_string_conversion() {
        assert_eq!(Model::OpenAI(OpenAIModel::GPT4).as_str(), "gpt-4");
        assert_eq!(
            Model::OpenAI(OpenAIModel::GPT35Turbo).as_str(),
            "gpt-3.5-turbo"
        );
        assert_eq!(
            Model::Anthropic(AnthropicModel::Claude3Sonnet).as_str(),
            "claude-3-sonnet"
        );
    }

    #[tokio::test]
    async fn test_base_url_selection() {
        let openai_client = Client::new(
            "test-key".to_string(),
            Model::OpenAI(OpenAIModel::GPT35Turbo),
        );
        assert_eq!(openai_client.get_base_url(), "https://api.openai.com");

        let anthropic_client = Client::new(
            "test-key".to_string(),
            Model::Anthropic(AnthropicModel::Claude3Sonnet),
        );
        assert_eq!(anthropic_client.get_base_url(), "https://api.anthropic.com");
    }
}
