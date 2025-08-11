/*
 * Binance CEX client implementation
 */

use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use crate::cex::CexClient;
use crate::models::{ArgusError, CexPrice, Result};

pub struct BinanceClient {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct BinanceTickerResponse {
    price: String,
}

impl Default for BinanceClient {
    fn default() -> Self {
        Self::new()
    }
}

impl BinanceClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
    
    fn format_symbol(base: &str, quote: &str) -> String {
        format!("{}{}", base.to_uppercase(), quote.to_uppercase())
    }
}

#[async_trait]
impl CexClient for BinanceClient {
    async fn get_spot_price(&self, base: &str, quote: &str) -> Result<CexPrice> {
        let symbol = Self::format_symbol(base, quote);
        let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={symbol}");
        
        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<BinanceTickerResponse>()
            .await
            .map_err(|e| ArgusError::CexApiError(format!("Failed to parse Binance response: {e}")))?;
        
        let price = Decimal::from_str(&response.price)
            .map_err(|e| ArgusError::CexApiError(format!("Failed to parse price: {e}")))?;
        
        Ok(CexPrice {
            exchange: "Binance".to_string(),
            pair: format!("{}/{}", base.to_uppercase(), quote.to_uppercase()),
            price,
            timestamp: Utc::now(),
        })
    }
}