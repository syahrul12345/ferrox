pub mod text_agent;
use std::{future::Future, pin::Pin};

/// Agent trait represents an LLM
/// It contains only two methods, system prompt and  process_prompt
/// Ferrox comes with these agents already implemented
/// `VoiceAgent` and agent that wraps around the whisper-1 model. It's takes in a base64 encoded audio and returns the text
/// `TextAgent` and agent that wraps around any text based LLM. It takes in a prompt and returns the stringified response. OpenAI and Anthropic models are supported.
/// `AssistantAgent` and agent that wraps around any text based LLM. It takes in a prompt and returns the stringified response, but uses openAI asssitant API. As such, only openAI models are supported.
pub trait Agent {
    fn system_prompt(&self) -> &str;
    fn process_prompt(&self, prompt: &str)
        -> Pin<Box<dyn Future<Output = Result<String, String>>>>;
}
