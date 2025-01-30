pub mod pro;

use crate::{
    action::{ActionBuilder, ActionGroup, FunctionAction},
    AgentState,
};
use pro::CoinGeckoProClient;
use serde::Deserialize;
use std::sync::Arc;

// Parameter structs for each action
#[derive(Debug, Deserialize)]
pub struct CoinContractMarketChartRangeParams {
    id: String,
    contract_address: String,
    vs_currency: String,
    from: u64,
    to: u64,
}

#[derive(Debug, Deserialize)]
pub struct CoinMarketChartParams {
    id: String,
    vs_currency: String,
    days: String,
    interval: Option<String>,
}

// Add these parameter structs at the top with the other parameter structs
#[derive(Debug, Deserialize)]
pub struct NetworkStatusParams {} // Empty struct for endpoints with no parameters

#[derive(Debug, Deserialize)]
pub struct GlobalDataParams {}

#[derive(Debug, Deserialize)]
pub struct GlobalDefiDataParams {}

#[derive(Debug, Deserialize)]
pub struct ExchangesParams {
    per_page: Option<u32>,
    page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeParams {
    id: String,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeTickersParams {
    id: String,
    coin_ids: Option<Vec<String>>,
    include_exchange_logo: Option<bool>,
    page: Option<u32>,
    depth: Option<bool>,
    order: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeVolumeChartParams {
    id: String,
    days: u32,
}

#[derive(Debug, Deserialize)]
pub struct CoinsListParams {
    include_platform: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CoinTickersParams {
    id: String,
    exchange_ids: Option<Vec<String>>,
    include_exchange_logo: Option<bool>,
    page: Option<u32>,
    order: Option<String>,
    depth: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CoinHistoryParams {
    id: String,
    date: String,
    localization: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CoinOhlcParams {
    id: String,
    vs_currency: String,
    days: String,
}

#[derive(Debug, Deserialize)]
pub struct CoinContractParams {
    id: String,
    contract_address: String,
}

#[derive(Debug, Deserialize)]
pub struct CoinContractMarketChartParams {
    id: String,
    contract_address: String,
    vs_currency: String,
    days: String,
}

// Add these parameter structs at the top with the other parameter structs
#[derive(Debug, Deserialize)]
pub struct AssetPlatformsParams {} // Empty struct for endpoints with no parameters

#[derive(Debug, Deserialize)]
pub struct CoinsCategoriesListParams {}

#[derive(Debug, Deserialize)]
pub struct CoinsCategoriesParams {
    order: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IndexesParams {}

#[derive(Debug, Deserialize)]
pub struct IndexesListParams {}

#[derive(Debug, Deserialize)]
pub struct DerivativesParams {}

#[derive(Debug, Deserialize)]
pub struct DerivativesExchangesParams {
    order: Option<String>,
    per_page: Option<u32>,
    page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct DerivativesExchangeParams {
    id: String,
    include_tickers: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeRatesParams {}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    query: String,
}

#[derive(Debug, Deserialize)]
pub struct TrendingParams {}

#[derive(Debug, Deserialize)]
pub struct CompaniesPublicTreasuryParams {
    coin_id: String,
}

// Action group that contains all CoinGecko actions
pub struct CoinGeckoActionGroup<S: Send + Sync + Clone + 'static> {
    actions: Vec<Arc<FunctionAction<S>>>,
}

impl<S: Send + Sync + Clone + 'static> ActionGroup<S> for CoinGeckoActionGroup<S> {
    fn actions(&self) -> &[Arc<FunctionAction<S>>] {
        &self.actions
    }
}

impl<S: Send + Sync + Clone + 'static> CoinGeckoActionGroup<S> {
    pub fn new() -> Self {
        let mut actions = Vec::new();

        // Add coin contract market chart range action
        {
            async fn get_coin_contract_market_chart_range<S: Send + Sync + Clone + 'static>(
                params: CoinContractMarketChartRangeParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client
                    .get_coin_contract_market_chart_range(
                        params.id,
                        params.contract_address,
                        params.vs_currency,
                        params.from,
                        params.to,
                    )
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_coin_contract_market_chart_range",
                get_coin_contract_market_chart_range,
                None,
            )
            .description("Get historical market data for a token contract address")
            .parameter("id", "The coin id (e.g. ethereum)", "string", true)
            .parameter(
                "contract_address",
                "Token's contract address",
                "string",
                true,
            )
            .parameter(
                "vs_currency",
                "The target currency (e.g. usd)",
                "string",
                true,
            )
            .parameter("from", "From timestamp (Unix timestamp)", "integer", true)
            .parameter("to", "To timestamp (Unix timestamp)", "integer", true)
            .build();

            actions.push(Arc::new(action));
        }

        // Add coin market chart action
        {
            async fn get_coin_market_chart<S: Send + Sync + Clone + 'static>(
                params: CoinMarketChartParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client
                    .get_coin_market_chart(
                        params.id,
                        params.vs_currency,
                        params.days,
                        params.interval,
                    )
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_coin_market_chart",
                get_coin_market_chart,
                None,
            )
            .description("Get historical market data include price, market cap, and 24h volume")
            .parameter("id", "The coin id (e.g. bitcoin)", "string", true)
            .parameter(
                "vs_currency",
                "The target currency (e.g. usd)",
                "string",
                true,
            )
            .parameter("days", "Data up to number of days ago", "string", true)
            .parameter("interval", "Data interval (e.g. daily)", "string", false)
            .build();

            actions.push(Arc::new(action));
        }

        // Add network status action
        {
            async fn get_network_status<S: Send + Sync + Clone + 'static>(
                _params: NetworkStatusParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_network_status().await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_network_status", get_network_status, None)
                    .description("Check API server status")
                    .build();

            actions.push(Arc::new(action));
        }

        // Add global data action
        {
            async fn get_global_data<S: Send + Sync + Clone + 'static>(
                _params: GlobalDataParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_global_data().await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_global_data", get_global_data, None)
                .description("Get cryptocurrency global data")
                .build();

            actions.push(Arc::new(action));
        }

        // Add global defi data action
        {
            async fn get_global_defi_data<S: Send + Sync + Clone + 'static>(
                _params: GlobalDefiDataParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_global_defi_data().await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_global_defi_data",
                get_global_defi_data,
                None,
            )
            .description("Get cryptocurrency global decentralized finance (defi) data")
            .build();

            actions.push(Arc::new(action));
        }

        // Add exchanges action
        {
            async fn get_exchanges<S: Send + Sync + Clone + 'static>(
                params: ExchangesParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_exchanges(params.per_page, params.page).await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_exchanges", get_exchanges, None)
                .description("List all exchanges")
                .parameter("per_page", "Total results per page", "integer", false)
                .parameter("page", "Page number", "integer", false)
                .build();

            actions.push(Arc::new(action));
        }

        // Add exchange action
        {
            async fn get_exchange<S: Send + Sync + Clone + 'static>(
                params: ExchangeParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_exchange(params.id).await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_exchange", get_exchange, None)
                .description("Get exchange volume in BTC and top 100 tickers only")
                .parameter("id", "Exchange id", "string", true)
                .build();

            actions.push(Arc::new(action));
        }

        // Add exchange tickers action
        {
            async fn get_exchange_tickers<S: Send + Sync + Clone + 'static>(
                params: ExchangeTickersParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client
                    .get_exchange_tickers(
                        params.id,
                        params.coin_ids,
                        params.include_exchange_logo,
                        params.page,
                        params.depth,
                        params.order,
                    )
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_exchange_tickers",
                get_exchange_tickers,
                None,
            )
            .description("Get exchange tickers (paginated)")
            .parameter("id", "Exchange id", "string", true)
            .parameter("coin_ids", "Filter tickers by coin ids", "array", false)
            .parameter(
                "include_exchange_logo",
                "Include exchange logo",
                "boolean",
                false,
            )
            .parameter("page", "Page number", "integer", false)
            .parameter("depth", "Include 2% orderbook depth", "boolean", false)
            .parameter("order", "Sort by order", "string", false)
            .build();

            actions.push(Arc::new(action));
        }

        // Add exchange volume chart action
        {
            async fn get_exchange_volume_chart<S: Send + Sync + Clone + 'static>(
                params: ExchangeVolumeChartParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client
                    .get_exchange_volume_chart(params.id, params.days)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_exchange_volume_chart",
                get_exchange_volume_chart,
                None,
            )
            .description("Get volume chart data for a given exchange")
            .parameter("id", "Exchange id", "string", true)
            .parameter("days", "Data up to number of days ago", "integer", true)
            .build();

            actions.push(Arc::new(action));
        }

        // Add coins list action
        {
            async fn get_coins_list<S: Send + Sync + Clone + 'static>(
                params: CoinsListParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_coins_list(params.include_platform).await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_coins_list", get_coins_list, None)
                .description("List all supported coins with id and name")
                .parameter(
                    "include_platform",
                    "Include platform contract addresses",
                    "boolean",
                    false,
                )
                .build();

            actions.push(Arc::new(action));
        }

        // Add coin tickers action
        {
            async fn get_coin_tickers<S: Send + Sync + Clone + 'static>(
                params: CoinTickersParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client
                    .get_coin_tickers(
                        params.id,
                        params.exchange_ids,
                        params.include_exchange_logo,
                        params.page,
                        params.order,
                        params.depth,
                    )
                    .await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_coin_tickers", get_coin_tickers, None)
                    .description("Get coin tickers (paginated to 100 items)")
                    .parameter("id", "The coin id", "string", true)
                    .parameter(
                        "exchange_ids",
                        "Filter results by exchange ids",
                        "array",
                        false,
                    )
                    .parameter(
                        "include_exchange_logo",
                        "Include exchange logo",
                        "boolean",
                        false,
                    )
                    .parameter("page", "Page through results", "integer", false)
                    .parameter("order", "Sort results by order", "string", false)
                    .parameter("depth", "Include 2% orderbook depth", "boolean", false)
                    .build();

            actions.push(Arc::new(action));
        }

        // Add coin history action
        {
            async fn get_coin_history<S: Send + Sync + Clone + 'static>(
                params: CoinHistoryParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client
                    .get_coin_history(params.id, params.date, params.localization)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_coin_history",
                get_coin_history,
                None,
            )
            .description(
                "Get historical data (name, price, market, stats) at a given date for a coin",
            )
            .parameter("id", "The coin id", "string", true)
            .parameter(
                "date",
                "The date of data snapshot in dd-mm-yyyy",
                "string",
                true,
            )
            .parameter(
                "localization",
                "Set to false to exclude localized languages",
                "boolean",
                false,
            )
            .build();

            actions.push(Arc::new(action));
        }

