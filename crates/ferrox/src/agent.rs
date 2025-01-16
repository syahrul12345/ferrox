pub mod null_agent;
pub mod text_agent;

use std::{future::Future, pin::Pin, sync::Arc};

pub use null_agent::NullAgent;

use crate::action::FunctionAction;

/// Agent trait represents an LLM with state management capabilities
/// The state type S must be Send + Sync + Clone + 'static
pub trait Agent<S: Send + Sync + Clone + 'static = ()>: Clone {
    /// Adds a tool to the agent
    fn add_action(&mut self, action: Arc<FunctionAction<S>>);

    /// Returns the system prompt for the agent
    fn system_prompt(&self) -> &str;

    /// Returns a reference to the agent's state
    fn state(&self) -> &S;

    /// Returns a mutable reference to the agent's state
    fn state_mut(&mut self) -> &mut S;

    /// Takes in a prompt and returns the stringified response.
    /// This will automatically add the tool calls to the prompt.
    fn process_prompt(
        &self,
        prompt: &str,
        history_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>>;
}
