use super::Agent;
use std::{future::Future, pin::Pin};

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
    ) -> Pin<Box<dyn Future<Output = Result<String, String>>>> {
        Box::pin(async { Ok(String::new()) })
    }

    fn add_action(&mut self, _action: Box<dyn crate::action::Action>) {
        // NullAgent ignores actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, ActionDefinition};

    // Mock action for testing
    struct MockAction;
    impl Action for MockAction {
        fn definition(&self) -> ActionDefinition {
            ActionDefinition {
                name: "mock_action".to_string(),
                description: "A mock action for testing".to_string(),
                parameters: vec![],
            }
        }

        fn execute(
            &self,
            _params: serde_json::Value,
        ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>> {
            Box::pin(async { Ok("mock result".to_string()) })
        }
    }

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
        // Should not panic or have any effect
        agent.add_action(Box::new(MockAction));
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
