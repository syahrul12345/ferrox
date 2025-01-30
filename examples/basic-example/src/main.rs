use std::{env, str::FromStr, sync::Arc};

use ferrox::{
    agent::{text_agent::TextAgent, Agent, NullAgent},
    Ferrox, Message,
};
use ferrox_actions::{
    ActionBuilder, AgentState, BirdeyeActionGroup, CoinGeckoActionGroup, DexScreenerActionGroup,
    EmptyParams, GmgnActionGroup,
};
use ferrox_wallet::{simple_wallet_manager::SimpleWalletManager, Wallet, WalletManager};
use openai_api::models::{Model, OpenAIModel};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
};

#[derive(Clone)]
struct TestState {
    counter: u32,
    wallet_manager: SimpleWalletManager,
}

#[derive(Deserialize, Debug)]
struct HelloParams {
    name: String,
}

const SYSTEM_PROMPT: &str = "
You are an onchain trading assitant with native capability to pull data from coingecko dexscreener or birdseye.
If asked to do a multi step action, and one of the steps produces invalid data our empty data, try to call an alternative api from the lsit of dexscreener, birdseye or coingecko api set.
For example when asked for technical analaysis, you can first get the tick data via the birdseye OHLCV API and then use that data to create the actual technical analyssis
";

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let wallet_manager = SimpleWalletManager::new();
    let mut decision_agent = TextAgent::<TestState, NullAgent>::new(
        NullAgent::default(),
        SYSTEM_PROMPT.to_string(),
        api_key,
        Model::OpenAI(OpenAIModel::GPT40),
        TestState {
            counter: 0,
            wallet_manager,
        },
    );

    //Now let's add some actions to the decision agent.
    {
        //Greets the user with their name
        async fn say_hello(
            params: HelloParams,
            _send_state: (),
            _state: AgentState<TestState>,
        ) -> Result<String, String> {
            println!("say hello called. Params: {:?}", params);
            Ok(format!("Hello, {}!", params.name))
        }
        let hello_action =
            ActionBuilder::<_, HelloParams, (), TestState>::new("say_hello", say_hello, None)
                .description("Greets the user with their name")
                .parameter("name", "Name of the person to greet", "string", true)
                .build();
        decision_agent.add_action(Arc::new(hello_action));

        /// Increments the counter in the state and returns the new value
        async fn increment_counter(
            _params: EmptyParams,
            _send_state: (),
            state: AgentState<TestState>,
        ) -> Result<String, String> {
            println!("increment counter called. Params: {:?}", _params);
            let mut state = state.lock().await;
            state.counter += 1;
            Ok(format!("Counter incremented to: {}", state.counter))
        }
        let var_name = "Increments the internal counter and returns the new value";
        let increment_action = ActionBuilder::<_, EmptyParams, (), TestState>::new(
            "increment_counter",
            increment_counter,
            None,
        )
        .description(var_name)
        .build();
        decision_agent.add_action(Arc::new(increment_action));

        /// Returns the current counter value
        async fn get_counter(
            _params: EmptyParams,
            _send_state: (),
            state: AgentState<TestState>,
        ) -> Result<String, String> {
            println!("get counter called. Params: {:?}", _params);
            let state = state.lock().await;
            Ok(format!("Current counter value: {}", state.counter))
        }
        let get_counter_action =
            ActionBuilder::<_, EmptyParams, (), TestState>::new("get_counter", get_counter, None)
                .description("Returns the current counter value")
                .build();
        decision_agent.add_action(Arc::new(get_counter_action));
    }

    {
        //The data the agent will call the preview_send_solana with
        #[derive(Deserialize, Debug)]
        struct SendSolanaParams {
            target_wallet: String,
            amount_to_send: String,
        }
        //The data the user will see in the preview
        #[derive(Serialize, Deserialize, Debug)]
        struct SendSolanaPreview {
            sender: Pubkey,
            target_wallet: Pubkey,
            amount_to_send: u64,
        }

        //This function will cause a preview to be shown to the user
        //The second arugment is the message from telegram.
        //It acceps one parameter which is the target wallet to send to. So the user can prompt the agent to send to a specific wallet.
        async fn preview_send_solana(
            params: SendSolanaParams,
            message: Message,
            state: AgentState<TestState>,
        ) -> Result<SendSolanaPreview, String> {
            println!("LLM called preview send solana");
            let user_id = message.from().unwrap().id.0;
            println!("User ID: {:?}", user_id);
            let amount_to_send =
                (params.amount_to_send.parse::<f64>().unwrap() * 10.0f64.powi(9)).round() as u64;
            let wallet = state
                .lock()
                .await
                .wallet_manager
                .get_wallet(&format!("{:?}", user_id))
                .await
                .unwrap();
            let wallet = match wallet {
                Wallet::Solana(wallet) => wallet.pubkey(),
            };
            println!("Wallet: {:?}", wallet);
            let target_wallet = Pubkey::from_str(&params.target_wallet).unwrap();
            Ok(SendSolanaPreview {
                sender: wallet,
                target_wallet,
                amount_to_send,
            })
        }

        //NOTE: The params value in the confirm MUST match the output type of the preview
        async fn confirm_send_solana(
            _params: SendSolanaPreview,
            _message: Message,
            _state: AgentState<TestState>,
        ) -> Result<String, String> {
            println!("User clicked confirm send solana");
            // For now we just return a dummy signature
            // In reality, we can use the input parameters to hit some backend service to send the transaction or do some processing
            Ok(Signature::new_unique().to_string())
        }

        //Create the action
        let get_send_solana_action =
            ActionBuilder::<_, SendSolanaParams, Message, TestState, SendSolanaPreview, _>::new(
                "send_solana",
                preview_send_solana,
                Some(confirm_send_solana),
            )
            .description("
                Generates the payload to send SOL to a target wallet. 
                This action itself will not send the SOL, but merely a preview for the user to confirm. 
                Never mention that the sol has been sent, nor is this a preview. Prompt the user to confirm.")
            .parameter(
                "target_wallet",
                "Target wallet to send SOL to",
                "string",
                true,
            )
            .parameter("amount_to_send", "Amount of SOL to send", "string", true)
            .build();
        decision_agent.add_action(Arc::new(get_send_solana_action));
    }

    //Coingecko actions
    let coingecko_group = CoinGeckoActionGroup::new();
    decision_agent.add_action_group(&coingecko_group);

    //Dexscreener actions
    let dexscreener_group = DexScreenerActionGroup::new();
    decision_agent.add_action_group(&dexscreener_group);

    //Birdeye actions
    let birdeye_group = BirdeyeActionGroup::new();
    decision_agent.add_action_group(&birdeye_group);

    //GMGN actions
    let gmgn_group = GmgnActionGroup::new();
    decision_agent.add_action_group(&gmgn_group);

    let ferrox = Ferrox::<_, TestState>::new(decision_agent);
    ferrox.start().await;
}
