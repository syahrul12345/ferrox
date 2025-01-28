mod action;
pub mod birdeye;
pub mod coingecko;
pub mod dexscreener;
pub mod gmgn;

use std::sync::Arc;

pub use action::{
    ActionBuilder, ActionDefinition, ActionGroup, ActionParameter, EmptyParams, FunctionAction,
};
pub use birdeye::BirdeyeActionGroup;
pub use coingecko::CoinGeckoActionGroup;
pub use dexscreener::DexScreenerActionGroup;
pub use gmgn::GmgnActionGroup;

pub type AgentState<S> = Arc<Mutex<S>>;
use tokio::sync::Mutex;
