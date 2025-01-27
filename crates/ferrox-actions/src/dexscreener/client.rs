use reqwest::Client;

const BASE_URL: &str = "https://api.dexscreener.com";

#[derive(Debug, Clone)]
pub struct DexScreenerClient {
    client: Client,
}

impl DexScreenerClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn make_request(&self, endpoint: &str) -> Result<String, String> {
        let url = format!("{}{}", BASE_URL, endpoint);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            response.text().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Request failed with status: {}", response.status()))
        }
    }

    pub async fn get_token_profiles(&self) -> Result<String, String> {
        self.make_request("/token-profiles/latest/v1").await
    }

    pub async fn get_token_orders(
        &self,
        chain_id: String,
        token_address: String,
    ) -> Result<String, String> {
        self.make_request(&format!("/orders/v1/{}/{}", chain_id, token_address))
            .await
    }

    pub async fn get_token_boosts(&self) -> Result<String, String> {
        self.make_request("/token-boosts/latest/v1").await
    }

    pub async fn get_token_boosts_top(&self) -> Result<String, String> {
        self.make_request("/token-boosts/top/v1").await
    }

    pub async fn get_token_pairs(
        &self,
        chain_id: String,
        token_address: String,
    ) -> Result<String, String> {
        self.make_request(&format!("/token-pairs/v1/{}/{}", chain_id, token_address))
            .await
    }

    pub async fn get_tokens(
        &self,
        chain_id: String,
        token_addresses: String,
    ) -> Result<String, String> {
        self.make_request(&format!("/tokens/v1/{}/{}", chain_id, token_addresses))
            .await
    }

    pub async fn search_pairs(&self, query: String) -> Result<String, String> {
        self.make_request(&format!("/latest/dex/search?q={}", query))
            .await
    }

    pub async fn get_pairs(&self, chain_id: String, pair_id: String) -> Result<String, String> {
        self.make_request(&format!("/latest/dex/pairs/{}/{}", chain_id, pair_id))
            .await
    }
}
