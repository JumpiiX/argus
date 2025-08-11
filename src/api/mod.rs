/*
 * REST API module for the arbitrage monitoring service
 */

use rocket::{State, get, routes};
use rocket::serde::json::Json;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::models::ArbitrageOpportunity;
use crate::config::Config;

pub struct ApiState {
    pub config: Config,
    pub arbitrage_service: Arc<RwLock<crate::ArbitrageService>>,
}

#[get("/api/v1/arbitrage-opportunity?<trade_size_eth>")]
pub async fn get_arbitrage_opportunity(
    trade_size_eth: Option<String>,
    state: &State<ApiState>,
) -> std::result::Result<Json<ArbitrageOpportunity>, rocket::response::status::Custom<String>> {
    let trade_size = match trade_size_eth {
        Some(size) => Decimal::from_str(&size)
            .map_err(|e| rocket::response::status::Custom(
                rocket::http::Status::BadRequest,
                format!("Invalid trade size: {e}")
            ))?,
        None => Decimal::from_str(&state.config.trading.default_trade_size_eth)
            .map_err(|e| rocket::response::status::Custom(
                rocket::http::Status::InternalServerError,
                format!("Invalid default trade size: {e}")
            ))?,
    };
    
    let service = state.arbitrage_service.read().await;
    let opportunity = service.check_arbitrage_opportunity(trade_size).await
        .map_err(|e| {
            eprintln!("Error checking arbitrage opportunity: {e:?}");
            rocket::response::status::Custom(
                rocket::http::Status::InternalServerError,
                format!("Error checking arbitrage: {e}")
            )
        })?;
    
    Ok(Json(opportunity))
}

#[must_use]
pub fn create_rocket(state: ApiState) -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .manage(state)
        .mount("/", routes![get_arbitrage_opportunity, health_check])
}

#[get("/health")]
pub async fn health_check() -> &'static str {
    "OK"
}