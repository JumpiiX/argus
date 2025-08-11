/*
 * Coinbase CEX client implementation
 */

use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use rust_decimal::Decimal;
use std::str::FromStr;
use crate::cex::CexClient;
use crate::models::{ArgusError, CexPrice, Result};

pub struct CoinbaseClient {
    client: Client,
}


impl Default for CoinbaseClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CoinbaseClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl CexClient for CoinbaseClient {
    async fn get_spot_price(&self, base: &str, quote: &str) -> Result<CexPrice> {
        let url = format!(
            "https://api.coinbase.com/v2/exchange-rates?currency={}&rates={}",
            base.to_uppercase(),
            quote.to_uppercase()
        );
        
        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        let rate_str = response["data"]["rates"][quote.to_uppercase()]
            .as_str()
            .ok_or_else(|| ArgusError::CexApiError("Failed to parse Coinbase response".to_string()))?;
        
        let price = Decimal::from_str(rate_str)
            .map_err(|e| ArgusError::CexApiError(format!("Failed to parse price: {e}")))?;
        
        Ok(CexPrice {
            exchange: "Coinbase".to_string(),
            pair: format!("{}/{}", base.to_uppercase(), quote.to_uppercase()),
            price,
            timestamp: Utc::now(),
        })
    }
}