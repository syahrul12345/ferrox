use super::Agent;
use crate::action::Action;
use openai_api::{
    completions::Client as OpenAIClient,
    models::{FunctionDefinition, Message, Model, Tool},
};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

pub struct TextAgent<T>
where
    T: Agent,
{
    pub inner_agent: Option<T>,
    pub system_prompt: String,
    pub open_ai_client: OpenAIClient,
    conversation_history: Arc<Mutex<HashMap<String, Vec<Message>>>>,
    actions: Arc<Mutex<Vec<Box<dyn Action>>>>,
}

impl<T: Agent> TextAgent<T> {
    pub fn new(
        inner_agent: Option<T>,
        system_prompt: String,
        api_key: String,
        model: Model,
    ) -> Self {
        Self {
            inner_agent,
            system_prompt,
            open_ai_client: OpenAIClient::new(api_key, model),
            conversation_history: Arc::new(Mutex::new(HashMap::new())),
            actions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn send_prompt(
        &self,
        prompt: &str,
        history_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + 'static>> {
        // Clone what we need for the async block
        let conversation_history = self.conversation_history.clone();
        let system_prompt = self.system_prompt.clone();
        let open_ai_client = self.open_ai_client.clone();
        let actions = self.actions.clone();
        let history_id = history_id.to_string();
        let prompt = prompt.to_string();

        // Convert actions to OpenAI tools
        let tools: Vec<Tool> = self
            .actions
            .lock()
            .unwrap()
            .iter()
            .map(|action| {
                let definition = action.definition();
                Tool {
                    tool_type: "function".to_string(),
                    function: FunctionDefinition {
                        name: definition.name,
                        description: definition.description,
                        parameters: serde_json::json!({
                            "type": "object",
                            "properties": definition.parameters.clone().into_iter().map(|param| {
                                (param.name, serde_json::json!({
                                    "type": param.param_type,
                                    "description": param.description,
                                }))
                            }).collect::<serde_json::Map<String, serde_json::Value>>(),
                            "required": definition.parameters.clone().into_iter()
                                .filter(|p| p.required)
                                .map(|p| p.name.clone())
                                .collect::<Vec<String>>(),
                            "additionalProperties": false,
                        }),
                    },
                }
            })
            .collect();

        Box::pin(async move {
            let history = {
                let mut history_map = conversation_history.lock().map_err(|e| e.to_string())?;

                if let Some(existing_history) = history_map.get(&history_id) {
                    existing_history.clone()
                } else {
                    let mut new_history = Vec::new();
                    new_history.push(Message {
                        role: "system".to_string(),
                        content: Some(system_prompt),
                        tool_calls: None,
                    });
                    history_map.insert(history_id.clone(), new_history.clone());
                    new_history
                }
            };

            let response = open_ai_client
                .send_prompt_with_tools(prompt, history.clone(), tools)
                .await
                .map_err(|e| e.to_string())?;

            let final_response = if response.tool_call {
                let tool_calls: Vec<openai_api::models::ToolCall> =
                    serde_json::from_str(&response.content).map_err(|e| e.to_string())?;

                let mut results = Vec::new();

                for tool_call in tool_calls {
                    if let Some(action) = actions
                        .lock()
                        .unwrap()
                        .iter()
                        .find(|a| a.definition().name == tool_call.function.name)
                    {
                        let result = action
                            .execute(
                                serde_json::from_str(&tool_call.function.arguments)
                                    .map_err(|e| e.to_string())?,
                            )
                            .await
                            .map_err(|e| {
                                format!("Failed to execute {}: {}", tool_call.function.name, e)
                            })?;
                        results.push(result);
                    }
                }
                results.join("\n")
            } else {
                response.content
            };

            // Update history with response
            {
                let mut history_map = conversation_history.lock().map_err(|e| e.to_string())?;
                if let Some(conversation) = history_map.get_mut(&history_id) {
                    conversation.extend(history.iter().skip(conversation.len()).cloned());
                    conversation.push(Message {
                        role: "assistant".to_string(),
                        content: Some(final_response.clone()),
                        tool_calls: None,
                    });
                }
            }

            Ok(final_response)
        })
    }
}

impl<T: Agent> Agent for TextAgent<T> {
    fn add_action(&mut self, action: Box<dyn Action>) {
        self.actions.lock().unwrap().push(action);
    }

    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    fn process_prompt(
        &self,
        prompt: &str,
        history_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>>>> {
        self.send_prompt(prompt, history_id)
    }
}

//For these tests make sure to set the OPENAI_API_KEY environment variable
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{action::ActionBuilder, agent::NullAgent};
    use openai_api::models::OpenAIModel;
    use serde::Deserialize;
    use std::env;

    #[tokio::test]
    async fn test_text_agent_with_actions() {
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

        let mut agent = TextAgent::<NullAgent>::new(
            None,
            "You are a helpful assistant that can perform calculations, generate greetings, and reverse text. \
             Please use the appropriate action when needed.".to_string(),
            api_key,
            Model::OpenAI(OpenAIModel::GPT35Turbo),
        );

        println!("Created text agent");

        {
            #[derive(Deserialize, Debug)]
            struct CalcParams {
                a: f64,
                b: f64,
                operation: String,
            }
            async fn calculator(params: CalcParams) -> Result<String, String> {
                println!("Calculator called with params: {:?}", params);
                let result = match params.operation.as_str() {
                    "add" => params.a + params.b,
                    "subtract" => params.a - params.b,
                    "multiply" => params.a * params.b,
                    "divide" => {
                        if params.b == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        params.a / params.b
                    }
                    _ => return Err("Invalid operation".to_string()),
                };
                Ok(result.to_string())
            }
            let calc_action = ActionBuilder::<_, CalcParams>::new("calculator", calculator)
                .description("Perform basic arithmetic operations")
                .parameter("a", "First number", "number", true)
                .parameter("b", "Second number", "number", true)
                .parameter(
                    "operation",
                    "Operation to perform (add/subtract/multiply/divide)",
                    "string",
                    true,
                )
                .build();
            agent.add_action(Box::new(calc_action));
            println!("Added calculator action");
        }

        {
            #[derive(Deserialize, Debug)]
            struct GreetParams {
                name: String,
                language: Option<String>,
            }
            async fn greeter(params: GreetParams) -> Result<String, String> {
                println!("Greeter called with params: {:?}", params);
                let greeting = match params.language.as_deref() {
                    Some("es") => "¡Hola",
                    Some("fr") => "Bonjour",
                    _ => "Hello",
                };
                Ok(format!("{} {}!", greeting, params.name))
            }
            let greet_action = ActionBuilder::<_, GreetParams>::new("greeter", greeter)
                .description("Generate a greeting message")
                .parameter("name", "Name to greet", "string", true)
                .parameter("language", "Language code (en/es/fr)", "string", false)
                .build();
            agent.add_action(Box::new(greet_action));
            println!("Added greeter action");
        }

        {
            #[derive(Deserialize, Debug)]
            struct ReverseParams {
                text: String,
            }

            async fn reverser(params: ReverseParams) -> Result<String, String> {
                println!("Reverser called with params: {:?}", params);
                Ok(params.text.chars().rev().collect())
            }
            let reverse_action = ActionBuilder::<_, ReverseParams>::new("reverser", reverser)
                .description("Reverse input text")
                .parameter("text", "Text to reverse", "string", true)
                .build();
            agent.add_action(Box::new(reverse_action));
            println!("Added reverser action");
        }
        // Text reverser action

        // Test individual actions through the agent
        let calc_prompt = "Calculate 5 plus 3";
        println!("Testing calculator with prompt: {}", calc_prompt);
        let calc_response = agent.process_prompt(calc_prompt, "test1").await.unwrap();
        println!("Calculator response: {}", calc_response);

        let greet_prompt = "Say hello to Alice in Spanish";
        println!("Testing greeter with prompt: {}", greet_prompt);
        let greet_response = agent.process_prompt(greet_prompt, "test2").await.unwrap();
        println!("Greeter response: {}", greet_response);

        let reverse_prompt = "Reverse the text 'hello world'";
        println!("Testing reverser with prompt: {}", reverse_prompt);
        let reverse_response = agent.process_prompt(reverse_prompt, "test3").await.unwrap();
        println!("Reverser response: {}", reverse_response);

        // Test chained actions
        let chained_prompt = "Calculate 10 plus 5, then greet the result in Spanish, and finally reverse that greeting";
        println!("Testing chained actions with prompt: {}", chained_prompt);
        let chained_response = agent.process_prompt(chained_prompt, "test4").await.unwrap();
        println!("Chained actions response: {}", chained_response);
    }

    // Keep the existing conversation tests
    #[tokio::test]
    async fn test_text_agent_conversation() {
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

        let agent = TextAgent::<NullAgent>::new(
            None,
            "You are a helpful assistant that provides concise responses.".to_string(),
            api_key,
            Model::OpenAI(OpenAIModel::GPT35Turbo),
        );

        // Test first message
        let response = agent
            .process_prompt("What is Rust programming language?", "default")
            .await
            .expect("Failed to get response");

        println!("First response: {}", response);
        assert!(!response.is_empty());

        // Test follow-up question (should maintain context)
        let response = agent
            .process_prompt("What are its main features?", "default")
            .await
            .expect("Failed to get response");

        println!("Follow-up response: {}", response);
        assert!(!response.is_empty());

        // Verify conversation history
        let history = agent.conversation_history.lock().unwrap();
        let default_history = history
            .get("default")
            .expect("No conversation history found");

        assert_eq!(default_history[0].role, "system");
        assert_eq!(default_history[1].role, "user");
        assert_eq!(default_history[2].role, "assistant");
        assert_eq!(default_history[3].role, "user");
        assert_eq!(default_history[4].role, "assistant");
    }

    #[tokio::test]
    async fn test_text_agent_multiple_conversations() {
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

        let agent = TextAgent::<NullAgent>::new(
            None,
            "You are a helpful assistant.".to_string(),
            api_key,
            Model::OpenAI(OpenAIModel::GPT35Turbo),
        );

        // Test sending prompts with different conversation IDs
        let send_prompt = |id: &str, prompt: &str| {
            let agent = &agent;
            let id = id.to_string();
            let prompt = prompt.to_string();
            async move {
                agent
                    .process_prompt(&prompt, &id)
                    .await
                    .expect("Failed to get response")
            }
        };

        let (response1, response2) = tokio::join!(
            send_prompt("conv1", "Tell me about Python"),
            send_prompt("conv2", "Tell me about JavaScript")
        );

        println!("Python response: {}", response1);
        println!("JavaScript response: {}", response2);

        // Verify separate conversation histories
        let history = agent.conversation_history.lock().unwrap();

        let conv1 = history
            .get("conv1")
            .expect("No conversation history for conv1");
        assert_eq!(conv1[0].role, "system");
        assert_eq!(conv1[1].role, "user");
        assert_eq!(conv1[1].content, Some("Tell me about Python".to_string()));

        let conv2 = history
            .get("conv2")
            .expect("No conversation history for conv2");
        assert_eq!(conv2[0].role, "system");
        assert_eq!(conv2[1].role, "user");
        assert_eq!(
            conv2[1].content,
            Some("Tell me about JavaScript".to_string())
        );
    }
}