        // Add coin OHLC action
        {
            async fn get_coin_ohlc<S: Send + Sync + Clone + 'static>(
                params: CoinOhlcParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client
                    .get_coin_ohlc(params.id, params.vs_currency, params.days)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_coin_ohlc", get_coin_ohlc, None)
                .description("Get coin's OHLC (Open, High, Low, Close) data")
                .parameter("id", "The coin id", "string", true)
                .parameter(
                    "vs_currency",
                    "The target currency of market data (usd, eur, jpy, etc.)",
                    "string",
                    true,
                )
                .parameter("days", "Data up to number of days ago", "string", true)
                .build();

            actions.push(Arc::new(action));
        }

        // Add coin contract action
        {
            async fn get_coin_contract<S: Send + Sync + Clone + 'static>(
                params: CoinContractParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client
                    .get_coin_contract(params.id, params.contract_address)
                    .await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_coin_contract", get_coin_contract, None)
                    .description("Get coin info from contract address")
                    .parameter("id", "Asset platform (e.g. ethereum)", "string", true)
                    .parameter(
                        "contract_address",
                        "Token's contract address",
                        "string",
                        true,
                    )
                    .build();

            actions.push(Arc::new(action));
        }

        // Add coin contract market chart action
        {
            async fn get_coin_contract_market_chart<S: Send + Sync + Clone + 'static>(
                params: CoinContractMarketChartParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client
                    .get_coin_contract_market_chart(
                        params.id,
                        params.contract_address,
                        params.vs_currency,
                        params.days,
                    )
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_coin_contract_market_chart",
                get_coin_contract_market_chart,
                None,
            )
            .description("Get historical market data for a contract address")
            .parameter("id", "The platform id (e.g. ethereum)", "string", true)
            .parameter(
                "contract_address",
                "Token's contract address",
                "string",
                true,
            )
            .parameter(
                "vs_currency",
                "The target currency (e.g. usd)",
                "string",
                true,
            )
            .parameter("days", "Data up to number of days ago", "string", true)
            .build();

