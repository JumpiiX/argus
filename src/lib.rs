/*
 * Argus - Arbitrage Monitoring Service
 * Core library exports and module declarations
 */

pub mod api;
pub mod analytics;
pub mod cex;
pub mod config;
pub mod dex;
pub mod models;
pub mod rpc;
pub mod service;
pub mod utils;

pub use config::Config;
pub use models::*;
pub use service::ArbitrageService;