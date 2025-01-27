use crate::models::{CompletionRequest, CompletionResponse, Message, Model, Tool};
use anyhow::Result;
use serde::Serialize;
use serde_json::json;

#[derive(Clone)]
pub struct Client {
    api_key: String,
    model: Model,
    client: reqwest::Client,
    base_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StructuredResponse {
    pub tool_call: bool,
    pub content: String,
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

    pub async fn send_prompt_with_tools(
        &self,
        prompt: Option<String>,
        mut history: Vec<Message>,
        mut tools: Vec<Tool>,
    ) -> Result<StructuredResponse> {
        println!("Sending prompt with tools");
        // Add the user's prompt to the message history
        if let Some(prompt) = prompt {
            history.push(Message {
                role: "user".to_string(),
                content: Some(prompt),
                tool_calls: None,
                tool_call_id: None,
            });
        }

        // Process array parameters in tools
        for tool in &mut tools {
            if let Some(properties) = tool.function.parameters.get_mut("properties") {
                if let Some(obj) = properties.as_object_mut() {
                    for (_, value) in obj.iter_mut() {
                        if let Some(param_obj) = value.as_object_mut() {
                            if param_obj.get("type").and_then(|t| t.as_str()) == Some("array") {
                                // Add items field for array type if not present
                                if !param_obj.contains_key("items") {
                                    param_obj.insert(
                                        "items".to_string(),
                                        json!({
                                            "type": "string"  // Default to string array
                                        }),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        let request = CompletionRequest {
            model: self.model.as_str().to_string(),
            messages: history,
            temperature: Some(0.7),
            tool_choice: match tools.is_empty() {
                true => None,
                false => Some("auto".to_string()),
            },
            parallel_tool_calls: match tools.is_empty() {
                true => None,
                false => Some(true),
            },
            tools: match tools.is_empty() {
                true => None,
                false => Some(tools),
            },
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

        let text = response.text().await?;
        let completion: CompletionResponse = serde_json::from_str(&text)?;
        // Handle both regular responses and tool calls
        let first_choice = completion
            .choices
            .first()
            .ok_or_else(|| anyhow::anyhow!("No completion choices returned from the API"))?;

        match &first_choice.message.tool_calls {
            Some(tool_calls) if !tool_calls.is_empty() => Ok(StructuredResponse {
                tool_call: true,
                content: serde_json::to_string(&tool_calls)?,
            }),
            _ => Ok(StructuredResponse {
                tool_call: false,
                content: first_choice
                    .message
                    .content
                    .as_ref()
                    .unwrap_or(&"".to_string())
                    .clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AnthropicModel, FunctionDefinition, OpenAIModel};
    use mockito;
    use serde_json::json;

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
    async fn test_send_prompt_with_tools() {
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
                            "content": "Hello! How can I help you today?",
                            "tool_calls": null
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
            content: Some("You are a helpful assistant.".to_string()),
            tool_calls: None,
            tool_call_id: None,
        }];

        let tools = vec![]; // Empty tools for this test

        let result = client
            .send_prompt_with_tools(Some("Hello!".to_string()), history, tools)
            .await
            .unwrap();

        assert_eq!(result.content, "Hello! How can I help you today?");
        mock.assert();
    }

    #[tokio::test]
    async fn test_send_prompt_with_tool_call_response() {
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
                            "content": null,
                            "tool_calls": [{
                                "id": "call_123",
                                "function": {
                                    "name": "calculator",
                                    "arguments": "{\"a\":5,\"b\":3,\"operation\":\"add\"}"
                                }
                            }]
                        },
                        "finish_reason": "tool_calls"
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
            content: Some("You are a helpful assistant.".to_string()),
            tool_calls: None,
            tool_call_id: None,
        }];

        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "calculator".to_string(),
                description: "Calculate two numbers".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number"},
                        "b": {"type": "number"},
                        "operation": {"type": "string"}
                    },
                    "required": ["a", "b", "operation"]
                }),
            },
        }];

        let result = client
            .send_prompt_with_tools(Some("Calculate 5 plus 3".to_string()), history, tools)
            .await
            .unwrap();

        // The result should be the JSON string of the tool calls
        assert!(result.content.contains("calculator"));
        assert!(result.content.contains("add"));
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
