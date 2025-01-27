use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE},
    Client,
};

const BASE_URL: &str = "https://public-api.birdeye.so";

#[derive(Debug, Clone)]
pub struct BirdeyeClient {
    api_key: String,
    client: Client,
}

impl BirdeyeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("X-API-KEY", HeaderValue::from_str(&self.api_key).unwrap());
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    async fn make_request(&self, endpoint: &str) -> Result<String, String> {
        let url = format!("{}{}", BASE_URL, endpoint);
        println!("Making request to {}", url);
        let response = self
            .client
            .get(&url)
            .headers(self.get_headers())
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            response.text().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Request failed with status: {}", response.status()))
        }
    }

    pub async fn get_token_price(&self, address: String) -> Result<String, String> {
        self.make_request(&format!("/defi/price?address={}", address))
            .await
    }

    pub async fn get_token_price_history(
        &self,
        address: String,
        resolution: String,
        time_from: Option<i64>,
        time_to: Option<i64>,
        limit: Option<i32>,
    ) -> Result<String, String> {
        let mut endpoint = format!(
            "/defi/history_price?address={}&address_type=token&type={}",
            address, resolution
        );

        if let Some(from) = time_from {
            endpoint.push_str(&format!("&time_from={}", from));
        }
        if let Some(to) = time_to {
            endpoint.push_str(&format!("&time_to={}", to));
        }
        if let Some(limit) = limit {
            endpoint.push_str(&format!("&limit={}", limit));
        }
        self.make_request(&endpoint).await
    }

    pub async fn get_multi_token_price(&self, addresses: String) -> Result<String, String> {
        self.make_request(&format!("/defi/multi_price?list_address={}", addresses))
            .await
    }

    pub async fn get_token_trending(&self, limit: Option<i32>) -> Result<String, String> {
        let mut endpoint = "/defi/token_trending".to_string();
        if let Some(limit) = limit {
            endpoint.push_str(&format!("?limit={}", limit));
        }
        self.make_request(&endpoint).await
    }

    pub async fn get_token_ohlcv(
        &self,
        address: String,
        resolution: String,
        time_from: i64,
        time_to: i64,
    ) -> Result<String, String> {
        self.make_request(&format!(
            "/defi/ohlcv?address={}&type={}&time_from={}&time_to={}",
            address, resolution, time_from, time_to
        ))
        .await
    }

    pub async fn get_pair_ohlcv(
        &self,
        pair_address: String,
        resolution: String,
        time_from: i64,
        time_to: i64,
    ) -> Result<String, String> {
        self.make_request(&format!(
            "/defi/ohlcv/pair?address={}&type={}&time_from={}&time_to={}",
            pair_address, resolution, time_from, time_to
        ))
        .await
    }

    pub async fn get_token_trades(
        &self,
        address: String,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<String, String> {
        let mut endpoint = format!("/defi/txs/token?address={}&sort_type=asc", address);
        if let Some(limit) = limit {
            endpoint.push_str(&format!("&limit={}", limit));
        }
        if let Some(offset) = offset {
            endpoint.push_str(&format!("&offset={}", offset));
        }
        self.make_request(&endpoint).await
    }

    pub async fn get_pair_trades(
        &self,
        pair_address: String,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<String, String> {
        let mut endpoint = format!("/dex/trades?address={}", pair_address);
        if let Some(limit) = limit {
            endpoint.push_str(&format!("&limit={}", limit));
        }
        if let Some(offset) = offset {
            endpoint.push_str(&format!("&offset={}", offset));
        }
        self.make_request(&endpoint).await
    }

    pub async fn get_token_overview(&self, address: String) -> Result<String, String> {
        self.make_request(&format!("/defi/token_overview?address={}", address))
            .await
    }

    pub async fn get_token_list(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<String, String> {
        let mut endpoint = "/defi/tokenList".to_string();
        let mut has_param = false;
        if let Some(limit) = limit {
            endpoint.push_str(&format!("?limit={}", limit));
            has_param = true;
        }
        if let Some(offset) = offset {
            endpoint.push_str(&format!(
                "{}offset={}",
                if has_param { "&" } else { "?" },
                offset
            ));
        }
        self.make_request(&endpoint).await
    }

    pub async fn get_token_security(&self, address: String) -> Result<String, String> {
        self.make_request(&format!("/defi/token_security?address={}", address))
            .await
    }

    pub async fn get_token_market_list(&self, address: String) -> Result<String, String> {
        self.make_request(&format!("/defi/v2/markets?address={}", address))
            .await
    }

    pub async fn get_token_new_listing(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<String, String> {
        let mut endpoint = "/defi/v2/tokens/new_listing".to_string();
        let mut has_param = false;
        if let Some(limit) = limit {
            endpoint.push_str(&format!("?limit={}", limit));
            has_param = true;
        }
        if let Some(offset) = offset {
            endpoint.push_str(&format!(
                "{}offset={}",
                if has_param { "&" } else { "?" },
                offset
            ));
        }
        self.make_request(&endpoint).await
    }

    pub async fn get_token_top_traders(
        &self,
        address: String,
        limit: Option<i32>,
    ) -> Result<String, String> {
        let mut endpoint = format!("/defi/v2/tokens/top_traders?address={}", address);
        if let Some(limit) = limit {
            endpoint.push_str(&format!("&limit={}", limit));
        }
        self.make_request(&endpoint).await
    }

    // Trader endpoints
    pub async fn get_gainers_losers(&self) -> Result<String, String> {
        self.make_request("/trader/gainers-losers").await
    }

    pub async fn get_trader_txs_by_time(
        &self,
        address: String,
        time_from: i64,
        time_to: i64,
        limit: Option<i32>,
    ) -> Result<String, String> {
        let mut endpoint = format!(
            "/trader/txs/seek_by_time?address={}&from={}&to={}",
            address, time_from, time_to
        );
        if let Some(limit) = limit {
            endpoint.push_str(&format!("&limit={}", limit));
        }
        self.make_request(&endpoint).await
    }

    // Wallet endpoints
    pub async fn list_supported_chains(&self) -> Result<String, String> {
        self.make_request("/v1/wallet/list_supported_chain").await
    }

    pub async fn get_wallet_portfolio(
        &self,
        wallet_address: String,
        chain_id: String,
    ) -> Result<String, String> {
        self.make_request(&format!(
            "/v1/wallet/token_list?wallet={}&chain_id={}",
            wallet_address, chain_id
        ))
        .await
    }

    pub async fn get_wallet_portfolio_multichain(
        &self,
        wallet_address: String,
    ) -> Result<String, String> {
        self.make_request(&format!(
            "/v1/wallet/multichain_token_list?wallet={}",
            wallet_address
        ))
        .await
    }

    // pub async fn get_wallet_token_balance(
    //     &self,
    //     wallet_address: String,
    //     token_address: String,
    //     chain_id: String,
    // ) -> Result<String, String> {
    //     self.make_request(&format!(
    //         "/v1/wallet/token_balance?wallet={}&token_address={}&chain_id={}",
    //         wallet_address, token_address, chain_id
    //     ))
    //     .await
    // }

    pub async fn get_wallet_transaction_history(
        &self,
        wallet_address: String,
        chain_id: String,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<String, String> {
        let mut endpoint = format!(
            "/v1/wallet/tx_list?wallet={}&chain_id={}",
            wallet_address, chain_id
        );
        if let Some(limit) = limit {
            endpoint.push_str(&format!("&limit={}", limit));
        }
        if let Some(offset) = offset {
            endpoint.push_str(&format!("&offset={}", offset));
        }
        self.make_request(&endpoint).await
    }

    pub async fn get_wallet_transaction_history_multichain(
        &self,
        wallet_address: String,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<String, String> {
        let mut endpoint = format!("/v1/wallet/multichain_tx_list?wallet={}", wallet_address);
        if let Some(limit) = limit {
            endpoint.push_str(&format!("&limit={}", limit));
        }
        if let Some(offset) = offset {
            endpoint.push_str(&format!("&offset={}", offset));
        }
        self.make_request(&endpoint).await
    }

    pub async fn simulate_transaction(
        &self,
        chain_id: String,
        tx_data: String,
    ) -> Result<String, String> {
        self.make_request(&format!(
            "/v1/wallet/simulate?chain_id={}&tx_data={}",
            chain_id, tx_data
        ))
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_client() -> BirdeyeClient {
        let api_key = std::env::var("BIRDEYE_API_KEY")
            .expect("BIRDEYE_API_KEY must be set in .env for tests");
        BirdeyeClient::new(api_key)
    }

    const SOL_ADDRESS: &str = "So11111111111111111111111111111111111111112";
    const USDC_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    const TEST_WALLET: &str = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"; // Example Solana wallet
    const TEST_CHAIN_ID: &str = "solana";

    #[tokio::test]
    async fn test_get_token_price() {
        let client = setup_client();
        let result = client.get_token_price(SOL_ADDRESS.to_string()).await;
        println!("Token price result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_token_price_history() {
        let client = setup_client();
        let result = client
            .get_token_price_history(
                SOL_ADDRESS.to_string(),
                "15m".to_string(),
                Some(1677652288),
                Some(1677738688),
                Some(100),
            )
            .await;
        println!("Price history result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_multi_token_price() {
        let client = setup_client();
        let addresses = format!("{},{}", SOL_ADDRESS, USDC_ADDRESS);
        let result = client.get_multi_token_price(addresses).await;
        println!("Multi token price result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_token_ohlcv() {
        let client = setup_client();
        let result = client
            .get_token_ohlcv(
                SOL_ADDRESS.to_string(),
                "1D".to_string(),
                1677652288,
                1677738688,
            )
            .await;
        println!("OHLCV result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_pair_ohlcv() {
        let client = setup_client();
        let result = client
            .get_pair_ohlcv(
                "8HoQnePLqPj4M7PUDzfw8e3Ymdwgc7NLGnaTUapubyvu".to_string(), // SOL/USDC pair
                "1D".to_string(),
                1677652288,
                1677738688,
            )
            .await;
        println!("Pair OHLCV result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_token_trades() {
        let client = setup_client();
        let result = client
            .get_token_trades(SOL_ADDRESS.to_string(), Some(10), Some(0))
            .await;
        println!("Token trades result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_pair_trades() {
        let client = setup_client();
        let result = client
            .get_pair_trades(
                "8HoQnePLqPj4M7PUDzfw8e3Ymdwgc7NLGnaTUapubyvu".to_string(),
                Some(10),
                Some(0),
            )
            .await;
        println!("Pair trades result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_token_overview() {
        let client = setup_client();
        let result = client.get_token_overview(SOL_ADDRESS.to_string()).await;
        println!("Token overview result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_token_list() {
        let client = setup_client();
        let result = client.get_token_list(Some(10), Some(0)).await;
        println!("Token list result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_token_security() {
        let client = setup_client();
        let result = client.get_token_security(SOL_ADDRESS.to_string()).await;
        println!("Token security result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_token_market_list() {
        let client = setup_client();
        let result = client.get_token_market_list(SOL_ADDRESS.to_string()).await;
        println!("Market list result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_token_new_listing() {
        let client = setup_client();
        let result = client.get_token_new_listing(Some(10), Some(0)).await;
        println!("New listing result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_token_top_traders() {
        let client = setup_client();
        let result = client
            .get_token_top_traders(SOL_ADDRESS.to_string(), Some(10))
            .await;
        println!("Top traders result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_token_trending() {
        let client = setup_client();
        let result = client.get_token_trending(Some(10)).await;
        println!("Trending result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_gainers_losers() {
        let client = setup_client();
        let result = client.get_gainers_losers().await;
        println!("Gainers/Losers result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_trader_txs_by_time() {
        let client = setup_client();
        let result = client
            .get_trader_txs_by_time(SOL_ADDRESS.to_string(), 1677652288, 1677738688, Some(10))
            .await;
        println!("Trader txs result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_supported_chains() {
        let client = setup_client();
        let result = client.list_supported_chains().await;
        println!("Supported chains result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_wallet_portfolio() {
        let client = setup_client();
        let result = client
            .get_wallet_portfolio(TEST_WALLET.to_string(), TEST_CHAIN_ID.to_string())
            .await;
        println!("Wallet portfolio result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_wallet_portfolio_multichain() {
        let client = setup_client();
        let result = client
            .get_wallet_portfolio_multichain(TEST_WALLET.to_string())
            .await;
        println!("Multichain portfolio result: {:?}", result);
        assert!(result.is_ok());
    }

    // #[tokio::test]
    // async fn test_get_wallet_token_balance() {
    //     let client = setup_client();
    //     let result = client
    //         .get_wallet_token_balance(
    //             TEST_WALLET.to_string(),
    //             SOL_ADDRESS.to_string(),
    //             TEST_CHAIN_ID.to_string(),
    //         )
    //         .await;
    //     println!("Token balance result: {:?}", result);
    //     assert!(result.is_ok());
    // }

    #[tokio::test]
    async fn test_get_wallet_transaction_history() {
        let client = setup_client();
        let result = client
            .get_wallet_transaction_history(
                TEST_WALLET.to_string(),
                TEST_CHAIN_ID.to_string(),
                Some(10),
                Some(0),
            )
            .await;
        println!("Transaction history result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_wallet_transaction_history_multichain() {
        let client = setup_client();
        let result = client
            .get_wallet_transaction_history_multichain(TEST_WALLET.to_string(), Some(10), Some(0))
            .await;
        println!("Multichain transaction history result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_simulate_transaction() {
        let client = setup_client();
        let result = client
            .simulate_transaction(
                TEST_CHAIN_ID.to_string(),
                "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDBXsgXgYAAAAAAAA".to_string(),
            )
            .await;
        println!("Transaction simulation result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_error_handling() {
        let client = BirdeyeClient::new("invalid-api-key".to_string());
        let result = client.get_token_price(SOL_ADDRESS.to_string()).await;
        assert!(result.is_err());
    }
}
