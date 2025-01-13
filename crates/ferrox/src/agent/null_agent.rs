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
    ) -> Pin<Box<dyn Future<Output = Result<String, String>>>> {
        Box::pin(async { Ok(String::new()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_null_agent() {
        let agent = NullAgent;

        assert_eq!(agent.system_prompt(), "");

        let response = agent
            .process_prompt("test prompt")
            .await
            .expect("Failed to process prompt");

        assert_eq!(response, "");
    }
}
