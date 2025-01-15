use std::env;

use ferrox::{
    agent::{text_agent::TextAgent, NullAgent},
    Ferrox,
};
use openai_api::models::{Model, OpenAIModel};

const SYSTEM_PROMPT: &str = "You are an onchain trading assitant with native capability to pull data from coingecko dexscreener or birdseye.";

//In this example, the bot will be connected to a single agent.
//We also register some basic onchain functions for the agent to use.
#[tokio::main]
async fn main() {
    //Make sure to set the TELOXIDE_TOKEN and OPENAI_API_KEY environment variables.
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let decision_agent = TextAgent::<NullAgent>::new(
        None,
        SYSTEM_PROMPT.to_string(),
        api_key,
        Model::OpenAI(OpenAIModel::GPT40),
    );
    let ferrox = Ferrox::new(decision_agent);
    ferrox.start().await;
    //You can now chat wit the bot.
}
