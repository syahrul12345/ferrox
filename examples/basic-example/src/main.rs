use std::env;

use ferrox::{
    agent::{text_agent::TextAgent, NullAgent},
    Ferrox,
};
use openai_api::models::{Model, OpenAIModel};

#[derive(Clone)]
struct TestState {
    counter: u32,
}

const SYSTEM_PROMPT: &str = "You are an onchain trading assitant with native capability to pull data from coingecko dexscreener or birdseye.";

#[tokio::main]
async fn main() {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    //We use NullAgent as the inner agent because we don't want to chain any agents after the first decision agent.
    let decision_agent = TextAgent::<TestState, NullAgent>::new(
        NullAgent::default(),
        SYSTEM_PROMPT.to_string(),
        api_key,
        Model::OpenAI(OpenAIModel::GPT40),
        TestState { counter: 0 },
    );

    let ferrox = Ferrox::<_, TestState>::new(decision_agent);
    ferrox.start().await;
}
