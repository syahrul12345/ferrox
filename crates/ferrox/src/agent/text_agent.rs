use super::Agent;
use crate::action::{Action, ActionDefinition};
use openai_api::{
    completions::Client as OpenAIClient,
    models::{Message, Model},
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
    actions: Vec<Box<dyn Action>>,
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
            actions: Vec::new(),
        }
    }

    fn send_prompt(
        &self,
        prompt: &str,
        history_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + 'static>> {
        // Clone the data we need to move into the async block
        let conversation_history = self.conversation_history.clone();
        let system_prompt = self.system_prompt.clone();
        let open_ai_client = self.open_ai_client.clone();
        let history_id = history_id.to_string();
        let prompt = prompt.to_string();

        Box::pin(async move {
            let mut history = {
                let mut history_map = conversation_history.lock().map_err(|e| e.to_string())?;

                if let Some(existing_history) = history_map.get(&history_id) {
                    // Clone existing history
                    existing_history.clone()
                } else {
                    // Create new history with system prompt
                    let mut new_history = Vec::new();
                    new_history.push(Message {
                        role: "system".to_string(),
                        content: system_prompt,
                    });
                    history_map.insert(history_id.clone(), new_history.clone());
                    new_history
                }
            };

            // Add the new prompt to history
            history.push(Message {
                role: "user".to_string(),
                content: prompt.clone(),
            });

            // Send to OpenAI
            let response = open_ai_client
                .send_prompt(prompt, history.clone())
                .await
                .map_err(|e| e.to_string())?;

            // Update history with assistant's response
            {
                let mut history_map = conversation_history.lock().map_err(|e| e.to_string())?;
                if let Some(conversation) = history_map.get_mut(&history_id) {
                    conversation.extend(history.iter().skip(conversation.len()).cloned());
                    conversation.push(Message {
                        role: "assistant".to_string(),
                        content: response.clone(),
                    });
                }
            }

            Ok(response)
        })
    }

    pub fn with_action(mut self, action: impl Action + 'static) -> Self {
        self.actions.push(Box::new(action));
        self
    }

    fn get_tool_definitions(&self) -> Vec<ActionDefinition> {
        self.actions
            .iter()
            .map(|action| action.definition())
            .collect()
    }
}

impl<T: Agent> Agent for TextAgent<T> {
    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    fn process_prompt(
        &self,
        prompt: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>>>> {
        // Generate a unique history ID if needed
        let history_id = "default".to_string(); // You might want to generate this differently
        self.send_prompt(prompt, &history_id)
    }
}

//For these tests make sure to set the OPENAI_API_KEY environment variable
#[cfg(test)]
mod tests {
    use crate::agent::NullAgent;

    use super::*;
    use openai_api::models::OpenAIModel;
    use std::env;

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
            .process_prompt("What is Rust programming language?")
            .await
            .expect("Failed to get response");

        println!("First response: {}", response);
        assert!(!response.is_empty());

        // Test follow-up question (should maintain context)
        let response = agent
            .process_prompt("What are its main features?")
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
                    .send_prompt(&prompt, &id)
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
        assert_eq!(conv1[1].content, "Tell me about Python");

        let conv2 = history
            .get("conv2")
            .expect("No conversation history for conv2");
        assert_eq!(conv2[0].role, "system");
        assert_eq!(conv2[1].role, "user");
        assert_eq!(conv2[1].content, "Tell me about JavaScript");
    }
}
