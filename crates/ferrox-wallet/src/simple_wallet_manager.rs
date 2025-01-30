use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

use solana_sdk::signature::Keypair;

use crate::{Wallet, WalletManager};

#[derive(Clone)]
pub struct SimpleWalletManager {
    wallets: Arc<Mutex<HashMap<String, Wallet>>>,
}

impl SimpleWalletManager {
    pub fn new() -> Self {
        Self {
            wallets: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl WalletManager for SimpleWalletManager {
    fn get_wallet(
        &self,
        user_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Wallet, String>> + Send + Sync>> {
        let wallet = self.wallets.lock().unwrap().get(user_id).cloned();
        if let Some(wallet) = wallet {
            return Box::pin(async move { Ok(wallet.clone()) });
        } else {
            // For test purposes, we return 1 hardcoded wallet
            let private_key = [
                103, 17, 11, 163, 113, 182, 255, 6, 9, 212, 145, 104, 9, 54, 192, 214, 170, 91, 36,
                255, 10, 225, 26, 73, 183, 136, 250, 134, 171, 24, 250, 184, 9, 247, 185, 29, 89,
                143, 75, 110, 195, 235, 251, 190, 182, 47, 42, 83, 2, 95, 187, 132, 253, 38, 244,
                162, 168, 81, 252, 6, 133, 28, 79, 228,
            ];
            return Box::pin(async move {
                Ok(Wallet::Solana(Arc::new(
                    Keypair::from_bytes(&private_key).unwrap(),
                )))
            });
        }
    }

    fn get_wallets(
        &self,
        _user_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Wallet>, String>> + Send + Sync>> {
        let wallets = self.wallets.lock().unwrap().values().cloned().collect();
        return Box::pin(async move { Ok(wallets) });
    }

    fn create_wallet(
        &self,
        user_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Wallet, String>> + Send + Sync>> {
        let keypair = Keypair::new();
        let wallet = Wallet::Solana(Arc::new(keypair));
        self.wallets
            .lock()
            .unwrap()
            .insert(user_id.to_string(), wallet.clone());
        return Box::pin(async move { Ok(wallet) });
    }
}
