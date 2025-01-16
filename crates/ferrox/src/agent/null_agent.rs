use tokio::sync::Mutex;

use super::{Agent, AgentState};
use crate::action::FunctionAction;
use std::{future::Future, pin::Pin, sync::Arc};

/// A no-op agent implementation used primarily for testing
#[derive(Clone)]
pub struct NullAgent {
    state: AgentState<()>,
}

impl Default for NullAgent {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(())),
        }
    }
}

impl Agent for NullAgent {
    fn system_prompt(&self) -> &str {
        ""
    }

    fn state(&self) -> &AgentState<()> {
        &self.state
    }

    fn process_prompt(
        &self,
        _prompt: &str,
        _history_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>> {
        let prompt = _prompt.to_string();
        Box::pin(async move { Ok(prompt.to_string()) })
    }

    fn add_action(&mut self, _action: Arc<FunctionAction<()>>) {
        // NullAgent ignores actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::ActionBuilder;

    #[tokio::test]
    async fn test_system_prompt() {
        let agent = NullAgent::default();
        assert_eq!(agent.system_prompt(), "");
    }

    #[tokio::test]
    async fn test_process_prompt() {
        let agent = NullAgent::default();
        let response = agent
            .process_prompt("test prompt", "test_history")
            .await
            .expect("Failed to process prompt");

        assert_eq!(response, "");
    }

    #[tokio::test]
    async fn test_add_action() {
        let mut agent = NullAgent::default();

        // Create a test action using ActionBuilder
        async fn mock_handler(_: serde_json::Value, _: AgentState<()>) -> Result<String, String> {
            Ok("mock result".to_string())
        }

        let action = ActionBuilder::new("mock_action", mock_handler)
            .description("A mock action for testing")
            .build();

        // Should not panic or have any effect
        agent.add_action(Arc::new(action));
    }

    #[tokio::test]
    async fn test_process_prompt_with_empty_input() {
        let agent = NullAgent::default();
        let response = agent
            .process_prompt("", "test_history")
            .await
            .expect("Failed to process prompt");

        assert_eq!(response, "");
    }

    #[tokio::test]
    async fn test_clone() {
        let agent = NullAgent::default();
        let _cloned = agent.clone();
        // If we got here, clone worked
    }
}
