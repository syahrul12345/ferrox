pub mod client;

use crate::{
    action::{ActionBuilder, ActionGroup, FunctionAction},
    AgentState,
};
use client::BirdeyeClient;
use serde::Deserialize;
use std::sync::Arc;

// Parameter structs for each action
#[derive(Debug, Deserialize)]
pub struct TokenPriceParams {
    address: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenPriceHistoryParams {
    address: String,
    resolution: String,
    limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct MultiTokenPriceParams {
    addresses: String, // Comma-separated list of addresses
}

#[derive(Debug, Deserialize)]
pub struct TokenOhlcvParams {
    address: String,
    resolution: String, // "1" | "3" | "5" | "15" | "30" | "60" | "120" | "240" | "360" | "480" | "720" | "1D" | "3D" | "1W" | "1M"
}

#[derive(Debug, Deserialize)]
pub struct PairOhlcvParams {
    pair_address: String,
    resolution: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenTradesParams {
    address: String,
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PairTradesParams {
    pair_address: String,
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TokenOverviewParams {
    address: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenListParams {
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TokenSecurityParams {
    address: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenMarketListParams {
    address: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenNewListingParams {
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TokenTopTradersParams {
    address: String,
    limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TokenTrendingParams {
    limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct GainersLosersParams {}

#[derive(Debug, Deserialize)]
pub struct TraderTxsByTimeParams {
    address: String,
    time_from: i64,
    time_to: i64,
    limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SupportedChainsParams {}

#[derive(Debug, Deserialize)]
pub struct WalletPortfolioParams {
    wallet_address: String,
    chain_id: String,
}

#[derive(Debug, Deserialize)]
pub struct WalletPortfolioMultichainParams {
    wallet_address: String,
}

#[derive(Debug, Deserialize)]
pub struct WalletTransactionHistoryParams {
    wallet_address: String,
    chain_id: String,
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct WalletTransactionHistoryMultichainParams {
    wallet_address: String,
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SimulateTransactionParams {
    chain_id: String,
    tx_data: String,
}

// Action group that contains all Birdeye actions
pub struct BirdeyeActionGroup<S: Send + Sync + Clone + 'static> {
    actions: Vec<Arc<FunctionAction<S>>>,
}

impl<S: Send + Sync + Clone + 'static> ActionGroup<S> for BirdeyeActionGroup<S> {
    fn actions(&self) -> &[Arc<FunctionAction<S>>] {
        &self.actions
    }
}

impl<S: Send + Sync + Clone + 'static> BirdeyeActionGroup<S> {
    pub fn new() -> Self {
        let mut actions = Vec::new();

        // Add token price action
        {
            async fn get_token_price<S: Send + Sync + Clone + 'static>(
                params: TokenPriceParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client.get_token_price(params.address).await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_token_price", get_token_price, None)
                .description("Get real-time price data for a token")
                .parameter("address", "Token address", "string", true)
                .build();

            actions.push(Arc::new(action));
        }

        // Add token price history action
        {
            async fn get_token_price_history<S: Send + Sync + Clone + 'static>(
                params: TokenPriceHistoryParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);

                let time_to = chrono::Utc::now().timestamp();
                let time_from = calculate_time_from(time_to, &params.resolution)?;

                client
                    .get_token_price_history(
                        params.address,
                        params.resolution,
                        Some(time_from),
                        Some(time_to),
                        params.limit,
                    )
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_token_price_history",
                get_token_price_history,
                None,
            )
            .description("Get historical price data for a token")
            .parameter("address", "Token address", "string", true)
            .parameter(
                "resolution",
                "Time resolution (1, 3, 5, 15, 30, 60, 120, 240, 360, 480, 720, 1D, 3D, 1W, 1M)",
                "string",
                true,
            )
            .parameter("limit", "Number of records to return", "integer", false)
            .build();

            actions.push(Arc::new(action));
        }

        // Continue with more actions...
        // Add multi token price action
        {
            async fn get_multi_token_price<S: Send + Sync + Clone + 'static>(
                params: MultiTokenPriceParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client.get_multi_token_price(params.addresses).await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_multi_token_price",
                get_multi_token_price,
                None,
            )
            .description("Get price data for multiple tokens")
            .parameter(
                "addresses",
                "Comma-separated list of token addresses",
                "string",
                true,
            )
            .build();

            actions.push(Arc::new(action));
        }

        // Add token trending action
        {
            async fn get_token_trending<S: Send + Sync + Clone + 'static>(
                params: TokenTrendingParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client.get_token_trending(params.limit).await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_token_trending", get_token_trending, None)
                    .description("Get trending tokens")
                    .parameter("limit", "Number of tokens to return", "integer", false)
                    .build();

            actions.push(Arc::new(action));
        }

        // Add token OHLCV action
        {
            async fn get_token_ohlcv<S: Send + Sync + Clone + 'static>(
                params: TokenOhlcvParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);

                let time_to = chrono::Utc::now().timestamp();
                let time_from = calculate_time_from(time_to, &params.resolution)?;

                client
                    .get_token_ohlcv(params.address, params.resolution, time_from, time_to)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_token_ohlcv",
                get_token_ohlcv,
                None,
            )
            .description("Get OHLCV data for a token (only solana tokens). Do not use if it is an ethereum token")
            .parameter("address", "Token address", "string", true)
            .parameter(
                "resolution",
                "Time resolution (1, 3, 5, 15, 30, 60, 120, 240, 360, 480, 720, 1D, 3D, 1W, 1M)",
                "string",
                true,
            )
            .build();

            actions.push(Arc::new(action));
        }

        // Add pair OHLCV action
        {
            async fn get_pair_ohlcv<S: Send + Sync + Clone + 'static>(
                params: PairOhlcvParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);

                let time_to = chrono::Utc::now().timestamp();
                let time_from = calculate_time_from(time_to, &params.resolution)?;

                client
                    .get_pair_ohlcv(params.pair_address, params.resolution, time_from, time_to)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_pair_ohlcv",
                get_pair_ohlcv,
                None,
            )
            .description("Get OHLCV data for a trading pair")
            .parameter("pair_address", "Pair address", "string", true)
            .parameter(
                "resolution",
                "Time resolution (1, 3, 5, 15, 30, 60, 120, 240, 360, 480, 720, 1D, 3D, 1W, 1M)",
                "string",
                true,
            )
            .build();

            actions.push(Arc::new(action));
        }

        // Add token trades action
        {
            async fn get_token_trades<S: Send + Sync + Clone + 'static>(
                params: TokenTradesParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client
                    .get_token_trades(params.address, params.limit, params.offset)
                    .await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_token_trades", get_token_trades, None)
                    .description("Get recent trades for a token")
                    .parameter("address", "Token address", "string", true)
                    .parameter("limit", "Number of trades to return", "integer", false)
                    .parameter("offset", "Number of trades to skip", "integer", false)
                    .build();

            actions.push(Arc::new(action));
        }

        // Add pair trades action
        {
            async fn get_pair_trades<S: Send + Sync + Clone + 'static>(
                params: PairTradesParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client
                    .get_pair_trades(params.pair_address, params.limit, params.offset)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_pair_trades", get_pair_trades, None)
                .description("Get recent trades for a trading pair")
                .parameter("pair_address", "Pair address", "string", true)
                .parameter("limit", "Number of trades to return", "integer", false)
                .parameter("offset", "Number of trades to skip", "integer", false)
                .build();

            actions.push(Arc::new(action));
        }

        // Add token overview action
        {
            async fn get_token_overview<S: Send + Sync + Clone + 'static>(
                params: TokenOverviewParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client.get_token_overview(params.address).await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_token_overview", get_token_overview, None)
                    .description("Get comprehensive overview data for a token")
                    .parameter("address", "Token address", "string", true)
                    .build();

            actions.push(Arc::new(action));
        }

        // Add token list action
        {
            async fn get_token_list<S: Send + Sync + Clone + 'static>(
                params: TokenListParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client.get_token_list(params.limit, params.offset).await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_token_list", get_token_list, None)
                .description("Get list of tokens with market data")
                .parameter("limit", "Number of tokens to return", "integer", false)
                .parameter("offset", "Number of tokens to skip", "integer", false)
                .build();

            actions.push(Arc::new(action));
        }

        // Add token security action
        {
            async fn get_token_security<S: Send + Sync + Clone + 'static>(
                params: TokenSecurityParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client.get_token_security(params.address).await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_token_security", get_token_security, None)
                    .description("Get security information for a token")
                    .parameter("address", "Token address", "string", true)
                    .build();

            actions.push(Arc::new(action));
        }

        // Add token market list action
        {
            async fn get_token_market_list<S: Send + Sync + Clone + 'static>(
                params: TokenMarketListParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client.get_token_market_list(params.address).await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_token_market_list",
                get_token_market_list,
                None,
            )
            .description("Get list of markets where a token is traded")
            .parameter("address", "Token address", "string", true)
            .build();

            actions.push(Arc::new(action));
        }

        // Add token new listing action
        {
            async fn get_token_new_listing<S: Send + Sync + Clone + 'static>(
                params: TokenNewListingParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client
                    .get_token_new_listing(params.limit, params.offset)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_token_new_listing",
                get_token_new_listing,
                None,
            )
            .description("Get list of newly listed tokens")
            .parameter("limit", "Number of tokens to return", "integer", false)
            .parameter("offset", "Number of tokens to skip", "integer", false)
            .build();

            actions.push(Arc::new(action));
        }

        // Add token top traders action
        {
            async fn get_token_top_traders<S: Send + Sync + Clone + 'static>(
                params: TokenTopTradersParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client
                    .get_token_top_traders(params.address, params.limit)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_token_top_traders",
                get_token_top_traders,
                None,
            )
            .description("Get top traders for a token")
            .parameter("address", "Token address", "string", true)
            .parameter("limit", "Number of traders to return", "integer", false)
            .build();

            actions.push(Arc::new(action));
        }

        // Add gainers/losers action
        {
            async fn get_gainers_losers<S: Send + Sync + Clone + 'static>(
                _params: GainersLosersParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client.get_gainers_losers().await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_gainers_losers", get_gainers_losers, None)
                    .description("Get gainers and losers data")
                    .build();

            actions.push(Arc::new(action));
        }

        // Add trader transactions by time action
        {
            async fn get_trader_txs_by_time<S: Send + Sync + Clone + 'static>(
                params: TraderTxsByTimeParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client
                    .get_trader_txs_by_time(
                        params.address,
                        params.time_from,
                        params.time_to,
                        params.limit,
                    )
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_trader_txs_by_time",
                get_trader_txs_by_time,
                None,
            )
            .description("Get trader transactions within a time range")
            .parameter("address", "Token address", "string", true)
            .parameter("time_from", "Start timestamp (Unix)", "integer", true)
            .parameter("time_to", "End timestamp (Unix)", "integer", true)
            .parameter(
                "limit",
                "Number of transactions to return",
                "integer",
                false,
            )
            .build();

            actions.push(Arc::new(action));
        }

        // Add supported chains action
        {
            async fn list_supported_chains<S: Send + Sync + Clone + 'static>(
                _params: SupportedChainsParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client.list_supported_chains().await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "list_supported_chains",
                list_supported_chains,
                None,
            )
            .description("List supported blockchain networks")
            .build();

            actions.push(Arc::new(action));
        }

        // Add wallet portfolio action
        {
            async fn get_wallet_portfolio<S: Send + Sync + Clone + 'static>(
                params: WalletPortfolioParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client
                    .get_wallet_portfolio(params.wallet_address, params.chain_id)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_wallet_portfolio",
                get_wallet_portfolio,
                None,
            )
            .description("Get wallet portfolio for a specific chain")
            .parameter("wallet_address", "Wallet address", "string", true)
            .parameter("chain_id", "Chain ID", "string", true)
            .build();

            actions.push(Arc::new(action));
        }

