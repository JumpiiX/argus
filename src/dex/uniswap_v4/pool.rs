/*
 * Uniswap V4 pool client implementation
 */

use async_trait::async_trait;
use ethers::{
    abi::{encode, Token},
    providers::Middleware,
    types::{Address, U256},
    utils::keccak256,
};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use crate::dex::{DexClient, PoolState, SwapQuote};
use crate::models::{ArgusError, Result};
use crate::rpc::RpcClient;
use super::types::PoolKey;

pub struct UniswapV4Client {
    rpc: Arc<RpcClient>,
    state_view: Address,
    pool_key: PoolKey,
}

impl UniswapV4Client {
    pub fn new(rpc: Arc<RpcClient>) -> Result<Self> {
        let state_view = Address::from_str(super::STATE_VIEW_ADDRESS)
            .map_err(|e| ArgusError::ContractError(format!("Invalid StateView address: {e}")))?;
        
        let pool_key = PoolKey::new_weth_usdc();
        
        Ok(Self {
            rpc,
            state_view,
            pool_key,
        })
    }
    
    async fn get_pool_state_internal(&self) -> Result<PoolState> {
        let slot0_data = self.read_slot0().await?;
        let liquidity = self.read_liquidity().await?;
        
        Ok(PoolState {
            sqrt_price_x96: slot0_data.0,
            tick: slot0_data.1,
            liquidity,
            fee: slot0_data.3,
        })
    }
    
    async fn read_slot0(&self) -> Result<(u128, i32, u32, u32)> {
        let provider = self.rpc.provider();
        
        let pool_id = self.pool_key.to_id();
        let function_selector = &keccak256(b"getSlot0(bytes32)")[0..4];

        let encoded_params = encode(&[Token::FixedBytes(pool_id.to_vec())]);
        let mut call_data = Vec::from(function_selector);
        call_data.extend_from_slice(&encoded_params);
        
        let tx = ethers::types::TransactionRequest::new()
            .to(self.state_view)
            .data(ethers::types::Bytes::from(call_data));
        
        let result = provider.call(&tx.into(), None).await
            .map_err(|e| ArgusError::ContractError(format!("Failed to call getSlot0: {e}")))?;
        
        if result.len() < 128 {
            return Err(ArgusError::ContractError("Invalid slot0 response".to_string()));
        }
        
        let sqrt_price_bytes = &result[0..32];
        let tick_bytes = &result[32..64];
        let protocol_fee_bytes = &result[64..96];
        let lp_fee_bytes = &result[96..128];

        let sqrt_price = U256::from_big_endian(sqrt_price_bytes);
        let sqrt_price_u128 = sqrt_price.as_u128();

        let tick_i32 = if tick_bytes[29] >= 0x80 {
            let val = (i32::from(tick_bytes[29]) << 16) | 
                     (i32::from(tick_bytes[30]) << 8) | 
                     i32::from(tick_bytes[31]);
            #[allow(clippy::cast_possible_wrap)]
            let result = val | 0xFF00_0000_u32 as i32;
            result
        } else {
            (i32::from(tick_bytes[29]) << 16) | 
            (i32::from(tick_bytes[30]) << 8) | 
            i32::from(tick_bytes[31])
        };

        let protocol_fee_u32 = (u32::from(protocol_fee_bytes[29]) << 16) | 
                               (u32::from(protocol_fee_bytes[30]) << 8) | 
                               u32::from(protocol_fee_bytes[31]);
        let lp_fee_u32 = (u32::from(lp_fee_bytes[29]) << 16) | 
                        (u32::from(lp_fee_bytes[30]) << 8) | 
                        u32::from(lp_fee_bytes[31]);
        
        Ok((sqrt_price_u128, tick_i32, protocol_fee_u32, lp_fee_u32))
    }
    
    async fn read_liquidity(&self) -> Result<u128> {
        let provider = self.rpc.provider();
        
        let pool_id = self.pool_key.to_id();
        let function_selector = &keccak256(b"getLiquidity(bytes32)")[0..4];
        
        let encoded_params = encode(&[Token::FixedBytes(pool_id.to_vec())]);
        let mut call_data = Vec::from(function_selector);
        call_data.extend_from_slice(&encoded_params);
        
        let tx = ethers::types::TransactionRequest::new()
            .to(self.state_view)
            .data(ethers::types::Bytes::from(call_data));
        
        let result = provider.call(&tx.into(), None).await
            .map_err(|e| ArgusError::ContractError(format!("Failed to call getLiquidity: {e}")))?;
        
        if result.len() < 32 {
            return Err(ArgusError::ContractError("Invalid liquidity response".to_string()));
        }
        
        let liquidity = U256::from_big_endian(&result[0..32]);
        Ok(liquidity.as_u128())
    }
}

#[async_trait]
impl DexClient for UniswapV4Client {
    async fn get_pool_state(&self) -> Result<PoolState> {
        self.get_pool_state_internal().await
    }
    
    async fn calculate_swap_output(&self, amount_in: Decimal, zero_for_one: bool) -> Result<SwapQuote> {
        let pool_state = self.get_pool_state().await?;

        #[allow(clippy::cast_precision_loss)]
        let sqrt_price_f64 = pool_state.sqrt_price_x96 as f64 / (1u128 << 96) as f64;
        let spot_price_raw = sqrt_price_f64 * sqrt_price_f64;

        let spot_price = Decimal::try_from(spot_price_raw * 1e12)
            .unwrap_or(Decimal::from(3000));
        
        
        let price_impact_percent = if amount_in > Decimal::ZERO {
            (amount_in / Decimal::from(10)) * Decimal::from_str("0.001").unwrap()
        } else {
            Decimal::ZERO
        };

        let effective_price = if zero_for_one {
            spot_price * (Decimal::ONE - price_impact_percent)
        } else {
            spot_price * (Decimal::ONE + price_impact_percent)
        };
        
        let amount_out = if zero_for_one {
            amount_in * effective_price
        } else {
            amount_in / effective_price
        };
        
        let fee_multiplier = Decimal::ONE - (Decimal::from(pool_state.fee) / Decimal::from(1_000_000));
        let amount_out_after_fee = amount_out * fee_multiplier;
        
        Ok(SwapQuote {
            amount_out: amount_out_after_fee,
            effective_price,
            price_impact: price_impact_percent,
            gas_estimate: 150_000,
        })
    }
    
    async fn estimate_gas(&self) -> Result<u64> {
        Ok(150_000)
    }
}


