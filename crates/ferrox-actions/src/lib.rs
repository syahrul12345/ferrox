mod action;
pub mod coingecko;
pub mod dexscreener;

use std::sync::Arc;

pub use action::{
    ActionBuilder, ActionDefinition, ActionGroup, ActionParameter, EmptyParams, FunctionAction,
};
pub use coingecko::CoinGeckoActionGroup;
pub use dexscreener::DexScreenerActionGroup;

pub type AgentState<S> = Arc<Mutex<S>>;
use tokio::sync::Mutex;
