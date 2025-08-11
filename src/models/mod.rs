/*
 * Data models and types for the arbitrage monitoring service
 */

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub timestamp_utc: DateTime<Utc>,
    pub trade_size_eth: Decimal,
    pub reference_cex_price_usd: Decimal,
    pub uniswap_v4_details: DexDetails,
    pub aerodrome_details: DexDetails,
    pub arbitrage_summary: ArbitrageSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexDetails {
    pub effective_price_usd: Decimal,
    pub price_impact_percent: Decimal,
    pub estimated_gas_cost_usd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageSummary {
    pub potential_profit_usd: Decimal,
    pub total_gas_cost_usd: Decimal,
    pub net_profit_usd: Decimal,
    pub recommended_action: RecommendedAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecommendedAction {
    ArbitrageDetected,
    NoArbitrage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CexPrice {
    pub exchange: String,
    pub pair: String,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum ArgusError {
    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("CEX API error: {0}")]
    CexApiError(String),

    #[error("Contract interaction error: {0}")]
    ContractError(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, ArgusError>;
