use super::Agent;
use ferrox_actions::{AgentState, FunctionAction};
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

#[derive(Clone)]
pub struct TextAgent<S, T>
where
    S: Send + Sync + Clone + 'static,
    T: Agent + Send + Sync + 'static,
{
    pub inner_agent: T,
    pub system_prompt: String,
    pub open_ai_client: OpenAIClient,
    conversation_history: Arc<Mutex<HashMap<String, Vec<Message>>>>,
    actions: Arc<Mutex<Vec<Arc<FunctionAction<S>>>>>,
    state: AgentState<S>,
}

impl<S, T> TextAgent<S, T>
where
    S: Send + Sync + Clone + 'static,
    T: Agent + Send + Sync + 'static,
{
    pub fn new(
        inner_agent: T,
        system_prompt: String,
        api_key: String,
        model: Model,
        state: S,
    ) -> Self {
        Self {
            inner_agent,
            system_prompt,
            open_ai_client: OpenAIClient::new(api_key, model),
            conversation_history: Arc::new(Mutex::new(HashMap::new())),
            actions: Arc::new(Mutex::new(Vec::new())),
            state: Arc::new(tokio::sync::Mutex::new(state)),
        }
    }

    fn send_prompt(
        &self,
        prompt: &str,
        history_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>> {
        // Clone what we need for the async block
        let conversation_history = self.conversation_history.clone();
        let system_prompt = self.system_prompt.clone();
        let state = self.state.clone();
        let open_ai_client = self.open_ai_client.clone();
        let actions = self.actions.clone();
        let history_id = history_id.to_string();
        let prompt = prompt.to_string();

        Box::pin(async move {
            // Get or create conversation history
            let mut conversation = {
                let mut history_map = conversation_history.lock().map_err(|e| e.to_string())?;
                if let Some(existing_history) = history_map.get(&history_id) {
                    existing_history.clone()
                } else {
                    let new_history = vec![Message {
                        role: "system".to_string(),
                        content: Some(system_prompt),
                        tool_calls: None,
                        tool_call_id: None,
                    }];
                    history_map.insert(history_id.to_string(), new_history.clone());
                    new_history
                }
            };

            // Add user's prompt to conversation
            conversation.push(Message {
                role: "user".to_string(),
                content: Some(prompt.clone()),
                tool_calls: None,
                tool_call_id: None,
            });

            // Convert actions to OpenAI tools
            let tools: Vec<Tool> = {
                let actions = actions.lock().map_err(|e| e.to_string())?;
                actions
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
                    .collect()
            };

            let mut final_result = String::new();
            let mut count = 0;
            while count <= 5 {
                let response = open_ai_client
                    .send_prompt_with_tools(
                        if count == 0 {
                            Some(prompt.clone())
                        } else {
                            None
                        },
                        conversation.clone(),
                        tools.clone(),
                    )
                    .await
                    .map_err(|e| e.to_string())?;

                if !response.tool_call {
                    final_result = response.content;
                    break;
                }

                let tool_calls: Vec<openai_api::models::ToolCall> =
                    serde_json::from_str(&response.content).map_err(|e| e.to_string())?;

                // Add assistant's tool calls to conversation
                conversation.push(Message {
                    role: "assistant".to_string(),
                    content: None,
                    tool_calls: Some(tool_calls.clone()),
                    tool_call_id: None,
                });

                // Execute each tool
                let actions = {
                    let actions = actions.lock().map_err(|e| e.to_string())?;
                    let actions_vec = actions.clone();
                    drop(actions);
                    actions_vec
                };
                for tool_call in tool_calls {
                    if let Some(action) = actions
                        .iter()
                        .find(|a| a.definition().name == tool_call.function.name)
                    {
                        let result = action
                            .execute(
                                serde_json::from_str(&tool_call.function.arguments)
                                    .map_err(|e| e.to_string())?,
                                state.clone(),
                            )
                            .await
                            .map_err(|e| {
                                format!("Failed to execute {}: {}", tool_call.function.name, e)
                            })?;
                        println!(
                            "Executed function {} Result {}",
                            tool_call.function.name, result
                        );
                        conversation.push(Message {
                            role: "tool".to_string(),
                            content: Some(result),
                            tool_calls: None,
                            tool_call_id: Some(tool_call.id),
                        });
                    }
                }
                count += 1;
            }

            // Update conversation history
            {
                let mut history_map = conversation_history.lock().map_err(|e| e.to_string())?;
                // Add final assistant message
                conversation.push(Message {
                    role: "assistant".to_string(),
                    content: Some(final_result.clone()),
                    tool_calls: None,
                    tool_call_id: None,
                });
                // Update the history
                history_map.insert(history_id.to_string(), conversation);
            }

            if count == 5 {
                return Err(
                    "Failed to get a final response from the AI agent within 5 rounds".to_string(),
                );
            }
            Ok(final_result)
        })
    }
}

impl<S, T> Agent<S> for TextAgent<S, T>
where
    S: Send + Sync + Clone + 'static,
    T: Agent + Send + Sync + 'static,
{
    fn add_action(&mut self, action: Arc<FunctionAction<S>>) {
        println!("Adding action: {:?}", action.definition().name);
        self.actions.lock().unwrap().push(action);
    }

    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    fn state(&self) -> &AgentState<S> {
        &self.state
    }

    fn process_prompt(
        &self,
        prompt: &str,
        history_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>> {
        let history_id = history_id.to_string();
        let text_future = self.send_prompt(prompt, &history_id);
        let inner_agent = self.inner_agent.clone();
        Box::pin(async move {
            let text_result = text_future.await?;
            let text_result = inner_agent
                .process_prompt(&text_result, &history_id)
                .await?;
            Ok(text_result)
        })
    }
}

//Tests remain the same but need to be updated to use ActionBuilder instead of MockAction
//For these tests make sure to set the OPENAI_API_KEY environment variable
#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::NullAgent;
    use ferrox_actions::{ActionBuilder, EmptyParams};
    use openai_api::models::OpenAIModel;
    use serde::Deserialize;
    use std::env;

    #[derive(Clone, Debug, Default)]
    struct TestState {
        counter: i32,
    }

    #[tokio::test]
    async fn test_text_agent_with_actions() {
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

        let mut agent = TextAgent::<TestState, NullAgent>::new(
            NullAgent::default(),
            "You are a helpful assistant that can perform calculations, generate greetings, and reverse text. \
             Please use the appropriate action when needed.".to_string(),
            api_key,
            Model::OpenAI(OpenAIModel::GPT35Turbo),
            TestState { counter: 0 },
        );

        println!("Created text agent");

        {
            #[derive(Deserialize, Debug)]
            struct CalcParams {
                a: f64,
                b: f64,
                operation: String,
            }
            async fn calculator(
                params: CalcParams,
                state: AgentState<TestState>,
            ) -> Result<String, String> {
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
                state.lock().await.counter += 1;
                Ok(result.to_string())
            }
            let calc_action =
                ActionBuilder::<_, CalcParams, TestState>::new("calculator", calculator, None)
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
            agent.add_action(Arc::new(calc_action));
            println!("Added calculator action");
        }

        {
            #[derive(Deserialize, Debug)]
            struct GreetParams {
                name: String,
                language: Option<String>,
            }
            async fn greeter(
                params: GreetParams,
                state: AgentState<TestState>,
            ) -> Result<String, String> {
                println!("Greeter called with params: {:?}", params);
                let greeting = match params.language.as_deref() {
                    Some("es") => "¡Hola",
                    Some("fr") => "Bonjour",
                    _ => "Hello",
                };
                state.lock().await.counter += 1;
                Ok(format!("{} {}!", greeting, params.name))
            }
            let greet_action =
                ActionBuilder::<_, GreetParams, TestState>::new("greeter", greeter, None)
                    .description("Generate a greeting message")
                    .parameter("name", "Name to greet", "string", true)
                    .parameter("language", "Language code (en/es/fr)", "string", false)
                    .build();
            agent.add_action(Arc::new(greet_action));
            println!("Added greeter action");
        }

        {
            #[derive(Deserialize, Debug)]
            struct ReverseParams {
                text: String,
            }

            async fn reverser(
                params: ReverseParams,
                state: AgentState<TestState>,
            ) -> Result<String, String> {
                println!("Reverser called with params: {:?}", params);
                state.lock().await.counter += 1;
                Ok(params.text.chars().rev().collect())
            }
            let reverse_action =
                ActionBuilder::<_, ReverseParams, TestState>::new("reverser", reverser, None)
                    .description("Reverse input text")
                    .parameter("text", "Text to reverse", "string", true)
                    .build();
            agent.add_action(Arc::new(reverse_action));
            println!("Added reverser action");
        }

        // Test individual actions through the agent
        println!("--------------------------------");
        let calc_prompt = "Calculate 5 plus 3";
        println!("Testing calculator with prompt: {}", calc_prompt);
        let calc_response = agent.process_prompt(calc_prompt, "test1").await.unwrap();
        println!("Calculator response: {}", calc_response);
        assert_eq!(agent.state().lock().await.counter, 1);

        println!("--------------------------------");
        let greet_prompt = "Say hello to Alice in Spanish";
        println!("Testing greeter with prompt: {}", greet_prompt);
        let greet_response = agent.process_prompt(greet_prompt, "test2").await.unwrap();
        println!("Greeter response: {}", greet_response);
        assert_eq!(agent.state().lock().await.counter, 2);

        println!("--------------------------------");
        let reverse_prompt = "Reverse the text 'hello world'";
        println!("Testing reverser with prompt: {}", reverse_prompt);
        let reverse_response = agent.process_prompt(reverse_prompt, "test3").await.unwrap();
        println!("Reverser response: {}", reverse_response);
        assert_eq!(agent.state().lock().await.counter, 3);

        // Test chained actions
        println!("--------------------------------");
        let chained_prompt = "Calculate 10 plus 5, then greet the result in Spanish, and finally reverse that greeting";
        println!("Testing chained actions with prompt: {}", chained_prompt);
        let chained_response = agent.process_prompt(chained_prompt, "test4").await.unwrap();
        println!("Chained actions response: {}", chained_response);
        assert_eq!(agent.state().lock().await.counter, 6); // Should have used all 3 actions
    }

    // Keep the existing conversation tests
    #[tokio::test]
    async fn test_text_agent_conversation() {
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

        let agent = TextAgent::<_, NullAgent>::new(
            NullAgent::default(),
            "You are a helpful assistant that provides concise responses.".to_string(),
            api_key,
            Model::OpenAI(OpenAIModel::GPT35Turbo),
            (),
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

        let agent = TextAgent::<_, NullAgent>::new(
            NullAgent::default(),
            "You are a helpful assistant.".to_string(),
            api_key,
            Model::OpenAI(OpenAIModel::GPT35Turbo),
            (),
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

    #[tokio::test]
    async fn test_chained_text_agents() {
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

        // Create inner agent that adds markdown formatting
        let inner_agent = TextAgent::<_, NullAgent>::new(
            NullAgent::default(),
            "You are a formatting assistant. Your job is to take any text and format it as a markdown quote with emoji bullets. \
             Always format your response like this:\
             \n> 🔹 First point\
             \n> 🔸 Second point\
             \n> 💠 Final point"
                .to_string(),
            api_key.clone(),
            Model::OpenAI(OpenAIModel::GPT35Turbo),
            (),
        );

        // Create outer agent that generates content
        let agent = TextAgent::new(
            inner_agent,
            "You are a helpful assistant that explains technical concepts. \
             Break down your explanations into 2-3 key points."
                .to_string(),
            api_key,
            Model::OpenAI(OpenAIModel::GPT35Turbo),
            (),
        );

        // Test the chain
        let response = agent
            .process_prompt("What is Rust's ownership system?", "test_chain")
            .await
            .expect("Failed to get response");

        println!("Chained response:\n{}", response);

        // Verify the response has the inner agent's formatting
        assert!(response.contains(">"));
        assert!(response.contains("🔹"));
        assert!(response.contains("🔸"));
        assert!(response.contains("💠"));
    }

    #[tokio::test]
    async fn test_empty_params_action() {
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

        let mut agent = TextAgent::<TestState, NullAgent>::new(
            NullAgent::default(),
            "You are a helpful assistant that can get the current time. Please use the time action when asked about the current time."
                .to_string(),
            api_key,
            Model::OpenAI(OpenAIModel::GPT35Turbo),
            TestState { counter: 0 },
        );

        // Create action that takes EmptyParams
        async fn get_time(
            _params: EmptyParams,
            state: AgentState<TestState>,
        ) -> Result<String, String> {
            println!("get_time called. Params: {:?}", _params);
            state.lock().await.counter += 1;
            Ok("12:00 PM".to_string())
        }

        let time_action =
            ActionBuilder::<_, EmptyParams, TestState>::new("get_time", get_time, None)
                .description("Get the current time")
                .build();

        agent.add_action(Arc::new(time_action));

        // Test the action
        let response = agent
            .process_prompt("What time is it?", "test_empty")
            .await
            .unwrap();

        println!("Time response: {}", response);

        // Verify the action was called by checking the counter
        assert_eq!(agent.state().lock().await.counter, 1);

        // Response should contain the time
        assert!(response.contains("12:00 PM"));
    }
}
