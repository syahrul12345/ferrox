pub mod client;

use crate::{
    action::{ActionBuilder, ActionGroup, FunctionAction},
    AgentState,
};
use client::DexScreenerClient;
use serde::Deserialize;
use std::sync::Arc;

// Parameter structs for each action
#[derive(Debug, Deserialize)]
pub struct TokenProfilesParams {} // Empty struct for no params endpoint

#[derive(Debug, Deserialize)]
pub struct TokenOrdersParams {
    chain_id: String,
    token_address: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenBoostsParams {} // Empty struct for no params endpoint

#[derive(Debug, Deserialize)]
pub struct TokenBoostsTopParams {} // Empty struct for no params endpoint

#[derive(Debug, Deserialize)]
pub struct TokenPairsParams {
    chain_id: String,
    token_address: String,
}

#[derive(Debug, Deserialize)]
pub struct TokensParams {
    chain_id: String,
    token_addresses: String, // Comma-separated list of addresses
}

#[derive(Debug, Deserialize)]
pub struct SearchPairsParams {
    query: String,
}

#[derive(Debug, Deserialize)]
pub struct PairsParams {
    chain_id: String,
    pair_id: String,
}

// Action group that contains all DexScreener actions
pub struct DexScreenerActionGroup<S: Send + Sync + Clone + 'static> {
    actions: Vec<Arc<FunctionAction<S>>>,
}

impl<S: Send + Sync + Clone + 'static> ActionGroup<S> for DexScreenerActionGroup<S> {
    fn actions(&self) -> &[Arc<FunctionAction<S>>] {
        &self.actions
    }
}

impl<S: Send + Sync + Clone + 'static> DexScreenerActionGroup<S> {
    pub fn new() -> Self {
        let mut actions = Vec::new();

        // Add get latest token profiles action
        {
            async fn get_token_profiles<S: Send + Sync + Clone + 'static>(
                _params: TokenProfilesParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let client = DexScreenerClient::new();
                client.get_token_profiles().await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_token_profiles", get_token_profiles, None)
                    .description("Get the latest token profiles")
                    .build();

            actions.push(Arc::new(action));
        }

        // Add check token orders action
        {
            async fn get_token_orders<S: Send + Sync + Clone + 'static>(
                params: TokenOrdersParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let client = DexScreenerClient::new();
                client
                    .get_token_orders(params.chain_id, params.token_address)
                    .await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_token_orders", get_token_orders, None)
                    .description("Check orders paid for of token")
                    .parameter("chain_id", "The chain ID (e.g. solana)", "string", true)
                    .parameter("token_address", "Token's address", "string", true)
                    .build();

            actions.push(Arc::new(action));
        }

        // Add get latest token boosts action
        {
            async fn get_token_boosts<S: Send + Sync + Clone + 'static>(
                _params: TokenBoostsParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let client = DexScreenerClient::new();
                client.get_token_boosts().await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_token_boosts", get_token_boosts, None)
                    .description("Get the latest boosted tokens")
                    .build();

            actions.push(Arc::new(action));
        }

        // Add get top token boosts action
        {
            async fn get_token_boosts_top<S: Send + Sync + Clone + 'static>(
                _params: TokenBoostsTopParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let client = DexScreenerClient::new();
                client.get_token_boosts_top().await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_token_boosts_top",
                get_token_boosts_top,
                None,
            )
            .description("Get the tokens with most active boosts")
            .build();

            actions.push(Arc::new(action));
        }

        // Add get token pairs action
        {
            async fn get_token_pairs<S: Send + Sync + Clone + 'static>(
                params: TokenPairsParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let client = DexScreenerClient::new();
                client
                    .get_token_pairs(params.chain_id, params.token_address)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_token_pairs", get_token_pairs, None)
                .description("Get the pools of a given token address")
                .parameter("chain_id", "The chain ID (e.g. solana)", "string", true)
                .parameter("token_address", "Token's address", "string", true)
                .build();

            actions.push(Arc::new(action));
        }

        // Add get tokens action
        {
            async fn get_tokens<S: Send + Sync + Clone + 'static>(
                params: TokensParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let client = DexScreenerClient::new();
                client
                    .get_tokens(params.chain_id, params.token_addresses)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_tokens", get_tokens, None)
                .description("Get one or multiple pairs by token address")
                .parameter("chain_id", "The chain ID (e.g. solana)", "string", true)
                .parameter(
                    "token_addresses",
                    "Comma-separated list of token addresses (up to 30)",
                    "string",
                    true,
                )
                .build();

            actions.push(Arc::new(action));
        }

        // Add search pairs action
        {
            async fn search_pairs<S: Send + Sync + Clone + 'static>(
                params: SearchPairsParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let client = DexScreenerClient::new();
                client.search_pairs(params.query).await
            }

            let action = ActionBuilder::<_, _, _, _>::new("search_pairs", search_pairs, None)
                .description("Search for pairs or tokens matching query")
                .parameter("query", "Search query", "string", true)
                .build();

            actions.push(Arc::new(action));
        }

        // Add get pairs action
        {
            async fn get_pairs<S: Send + Sync + Clone + 'static>(
                params: PairsParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let client = DexScreenerClient::new();
                client.get_pairs(params.chain_id, params.pair_id).await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_pairs", get_pairs, None)
                .description("Get one or multiple pairs by chain and pair address")
                .parameter("chain_id", "The chain ID (e.g. solana)", "string", true)
                .parameter("pair_id", "Pair ID", "string", true)
                .build();

            actions.push(Arc::new(action));
        }

        Self { actions }
    }
}
