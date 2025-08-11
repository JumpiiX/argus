/*
 * Aerodrome Finance pool client implementation
 */

use async_trait::async_trait;
use ethers::{
    providers::Middleware,
    types::{Address, U256},
};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use crate::dex::{DexClient, PoolState, SwapQuote};
use crate::models::{ArgusError, Result};
use crate::rpc::RpcClient;

pub struct AerodromeClient {
    rpc: Arc<RpcClient>,
    pool_address: Address,
}

impl AerodromeClient {
    pub async fn new(rpc: Arc<RpcClient>) -> Result<Self> {
        let pool_address = Address::from_str(super::POOL_ADDRESS)
            .map_err(|e| ArgusError::ContractError(format!("Invalid pool address: {e}")))?;
        
        Ok(Self {
            rpc,
            pool_address,
        })
    }
    
    async fn get_reserves(&self) -> Result<(u128, u128)> {
        let provider = self.rpc.provider();
        let reserves_selector = ethers::utils::keccak256(b"getReserves()").to_vec();
        
        let call_data = &reserves_selector[0..4];
        
        let tx = ethers::types::TransactionRequest::new()
            .to(self.pool_address)
            .data(ethers::types::Bytes::from(call_data.to_vec()));
        
        let result = provider.call(&tx.into(), None).await
            .map_err(|e| ArgusError::ContractError(format!("Failed to call getReserves: {e}")))?;
        
        if result.len() < 64 {
            return Err(ArgusError::ContractError("Invalid reserves response - insufficient data".to_string()));
        }
        
        let reserve0 = U256::from_big_endian(&result[0..32]);
        let reserve1 = U256::from_big_endian(&result[32..64]);
        
        let r0 = reserve0.as_u128();
        let r1 = reserve1.as_u128();
        
        if r0 == 0 || r1 == 0 {
            return Err(ArgusError::ContractError("Pool has no liquidity".to_string()));
        }
        
        Ok((r0, r1))
    }
}

#[async_trait]
impl DexClient for AerodromeClient {
    async fn get_pool_state(&self) -> Result<PoolState> {
        let (reserve0, reserve1) = self.get_reserves().await?;
        
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let sqrt_price = ((reserve1 as f64 / reserve0 as f64).sqrt() * (1u128 << 96) as f64) as u128;
        
        Ok(PoolState {
            sqrt_price_x96: sqrt_price,
            tick: 0,
            liquidity: reserve0.saturating_add(reserve1) / 2,
            fee: 30,
        })
    }
    
    async fn calculate_swap_output(&self, amount_in: Decimal, zero_for_one: bool) -> Result<SwapQuote> {
        let (reserve0, reserve1) = self.get_reserves().await?;
        
        let amount_in_wei = (amount_in * Decimal::from_str("1000000000000000000")
            .map_err(|e| ArgusError::CalculationError(format!("Decimal conversion error: {e}")))?)
            .round_dp(0).to_string().parse::<u128>()
            .map_err(|e| ArgusError::CalculationError(format!("Failed to parse amount: {e}")))?;
        
        let (reserve_in, reserve_out) = if zero_for_one {
            (reserve0, reserve1)
        } else {
            (reserve1, reserve0)
        };
        
        let amount_out = self.get_amount_out(amount_in_wei, reserve_in, reserve_out)?;
        
        let amount_out_decimal = Decimal::from(amount_out) / Decimal::from_str("1000000")
            .map_err(|e| ArgusError::CalculationError(format!("Decimal conversion error: {e}")))?;
        
        let spot_price = Decimal::from(reserve1) / Decimal::from(reserve0) * Decimal::from_str("1000000000000")
            .map_err(|e| ArgusError::CalculationError(format!("Decimal conversion error: {e}")))?;
        
        let effective_price = if amount_in > Decimal::ZERO {
            amount_out_decimal / amount_in
        } else {
            Decimal::from(3000)
        };
        let price_impact = crate::utils::calculate_price_impact(amount_in, amount_out_decimal, spot_price);
        
        Ok(SwapQuote {
            amount_out: amount_out_decimal,
            effective_price,
            price_impact,
            gas_estimate: 80000,
        })
    }
    
    async fn estimate_gas(&self) -> Result<u64> {
        Ok(80000)
    }
}

impl AerodromeClient {
    fn get_amount_out(&self, amount_in: u128, reserve_in: u128, reserve_out: u128) -> Result<u128> {
        if reserve_in == 0 || reserve_out == 0 {
            return Err(ArgusError::CalculationError("Insufficient liquidity".to_string()));
        }
        
        let amount_in_with_fee = amount_in * 9999 / 10000;
        
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in + amount_in_with_fee;
        
        if denominator == 0 {
            return Err(ArgusError::CalculationError("Division by zero".to_string()));
        }
        
        Ok(numerator / denominator)
    }
}

