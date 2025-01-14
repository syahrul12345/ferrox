pub mod null_agent;
pub mod text_agent;

use std::{future::Future, pin::Pin};

pub use null_agent::NullAgent;

use crate::action::Action;

/// Agent trait represents an LLM
/// Agents contains these methods
/// system_prompt: Returns the system prompt for the agent
/// process_prompt: Takes in a prompt and returns the stringified response
/// add_action: Adds an action to the agent
/// Ferrox comes with these agents already implemented
/// `VoiceAgent` and agent that wraps around the whisper-1 model. It's takes in a base64 encoded audio and returns the text
/// `TextAgent` and agent that wraps around any text based LLM. It takes in a prompt and returns the stringified response. OpenAI and Anthropic models are supported.
/// `AssistantAgent` and agent that wraps around any text based LLM. It takes in a prompt and returns the stringified response, but uses openAI asssitant API. As such, only openAI models are supported.
pub trait Agent {
    /// Adds an tool to the agent
    fn add_action(&mut self, action: Box<dyn Action>);
    /// Returns the system prompt for the agent
    fn system_prompt(&self) -> &str;
    /// Takes in a prompt and returns the stringified response. This will automatically add the tool calls to the prompt.
    fn process_prompt(
        &self,
        prompt: &str,
        history_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>>>>;
}