        // Add multichain wallet portfolio action
        {
            async fn get_wallet_portfolio_multichain<S: Send + Sync + Clone + 'static>(
                params: WalletPortfolioMultichainParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client
                    .get_wallet_portfolio_multichain(params.wallet_address)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_wallet_portfolio_multichain",
                get_wallet_portfolio_multichain,
                None,
            )
            .description("Get wallet portfolio across all chains")
            .parameter("wallet_address", "Wallet address", "string", true)
            .build();

            actions.push(Arc::new(action));
        }

        // Add wallet transaction history action
        {
            async fn get_wallet_transaction_history<S: Send + Sync + Clone + 'static>(
                params: WalletTransactionHistoryParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client
                    .get_wallet_transaction_history(
                        params.wallet_address,
                        params.chain_id,
                        params.limit,
                        params.offset,
                    )
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_wallet_transaction_history",
                get_wallet_transaction_history,
                None,
            )
            .description("Get wallet transaction history for a specific chain")
            .parameter("wallet_address", "Wallet address", "string", true)
            .parameter("chain_id", "Chain ID", "string", true)
            .parameter(
                "limit",
                "Number of transactions to return",
                "integer",
                false,
            )
            .parameter("offset", "Number of transactions to skip", "integer", false)
            .build();

            actions.push(Arc::new(action));
        }

        // Add multichain wallet transaction history action
        {
            async fn get_wallet_transaction_history_multichain<S: Send + Sync + Clone + 'static>(
                params: WalletTransactionHistoryMultichainParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client
                    .get_wallet_transaction_history_multichain(
                        params.wallet_address,
                        params.limit,
                        params.offset,
                    )
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_wallet_transaction_history_multichain",
                get_wallet_transaction_history_multichain,
                None,
            )
            .description("Get wallet transaction history across all chains")
            .parameter("wallet_address", "Wallet address", "string", true)
            .parameter(
                "limit",
                "Number of transactions to return",
                "integer",
                false,
            )
            .parameter("offset", "Number of transactions to skip", "integer", false)
            .build();

            actions.push(Arc::new(action));
        }

        // Add transaction simulation action
        {
            async fn simulate_transaction<S: Send + Sync + Clone + 'static>(
                params: SimulateTransactionParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("BIRDEYE_API_KEY")
                    .map_err(|_| "BIRDEYE_API_KEY environment variable not set".to_string())?;
                let client = BirdeyeClient::new(api_key);
                client
                    .simulate_transaction(params.chain_id, params.tx_data)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "simulate_transaction",
                simulate_transaction,
                None,
            )
            .description("Simulate a transaction")
            .parameter("chain_id", "Chain ID", "string", true)
            .parameter("tx_data", "Transaction data", "string", true)
            .build();

            actions.push(Arc::new(action));
        }

        Self { actions }
    }
}

// Add helper function to calculate time_from based on resolution
fn calculate_time_from(time_to: i64, resolution: &str) -> Result<i64, String> {
    let duration = match resolution {
        "1" | "3" | "5" | "15" | "30" | "60" => {
            let minutes: i64 = resolution
                .parse()
                .map_err(|_| "Invalid resolution format")?;
            chrono::Duration::minutes(minutes * 100) // Get 100 data points
        }
        "120" | "240" | "360" | "480" | "720" => {
            let minutes: i64 = resolution
                .parse()
                .map_err(|_| "Invalid resolution format")?;
            chrono::Duration::minutes(minutes * 50) // Get 50 data points
        }
        "1D" => chrono::Duration::days(100),
        "3D" => chrono::Duration::days(300),
        "1W" => chrono::Duration::weeks(100),
        "1M" => chrono::Duration::days(100 * 30),
        _ => return Err("Invalid resolution".to_string()),
    };

    Ok(time_to - duration.num_seconds())
}
