use tokio::sync::Mutex;

use ferrox_actions::{AgentState, FunctionAction};
use std::{future::Future, pin::Pin, sync::Arc};

use super::{Agent, ConfirmHandler};

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

    fn state(&self) -> AgentState<()> {
        self.state.clone()
    }

    fn process_prompt(
        &self,
        _prompt: &str,
        _history_id: &str,
        _send_state: serde_json::Value,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        (String, Option<(serde_json::Value, ConfirmHandler<()>)>),
                        String,
                    >,
                > + Send
                + Sync,
        >,
    > {
        let prompt = _prompt.to_string();
        Box::pin(async move { Ok((prompt.to_string(), None)) })
    }

    fn add_action(&mut self, _action: Arc<FunctionAction<()>>) {
        // NullAgent ignores actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrox_actions::{ActionBuilder, EmptyParams};
    #[tokio::test]
    async fn test_system_prompt() {
        let agent = NullAgent::default();
        assert_eq!(agent.system_prompt(), "");
    }

    #[tokio::test]
    async fn test_process_prompt() {
        let agent = NullAgent::default();
        let (response, _) = agent
            .process_prompt("test prompt", "test_history", serde_json::Value::Null)
            .await
            .expect("Failed to process prompt");

        assert_eq!(response, "test prompt");
    }

    #[tokio::test]
    async fn test_add_action() {
        let mut agent = NullAgent::default();

        // Create a test action using ActionBuilder
        async fn mock_handler(
            _: EmptyParams,
            _: serde_json::Value,
            _: AgentState<()>,
        ) -> Result<String, String> {
            Ok("mock result".to_string())
        }

        let action = ActionBuilder::<_, _, _, _>::new("mock_action", mock_handler, None)
            .description("A mock action for testing")
            .build();

        // Should not panic or have any effect
        agent.add_action(Arc::new(action));
    }

    #[tokio::test]
    async fn test_process_prompt_with_empty_input() {
        let agent = NullAgent::default();
        let (response, _) = agent
            .process_prompt("", "test_history", serde_json::Value::Null)
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
