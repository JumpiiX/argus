/*
 * Argus - Arbitrage Monitoring Service
 * Main entry point for the application
 */

use argus::{config::Config, api, service::ArbitrageService};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    
    info!("Starting Argus Arbitrage Monitoring Service");
    
    let config = Config::from_env()
        .map_err(|e| {
            error!("Failed to load configuration: {}", e);
            e
        })?;
    
    info!("Configuration loaded successfully");
    
    let arbitrage_service = ArbitrageService::new(config.clone()).await?;
    let arbitrage_service = Arc::new(RwLock::new(arbitrage_service));
    
    let api_state = api::ApiState {
        config: config.clone(),
        arbitrage_service,
    };
    
    info!("Starting API server on {}:{}", config.server.host, config.server.port);
    
    let rocket = api::create_rocket(api_state);
    rocket.launch().await?;
    
    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "argus=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