            actions.push(Arc::new(action));
        }

        // Add asset platforms action
        {
            async fn get_asset_platforms<S: Send + Sync + Clone + 'static>(
                _params: AssetPlatformsParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_asset_platforms().await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_asset_platforms", get_asset_platforms, None)
                    .description("List all asset platforms (blockchain networks)")
                    .build();

            actions.push(Arc::new(action));
        }

        // Add coins categories list action
        {
            async fn get_coins_categories_list<S: Send + Sync + Clone + 'static>(
                _params: CoinsCategoriesListParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_coins_categories_list().await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_coins_categories_list",
                get_coins_categories_list,
                None,
            )
            .description("List all categories")
            .build();

            actions.push(Arc::new(action));
        }

        // Add coins categories action
        {
            async fn get_coins_categories<S: Send + Sync + Clone + 'static>(
                params: CoinsCategoriesParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_coins_categories(params.order).await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_coins_categories",
                get_coins_categories,
                None,
            )
            .description("List all categories with market data")
            .parameter("order", "Sort by market_cap or name", "string", false)
            .build();

            actions.push(Arc::new(action));
        }

        // Add indexes action
        {
            async fn get_indexes<S: Send + Sync + Clone + 'static>(
                _params: IndexesParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_indexes().await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_indexes", get_indexes, None)
                .description("List all market indexes")
                .build();

            actions.push(Arc::new(action));
        }

        // Add indexes list action
        {
            async fn get_indexes_list<S: Send + Sync + Clone + 'static>(
                _params: IndexesListParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_indexes_list().await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_indexes_list", get_indexes_list, None)
                    .description("List market indexes id and name")
                    .build();

            actions.push(Arc::new(action));
        }

        // Add derivatives action
        {
            async fn get_derivatives<S: Send + Sync + Clone + 'static>(
                _params: DerivativesParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_derivatives().await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_derivatives", get_derivatives, None)
                .description("List all derivative tickers")
                .build();

            actions.push(Arc::new(action));
        }

        // Add derivatives exchanges action
        {
            async fn get_derivatives_exchanges<S: Send + Sync + Clone + 'static>(
                params: DerivativesExchangesParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client
                    .get_derivatives_exchanges(params.order, params.per_page, params.page)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_derivatives_exchanges",
                get_derivatives_exchanges,
                None,
            )
            .description("List all derivative exchanges")
            .parameter("order", "Order results by specified field", "string", false)
            .parameter("per_page", "Total results per page", "integer", false)
            .parameter("page", "Page through results", "integer", false)
            .build();

            actions.push(Arc::new(action));
        }

        // Add derivatives exchange action
        {
            async fn get_derivatives_exchange<S: Send + Sync + Clone + 'static>(
                params: DerivativesExchangeParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client
                    .get_derivatives_exchange(params.id, params.include_tickers)
                    .await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_derivatives_exchange",
                get_derivatives_exchange,
                None,
            )
            .description("Show derivative exchange data")
            .parameter("id", "The exchange id", "string", true)
            .parameter(
                "include_tickers",
                "Include tickers data (all/unexpired)",
                "string",
                false,
            )
            .build();

            actions.push(Arc::new(action));
        }

        // Add exchange rates action
        {
            async fn get_exchange_rates<S: Send + Sync + Clone + 'static>(
                _params: ExchangeRatesParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_exchange_rates().await
            }

            let action =
                ActionBuilder::<_, _, _, _>::new("get_exchange_rates", get_exchange_rates, None)
                    .description("Get BTC-to-Currency exchange rates")
                    .build();

            actions.push(Arc::new(action));
        }

        // Add search action
        {
            async fn search<S: Send + Sync + Clone + 'static>(
                params: SearchParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.search(params.query).await
            }

            let action = ActionBuilder::<_, _, _, _>::new("search", search, None)
                .description("Search for coins, categories and markets")
                .parameter("query", "Search string", "string", true)
                .build();

            actions.push(Arc::new(action));
        }

        // Add trending action
        {
            async fn get_trending<S: Send + Sync + Clone + 'static>(
                _params: TrendingParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_trending().await
            }

            let action = ActionBuilder::<_, _, _, _>::new("get_trending", get_trending, None)
                .description("Get trending search coins (Top-7) on CoinGecko")
                .build();

            actions.push(Arc::new(action));
        }

        // Add companies public treasury action
        {
            async fn get_companies_public_treasury<S: Send + Sync + Clone + 'static>(
                params: CompaniesPublicTreasuryParams,
                _send_state: serde_json::Value,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let api_key = std::env::var("COINGECKO_PRO_API_KEY").map_err(|_| {
                    "COINGECKO_PRO_API_KEY environment variable not set".to_string()
                })?;

                let client = CoinGeckoProClient::new(api_key);
                client.get_companies_public_treasury(params.coin_id).await
            }

            let action = ActionBuilder::<_, _, _, _>::new(
                "get_companies_public_treasury",
                get_companies_public_treasury,
                None,
            )
            .description("Get public companies bitcoin or ethereum holdings")
            .parameter(
                "coin_id",
                "The coin id (bitcoin or ethereum)",
                "string",
                true,
            )
            .build();

            actions.push(Arc::new(action));
        }

        Self { actions }
    }

    pub fn actions(&self) -> &[Arc<FunctionAction<S>>] {
        &self.actions
    }
}
