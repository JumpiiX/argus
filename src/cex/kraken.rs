/*
 * Kraken CEX client implementation
 */

use crate::cex::CexClient;
use crate::models::{ArgusError, CexPrice, Result};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

pub struct KrakenClient {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct KrakenResponse {
    error: Vec<String>,
    result: Option<HashMap<String, KrakenTicker>>,
}

#[derive(Debug, Deserialize)]
struct KrakenTicker {
    c: Vec<String>,
}

impl Default for KrakenClient {
    fn default() -> Self {
        Self::new()
    }
}

impl KrakenClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn format_pair(base: &str, quote: &str) -> String {
        let base_formatted = if base.to_uppercase() == "ETH" {
            "ETH".to_string()
        } else {
            base.to_uppercase()
        };
        let quote_formatted = if quote.to_uppercase() == "USDC" {
            "USDC".to_string()
        } else {
            quote.to_uppercase()
        };
        format!("{base_formatted}{quote_formatted}")
    }
}

#[async_trait]
impl CexClient for KrakenClient {
    async fn get_spot_price(&self, base: &str, quote: &str) -> Result<CexPrice> {
        let pair = Self::format_pair(base, quote);
        let url = format!("https://api.kraken.com/0/public/Ticker?pair={pair}");

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<KrakenResponse>()
            .await
            .map_err(|e| {
                ArgusError::CexApiError(format!("Failed to parse Kraken response: {e}"))
            })?;

        if !response.error.is_empty() {
            return Err(ArgusError::CexApiError(format!(
                "Kraken API error: {:?}",
                response.error
            )));
        }

        let result = response
            .result
            .ok_or_else(|| ArgusError::CexApiError("No result in Kraken response".to_string()))?;

        let ticker = result
            .values()
            .next()
            .ok_or_else(|| ArgusError::CexApiError("No ticker data found".to_string()))?;

        let price_str = ticker
            .c
            .first()
            .ok_or_else(|| ArgusError::CexApiError("No price data found".to_string()))?;

        let price = Decimal::from_str(price_str)
            .map_err(|e| ArgusError::CexApiError(format!("Failed to parse price: {e}")))?;

        Ok(CexPrice {
            exchange: "Kraken".to_string(),
            pair: format!("{}/{}", base.to_uppercase(), quote.to_uppercase()),
            price,
            timestamp: Utc::now(),
        })
    }
}
