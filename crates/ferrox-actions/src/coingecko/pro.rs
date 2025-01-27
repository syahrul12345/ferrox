use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const BASE_URL: &str = "https://pro-api.coingecko.com/api/v3";

#[derive(Debug, Clone)]
pub struct CoinGeckoProClient {
    api_key: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderType {
    #[serde(rename = "market_cap_desc")]
    MarketCapDesc,
    #[serde(rename = "market_cap_asc")]
    MarketCapAsc,
    #[serde(rename = "gecko_desc")]
    GeckoDesc,
    #[serde(rename = "gecko_asc")]
    GeckoAsc,
    #[serde(rename = "volume_desc")]
    VolumeDesc,
    #[serde(rename = "volume_asc")]
    VolumeAsc,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PriceChangePercentage {
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "24h")]
    TwentyFourHours,
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "14d")]
    FourteenDays,
    #[serde(rename = "30d")]
    ThirtyDays,
    #[serde(rename = "200d")]
    TwoHundredDays,
    #[serde(rename = "1y")]
    OneYear,
}

impl CoinGeckoProClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::new();
        Self { api_key, client }
    }

    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-CG-Pro-API-Key",
            HeaderValue::from_str(&self.api_key).unwrap(),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    async fn make_request(
        &self,
        endpoint: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<String, String> {
        println!("Making coingecko request to {}", endpoint);
        let url = format!("{}{}", BASE_URL, endpoint);
        println!("URL: {} params {:?}", url, params);
        let response = self
            .client
            .get(&url)
            .headers(self.get_headers())
            .query(&params.unwrap_or_default())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        println!("Got response from {}", url);
        if response.status().is_success() {
            let text = response.text().await.map_err(|e| e.to_string())?;
            Ok(text)
        } else {
            Err(format!("Request failed with status: {}", response.status()))
        }
    }

    // Pro API specific endpoints
    pub async fn get_network_status(&self) -> Result<String, String> {
        self.make_request("/ping", None).await
    }

    pub async fn get_global_data(&self) -> Result<String, String> {
        self.make_request("/global", None).await
    }

    pub async fn get_global_defi_data(&self) -> Result<String, String> {
        self.make_request("/global/decentralized_finance_defi", None)
            .await
    }

    pub async fn get_exchanges(
        &self,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<String, String> {
        let mut params = HashMap::new();
        if let Some(per_page) = per_page {
            params.insert("per_page".to_string(), per_page.to_string());
        }
        if let Some(page) = page {
            params.insert("page".to_string(), page.to_string());
        }
        self.make_request("/exchanges", Some(params)).await
    }

    pub async fn get_exchange(&self, id: String) -> Result<String, String> {
        self.make_request(&format!("/exchanges/{}", id), None).await
    }

    pub async fn get_exchange_tickers(
        &self,
        id: String,
        coin_ids: Option<Vec<String>>,
        include_exchange_logo: Option<bool>,
        page: Option<u32>,
        depth: Option<bool>,
        order: Option<String>,
    ) -> Result<String, String> {
        let mut params = HashMap::new();
        if let Some(coin_ids) = coin_ids {
            params.insert("coin_ids".to_string(), coin_ids.join(","));
        }
        if let Some(include_exchange_logo) = include_exchange_logo {
            params.insert(
                "include_exchange_logo".to_string(),
                include_exchange_logo.to_string(),
            );
        }
        if let Some(page) = page {
            params.insert("page".to_string(), page.to_string());
        }
        if let Some(depth) = depth {
            params.insert("depth".to_string(), depth.to_string());
        }
        if let Some(order) = order {
            params.insert("order".to_string(), order);
        }
        self.make_request(&format!("/exchanges/{}/tickers", id), Some(params))
            .await
    }

    pub async fn get_exchange_volume_chart(&self, id: String, days: u32) -> Result<String, String> {
        let mut params = HashMap::new();
        params.insert("days".to_string(), days.to_string());
        self.make_request(&format!("/exchanges/{}/volume_chart", id), Some(params))
            .await
    }

    pub async fn get_coins_list(&self, include_platform: Option<bool>) -> Result<String, String> {
        let mut params = HashMap::new();
        if let Some(include_platform) = include_platform {
            params.insert("include_platform".to_string(), include_platform.to_string());
        }
        self.make_request("/coins/list", Some(params)).await
    }

    pub async fn get_coin_tickers(
        &self,
        id: String,
        exchange_ids: Option<Vec<String>>,
        include_exchange_logo: Option<bool>,
        page: Option<u32>,
        order: Option<String>,
        depth: Option<bool>,
    ) -> Result<String, String> {
        let mut params = HashMap::new();
        if let Some(exchange_ids) = exchange_ids {
            params.insert("exchange_ids".to_string(), exchange_ids.join(","));
        }
        if let Some(include_exchange_logo) = include_exchange_logo {
            params.insert(
                "include_exchange_logo".to_string(),
                include_exchange_logo.to_string(),
            );
        }
        if let Some(page) = page {
            params.insert("page".to_string(), page.to_string());
        }
        if let Some(order) = order {
            params.insert("order".to_string(), order);
        }
        if let Some(depth) = depth {
            params.insert("depth".to_string(), depth.to_string());
        }
        self.make_request(&format!("/coins/{}/tickers", id), Some(params))
            .await
    }

    pub async fn get_coin_history(
        &self,
        id: String,
        date: String,
        localization: Option<bool>,
    ) -> Result<String, String> {
        let mut params = HashMap::new();
        params.insert("date".to_string(), date);
        if let Some(localization) = localization {
            params.insert("localization".to_string(), localization.to_string());
        }
        self.make_request(&format!("/coins/{}/history", id), Some(params))
            .await
    }

    pub async fn get_coin_market_chart(
        &self,
        id: String,
        vs_currency: String,
        days: String,
        interval: Option<String>,
    ) -> Result<String, String> {
        let mut params = HashMap::new();
        params.insert("vs_currency".to_string(), vs_currency);
        params.insert("days".to_string(), days);
        if let Some(interval) = interval {
            params.insert("interval".to_string(), interval);
        }
        self.make_request(&format!("/coins/{}/market_chart", id), Some(params))
            .await
    }

    pub async fn get_coin_market_chart_range(
        &self,
        id: String,
        vs_currency: String,
        from: u64,
        to: u64,
    ) -> Result<String, String> {
        let mut params = HashMap::new();
        params.insert("vs_currency".to_string(), vs_currency);
        params.insert("from".to_string(), from.to_string());
        params.insert("to".to_string(), to.to_string());
        self.make_request(&format!("/coins/{}/market_chart/range", id), Some(params))
            .await
    }

    pub async fn get_coin_ohlc(
        &self,
        id: String,
        vs_currency: String,
        days: String,
    ) -> Result<String, String> {
        let mut params = HashMap::new();
        params.insert("vs_currency".to_string(), vs_currency);
        params.insert("days".to_string(), days);
        self.make_request(&format!("/coins/{}/ohlc", id), Some(params))
            .await
    }

    pub async fn get_coin_contract(
        &self,
        id: String,
        contract_address: String,
    ) -> Result<String, String> {
        self.make_request(
            &format!("/coins/{}/contract/{}", id, contract_address),
            None,
        )
        .await
    }

    pub async fn get_coin_contract_market_chart(
        &self,
        id: String,
        contract_address: String,
        vs_currency: String,
        days: String,
    ) -> Result<String, String> {
        let mut params = HashMap::new();
        params.insert("vs_currency".to_string(), vs_currency);
        params.insert("days".to_string(), days);
        self.make_request(
            &format!("/coins/{}/contract/{}/market_chart", id, contract_address),
            Some(params),
        )
        .await
    }

    pub async fn get_coin_contract_market_chart_range(
        &self,
        id: String,
        contract_address: String,
        vs_currency: String,
        from: u64,
        to: u64,
    ) -> Result<String, String> {
        let mut params = HashMap::new();
        params.insert("vs_currency".to_string(), vs_currency);
        params.insert("from".to_string(), from.to_string());
        params.insert("to".to_string(), to.to_string());
        self.make_request(
            &format!(
                "/coins/{}/contract/{}/market_chart/range",
                id, contract_address
            ),
            Some(params),
        )
        .await
    }

    pub async fn get_asset_platforms(&self) -> Result<String, String> {
        self.make_request("/asset_platforms", None).await
    }

    pub async fn get_coins_categories_list(&self) -> Result<String, String> {
        self.make_request("/coins/categories/list", None).await
    }

    pub async fn get_coins_categories(&self, order: Option<String>) -> Result<String, String> {
        let mut params = HashMap::new();
        if let Some(order) = order {
            params.insert("order".to_string(), order);
        }
        self.make_request("/coins/categories", Some(params)).await
    }

    pub async fn get_indexes(&self) -> Result<String, String> {
        self.make_request("/indexes", None).await
    }

    pub async fn get_indexes_list(&self) -> Result<String, String> {
        self.make_request("/indexes/list", None).await
    }

    pub async fn get_derivatives(&self) -> Result<String, String> {
        self.make_request("/derivatives", None).await
    }

    pub async fn get_derivatives_exchanges(
        &self,
        order: Option<String>,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<String, String> {
        let mut params = HashMap::new();
        if let Some(order) = order {
            params.insert("order".to_string(), order);
        }
        if let Some(per_page) = per_page {
            params.insert("per_page".to_string(), per_page.to_string());
        }
        if let Some(page) = page {
            params.insert("page".to_string(), page.to_string());
        }
        self.make_request("/derivatives/exchanges", Some(params))
            .await
    }

    pub async fn get_derivatives_exchange(
        &self,
        id: String,
        include_tickers: Option<String>,
    ) -> Result<String, String> {
        let mut params = HashMap::new();
        if let Some(include_tickers) = include_tickers {
            params.insert("include_tickers".to_string(), include_tickers);
        }
        self.make_request(&format!("/derivatives/exchanges/{}", id), Some(params))
            .await
    }

    pub async fn get_exchange_rates(&self) -> Result<String, String> {
        self.make_request("/exchange_rates", None).await
    }

    pub async fn search(&self, query: String) -> Result<String, String> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query);
        self.make_request("/search", Some(params)).await
    }

    pub async fn get_trending(&self) -> Result<String, String> {
        self.make_request("/search/trending", None).await
    }

    pub async fn get_companies_public_treasury(&self, coin_id: String) -> Result<String, String> {
        self.make_request(&format!("/companies/public_treasury/{}", coin_id), None)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_client() -> CoinGeckoProClient {
        let api_key = std::env::var("COINGECKO_PRO_API_KEY")
            .expect("COINGECKO_PRO_API_KEY must be set for tests");
        CoinGeckoProClient::new(api_key)
    }

    #[tokio::test]
    async fn test_network_status() {
        let client = get_test_client();
        let result = client.get_network_status().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_global_data() {
        let client = get_test_client();
        let result = client.get_global_data().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_global_defi_data() {
        let client = get_test_client();
        let result = client.get_global_defi_data().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_exchanges() {
        let client = get_test_client();
        let result = client.get_exchanges(Some(10), Some(1)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_exchange() {
        let client = get_test_client();
        let result = client.get_exchange("binance".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_exchange_tickers() {
        let client = get_test_client();
        let result = client
            .get_exchange_tickers(
                "binance".to_string(),
                Some(vec!["bitcoin".to_string()]),
                Some(true),
                Some(1),
                Some(true),
                Some("volume_desc".to_string()),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_exchange_volume_chart() {
        let client = get_test_client();
        let result = client
            .get_exchange_volume_chart("binance".to_string(), 1)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coins_list() {
        let client = get_test_client();
        let result = client.get_coins_list(Some(true)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coin_tickers() {
        let client = get_test_client();
        let result = client
            .get_coin_tickers(
                "bitcoin".to_string(),
                Some(vec!["binance".to_string()]),
                Some(true),
                Some(1),
                Some("volume_desc".to_string()),
                Some(true),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coin_history() {
        let client = get_test_client();
        let result = client
            .get_coin_history("bitcoin".to_string(), "30-12-2023".to_string(), Some(true))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coin_market_chart() {
        let client = get_test_client();
        let result = client
            .get_coin_market_chart(
                "bitcoin".to_string(),
                "usd".to_string(),
                "1".to_string(),
                Some("daily".to_string()),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coin_market_chart_range() {
        let client = get_test_client();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = client
            .get_coin_market_chart_range("bitcoin".to_string(), "usd".to_string(), now - 86400, now)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coin_ohlc() {
        let client = get_test_client();
        let result = client
            .get_coin_ohlc("bitcoin".to_string(), "usd".to_string(), "1".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coin_contract() {
        let client = get_test_client();
        let result = client
            .get_coin_contract(
                "ethereum".to_string(),
                "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984".to_string(), // UNI contract
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coin_contract_market_chart() {
        let client = get_test_client();
        let result = client
            .get_coin_contract_market_chart(
                "ethereum".to_string(),
                "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984".to_string(),
                "usd".to_string(),
                "1".to_string(),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coin_contract_market_chart_range() {
        let client = get_test_client();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = client
            .get_coin_contract_market_chart_range(
                "ethereum".to_string(),
                "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984".to_string(),
                "usd".to_string(),
                now - 86400,
                now,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_asset_platforms() {
        let client = get_test_client();
        let result = client.get_asset_platforms().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coins_categories_list() {
        let client = get_test_client();
        let result = client.get_coins_categories_list().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coins_categories() {
        let client = get_test_client();
        let result = client
            .get_coins_categories(Some("market_cap_desc".to_string()))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_indexes() {
        let client = get_test_client();
        let result = client.get_indexes().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_indexes_list() {
        let client = get_test_client();
        let result = client.get_indexes_list().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_derivatives() {
        let client = get_test_client();
        let result = client.get_derivatives().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_derivatives_exchanges() {
        let client = get_test_client();
        let result = client
            .get_derivatives_exchanges(Some("name_desc".to_string()), Some(10), Some(1))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_derivatives_exchange() {
        let client = get_test_client();
        let result = client
            .get_derivatives_exchange("binance_futures".to_string(), Some("all".to_string()))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_exchange_rates() {
        let client = get_test_client();
        let result = client.get_exchange_rates().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search() {
        let client = get_test_client();
        let result = client.search("bitcoin".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_trending() {
        let client = get_test_client();
        let result = client.get_trending().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_companies_public_treasury() {
        let client = get_test_client();
        let result = client
            .get_companies_public_treasury("bitcoin".to_string())
            .await;
        assert!(result.is_ok());
    }
}
