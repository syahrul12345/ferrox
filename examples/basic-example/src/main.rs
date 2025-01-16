use std::{env, sync::Arc};

use ferrox::{
    action::{ActionBuilder, EmptyParams},
    agent::{text_agent::TextAgent, Agent, AgentState, NullAgent},
    Ferrox,
};
use openai_api::models::{Model, OpenAIModel};
use serde::Deserialize;

#[derive(Clone)]
struct TestState {
    counter: u32,
}

#[derive(Deserialize, Debug)]
struct HelloParams {
    name: String,
}

const SYSTEM_PROMPT: &str = "You are an onchain trading assitant with native capability to pull data from coingecko dexscreener or birdseye.";

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let mut decision_agent = TextAgent::<TestState, NullAgent>::new(
        NullAgent::default(),
        SYSTEM_PROMPT.to_string(),
        api_key,
        Model::OpenAI(OpenAIModel::GPT40),
        TestState { counter: 0 },
    );

    //Now let's add some actions to the decision agent.
    {
        //Greets the user with their name
        async fn say_hello(
            params: HelloParams,
            _state: AgentState<TestState>,
        ) -> Result<String, String> {
            println!("say hello called. Params: {:?}", params);
            Ok(format!("Hello, {}!", params.name))
        }
        let hello_action = ActionBuilder::<_, HelloParams, TestState>::new("say_hello", say_hello)
            .description("Greets the user with their name")
            .parameter("name", "Name of the person to greet", "string", true)
            .build();
        decision_agent.add_action(Arc::new(hello_action));

        /// Increments the counter in the state and returns the new value
        async fn increment_counter(
            _params: EmptyParams,
            state: AgentState<TestState>,
        ) -> Result<String, String> {
            println!("increment counter called. Params: {:?}", _params);
            let mut state = state.lock().await;
            state.counter += 1;
            Ok(format!("Counter incremented to: {}", state.counter))
        }
        let var_name = "Increments the internal counter and returns the new value";
        let increment_action =
            ActionBuilder::<_, EmptyParams, TestState>::new("increment_counter", increment_counter)
                .description(var_name)
                .build();
        decision_agent.add_action(Arc::new(increment_action));

        /// Returns the current counter value
        async fn get_counter(
            _params: EmptyParams,
            state: AgentState<TestState>,
        ) -> Result<String, String> {
            println!("get counter called. Params: {:?}", _params);
            let state = state.lock().await;
            Ok(format!("Current counter value: {}", state.counter))
        }
        let get_counter_action =
            ActionBuilder::<_, EmptyParams, TestState>::new("get_counter", get_counter)
                .description("Returns the current counter value")
                .build();
        decision_agent.add_action(Arc::new(get_counter_action));
    }

    let ferrox = Ferrox::<_, TestState>::new(decision_agent);
    ferrox.start().await;
}
