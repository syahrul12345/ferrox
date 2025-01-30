pub mod simple_wallet_manager;
use std::{future::Future, pin::Pin, sync::Arc};

use solana_sdk::{pubkey::Pubkey, signature::Keypair};

#[derive(Clone)]
pub enum Wallet {
    Solana(Arc<Keypair>),
}

//Implement this trait to manage the wallets of multiple users
pub trait WalletManager: Send + Sync + Clone {
    // Returns a wallet for a user
    fn get_wallet(
        &self,
        user_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Wallet, String>> + Send + Sync>>;

    fn get_wallets(
        &self,
        user_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Wallet>, String>> + Send + Sync>>;

    fn create_wallet(
        &self,
        user_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Wallet, String>> + Send + Sync>>;
}
