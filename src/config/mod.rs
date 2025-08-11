/*
 * Configuration management for the Argus service
 */

use crate::models::{ArgusError, Result};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub ethereum: ChainConfig,
    pub base: ChainConfig,
    pub cex: CexConfig,
    pub trading: TradingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub log_level: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainConfig {
    pub rpc_url: String,
    pub chain_id: u64,
    pub gas_price_multiplier: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CexConfig {
    pub provider: CexProvider,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CexProvider {
    Coinbase,
    Kraken,
    Binance,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradingConfig {
    pub default_trade_size_eth: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        Ok(Config {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .map_err(|e| ArgusError::ConfigError(format!("Invalid port: {e}")))?,
                log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            },
            ethereum: ChainConfig {
                rpc_url: env::var("ETHEREUM_RPC_URL")
                    .map_err(|_| ArgusError::ConfigError("ETHEREUM_RPC_URL not set".to_string()))?,
                chain_id: 1,
                gas_price_multiplier: 1.1, // Hardcoded 10% buffer
            },
            base: ChainConfig {
                rpc_url: env::var("BASE_RPC_URL")
                    .map_err(|_| ArgusError::ConfigError("BASE_RPC_URL not set".to_string()))?,
                chain_id: 8453,
                gas_price_multiplier: 1.1, // Hardcoded 10% buffer
            },
            cex: CexConfig {
                provider: env::var("CEX_PROVIDER")
                    .unwrap_or_else(|_| "coinbase".to_string())
                    .parse()
                    .unwrap_or(CexProvider::Coinbase),
            },
            trading: TradingConfig {
                default_trade_size_eth: "10".to_string(), // Hardcoded default
            },
        })
    }
}

impl std::str::FromStr for CexProvider {
    type Err = ArgusError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "coinbase" => Ok(CexProvider::Coinbase),
            "kraken" => Ok(CexProvider::Kraken),
            "binance" => Ok(CexProvider::Binance),
            _ => Err(ArgusError::ConfigError(format!(
                "Unknown CEX provider: {s}"
            ))),
        }
    }
}
