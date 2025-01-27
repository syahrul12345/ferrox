mod action;
pub mod coingecko;

use std::sync::Arc;

pub use action::{
    ActionBuilder, ActionDefinition, ActionGroup, ActionParameter, EmptyParams, FunctionAction,
};
pub use coingecko::CoinGeckoActionGroup;

pub type AgentState<S> = Arc<Mutex<S>>;
use tokio::sync::Mutex;
