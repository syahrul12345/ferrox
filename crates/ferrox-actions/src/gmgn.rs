use crate::{
    action::{ActionBuilder, ActionGroup, FunctionAction},
    AgentState,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
pub struct GmgnKlineResponse {
    pub code: i32,
    pub msg: String,
    pub data: Vec<KlineData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KlineData {
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub time: String,
}

pub async fn fetch_k_line_data_from_gmgn(
    token_address: String,
    time_from: i64,
    time_to: i64,
) -> Result<GmgnKlineResponse, String> {
    let client = reqwest::Client::new();

    let url = format!(
        "https://www.gmgn.cc/defi/quotation/v1/tokens/kline/sol/{}?resolution=1h&from={}&to={}",
        token_address, time_from, time_to
    );
    println!("Fetching kline data from GMGN: {}", url);

    match client.get(&url).send().await {
        Ok(response) => match response.json::<GmgnKlineResponse>().await {
            Ok(kline_data) => Ok(kline_data),
            Err(e) => {
                println!("Failed to parse GMGN response: {}", e);
                Err("Error parsing kline data".to_string())
            }
        },
        Err(e) => {
            println!("Failed to fetch from GMGN: {}", e);
            Err("Failed to fetch kline data".to_string())
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct KlineDataParams {
    token_address: String,
    time_from: i64,
    time_to: i64,
}

pub struct GmgnActionGroup<S: Send + Sync + Clone + 'static> {
    actions: Vec<Arc<FunctionAction<S>>>,
}

impl<S: Send + Sync + Clone + 'static> ActionGroup<S> for GmgnActionGroup<S> {
    fn actions(&self) -> &[Arc<FunctionAction<S>>] {
        &self.actions
    }
}

impl<S: Send + Sync + Clone + 'static> GmgnActionGroup<S> {
    pub fn new() -> Self {
        let mut actions = Vec::new();
        // Add kline data action
        {
            async fn get_kline_data<S: Send + Sync + Clone + 'static>(
                params: KlineDataParams,
                _state: AgentState<S>,
            ) -> Result<String, String> {
                let kline_data = fetch_k_line_data_from_gmgn(
                    params.token_address,
                    params.time_from,
                    params.time_to,
                )
                .await?;

                serde_json::to_string(&kline_data)
                    .map_err(|e| format!("Failed to serialize GMGN response: {}", e))
            }

            let action = ActionBuilder::<_, KlineDataParams, S>::new(
                "get_gmgn_kline_data",
                get_kline_data,
                None,
            )
            .description(
                "Get OHLCV kline data for a Solana token from GMGN (alternative to Birdeye). Use this if the birdeye response is empty or errored",
            )
            .parameter("token_address", "Solana token address", "string", true)
            .parameter("time_from", "Start timestamp (Unix)", "integer", true)
            .parameter("time_to", "End timestamp (Unix)", "integer", true)
            .build();

            actions.push(Arc::new(action));
        }

        Self { actions }
    }
}
