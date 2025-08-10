/*
 * DEX integration module for Uniswap V4 and Aerodrome
 */

pub mod uniswap_v4;
pub mod aerodrome;

use async_trait::async_trait;
use rust_decimal::Decimal;
use crate::models::Result;

#[async_trait]
pub trait DexClient: Send + Sync {
    async fn get_pool_state(&self) -> Result<PoolState>;
    async fn calculate_swap_output(&self, amount_in: Decimal, zero_for_one: bool) -> Result<SwapQuote>;
    async fn estimate_gas(&self) -> Result<u64>;
}

#[derive(Debug, Clone)]
pub struct PoolState {
    pub sqrt_price_x96: u128,
    pub tick: i32,
    pub liquidity: u128,
    pub fee: u32,
}

#[derive(Debug, Clone)]
pub struct SwapQuote {
    pub amount_out: Decimal,
    pub effective_price: Decimal,
    pub price_impact: Decimal,
    pub gas_estimate: u64,
}