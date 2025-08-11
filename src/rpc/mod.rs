/*
 * RPC client module for interacting with Ethereum and Base chains
 */

use crate::models::{ArgusError, Result};
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{Address, Block, Bytes, H256, U256};
use std::sync::Arc;

pub struct RpcClient {
    provider: Arc<Provider<Http>>,
    chain_id: u64,
}

impl RpcClient {
    pub async fn new(rpc_url: &str, chain_id: u64) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| ArgusError::RpcError(format!("Failed to create provider: {e}")))?;

        let chain = provider
            .get_chainid()
            .await
            .map_err(|e| ArgusError::RpcError(format!("Failed to get chain ID: {e}")))?;

        if chain.as_u64() != chain_id {
            return Err(ArgusError::RpcError(format!(
                "Chain ID mismatch: expected {}, got {}",
                chain_id,
                chain.as_u64()
            )));
        }

        Ok(Self {
            provider: Arc::new(provider),
            chain_id,
        })
    }

    #[must_use]
    pub fn provider(&self) -> Arc<Provider<Http>> {
        self.provider.clone()
    }

    #[must_use]
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    pub async fn get_gas_price(&self) -> Result<u64> {
        let gas_price = self
            .provider
            .get_gas_price()
            .await
            .map_err(|e| ArgusError::RpcError(format!("Failed to get gas price: {e}")))?;

        Ok(gas_price.as_u64())
    }

    pub async fn get_gas_price_gwei(&self) -> Result<f64> {
        let gas_price = self.get_gas_price().await?;
        #[allow(clippy::cast_precision_loss)]
        Ok(gas_price as f64 / 1_000_000_000.0)
    }

    pub async fn estimate_gas_cost(&self, gas_units: u64) -> Result<u64> {
        let gas_price = self.get_gas_price().await?;
        Ok(gas_price * gas_units)
    }

    pub fn get_typical_swap_gas(&self) -> Result<u64> {
        // Return typical gas units for DEX swaps on each chain
        // These are well-documented values from mainnet observations
        // We use real-time gas prices with these typical units
        // Also Documented why this approach was choosen in DESIGN_CHOICES.md

        if self.chain_id == 1 {
            // Ethereum mainnet - Uniswap V4 swap typically uses 150,000 gas
            // Source: Uniswap V4 documentation and mainnet observations
            Ok(150_000)
        } else if self.chain_id == 8453 {
            // Base - Aerodrome swap typically uses 80,000 gas
            // Source: Aerodrome documentation and Base mainnet observations
            Ok(80_000)
        } else {
            Err(ArgusError::RpcError(format!(
                "Unsupported chain ID: {}",
                self.chain_id
            )))
        }
    }

    pub async fn get_latest_block(&self) -> Result<Block<H256>> {
        let block = self
            .provider
            .get_block(ethers::types::BlockNumber::Latest)
            .await
            .map_err(|e| ArgusError::RpcError(format!("Failed to get latest block: {e}")))?
            .ok_or_else(|| ArgusError::RpcError("Latest block not found".to_string()))?;
        Ok(block)
    }

    pub async fn get_max_priority_fee_per_gas(&self) -> Result<u64> {
        // Try to get suggested priority fee - NO FALLBACK
        let priority_fee = self
            .provider
            .request::<_, U256>("eth_maxPriorityFeePerGas", ())
            .await
            .map_err(|e| ArgusError::RpcError(format!("Cannot get priority fee from RPC: {e}")))?;

        Ok(priority_fee.as_u64())
    }

    pub async fn estimate_l1_data_fee(
        &self,
        _to_address: Address,
        calldata: Vec<u8>,
    ) -> Result<u64> {
        if self.chain_id != 8453 {
            return Ok(0);
        }

        let oracle_address: Address = "0x420000000000000000000000000000000000000F"
            .parse()
            .unwrap();

        let mut tx_bytes = Vec::new();

        tx_bytes.extend_from_slice(&[0x02]);
        tx_bytes.extend_from_slice(&calldata.len().to_be_bytes()[6..]);
        tx_bytes.extend_from_slice(&calldata);

        let get_l1_fee_selector = &ethers::utils::keccak256(b"getL1Fee(bytes)")[0..4];

        let encoded_params = ethers::abi::encode(&[ethers::abi::Token::Bytes(tx_bytes)]);

        let mut oracle_call_data = Vec::from(get_l1_fee_selector);
        oracle_call_data.extend_from_slice(&encoded_params);

        let tx = ethers::types::TransactionRequest::new()
            .to(oracle_address)
            .data(Bytes::from(oracle_call_data));

        let result =
            self.provider.call(&tx.into(), None).await.map_err(|e| {
                ArgusError::RpcError(format!("Failed to get L1 fee from oracle: {e}"))
            })?;

        if result.len() < 32 {
            return Err(ArgusError::RpcError(
                "Invalid L1 fee response from oracle".to_string(),
            ));
        }

        let l1_fee_wei = U256::from_big_endian(&result[0..32]);

        Ok(l1_fee_wei.as_u64())
    }
}
