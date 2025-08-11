/*
 * Argus - Arbitrage Monitoring Service
 * Core library exports and module declarations
 */

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub mod analytics;
pub mod api;
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
