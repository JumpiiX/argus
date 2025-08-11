/*
 * CEX price fetcher module for getting reference prices
 */

mod coinbase;
mod kraken;
mod binance;

use async_trait::async_trait;
use crate::config::CexProvider;
use crate::models::{CexPrice, Result};

pub use coinbase::CoinbaseClient;
pub use kraken::KrakenClient;
pub use binance::BinanceClient;

#[async_trait]
pub trait CexClient: Send + Sync {
    async fn get_spot_price(&self, base: &str, quote: &str) -> Result<CexPrice>;
}

#[must_use]
pub fn create_cex_client(provider: &CexProvider) -> Box<dyn CexClient> {
    match provider {
        CexProvider::Coinbase => Box::new(CoinbaseClient::new()),
        CexProvider::Kraken => Box::new(KrakenClient::new()),
        CexProvider::Binance => Box::new(BinanceClient::new()),
    }
}