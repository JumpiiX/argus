/*
 * CEX price fetcher module for getting reference prices
 */

mod binance;
mod coinbase;
mod kraken;

use crate::config::CexProvider;
use crate::models::{CexPrice, Result};
use async_trait::async_trait;

pub use binance::BinanceClient;
pub use coinbase::CoinbaseClient;
pub use kraken::KrakenClient;

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
