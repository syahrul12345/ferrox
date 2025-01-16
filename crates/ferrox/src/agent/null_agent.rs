use super::Agent;
use crate::action::FunctionAction;
use std::{future::Future, pin::Pin, sync::Arc};

/// A no-op agent implementation used primarily for testing
#[derive(Clone)]
pub struct NullAgent;

impl Agent for NullAgent {
    fn system_prompt(&self) -> &str {
        ""
    }

    fn process_prompt(
        &self,
        _prompt: &str,
        _history_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Sync + Send>> {
        Box::pin(async { Ok(String::new()) })
    }

    fn add_action(&mut self, _action: Arc<FunctionAction>) {
        // NullAgent ignores actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{ActionBuilder, ActionDefinition};

    #[tokio::test]
    async fn test_system_prompt() {
        let agent = NullAgent;
        assert_eq!(agent.system_prompt(), "");
    }

    #[tokio::test]
    async fn test_process_prompt() {
        let agent = NullAgent;
        let response = agent
            .process_prompt("test prompt", "test_history")
            .await
            .expect("Failed to process prompt");

        assert_eq!(response, "");
    }

    #[tokio::test]
    async fn test_add_action() {
        let mut agent = NullAgent;

        // Create a test action using ActionBuilder
        async fn mock_handler(_: serde_json::Value) -> Result<String, String> {
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
        let agent = NullAgent;
        let response = agent
            .process_prompt("", "test_history")
            .await
            .expect("Failed to process prompt");

        assert_eq!(response, "");
    }

    #[tokio::test]
    async fn test_clone() {
        let agent = NullAgent;
        let _cloned = agent.clone();
        // If we got here, clone worked
    }
}
