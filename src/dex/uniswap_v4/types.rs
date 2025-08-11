/*
 * Uniswap V4 types and structures
 */

use ethers::types::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolKey {
    pub currency0: Address,
    pub currency1: Address,
    pub fee: u32,
    pub tick_spacing: i32,
    pub hooks: Address,
}

impl PoolKey {
    #[must_use]
    pub fn new_weth_usdc() -> Self {
        // IMPORTANT: Uniswap V4 supports native ETH directly (first version to do so!)
        // This saves ~40,000 gas per swap compared to V3 which requires WETH wrapping
        // Source: https://docs.uniswap.org/contracts/v4/overview
        // Pool initialized in tx: https://etherscan.io/tx/0x5205439b7e71dfe27d0911a0b05c0380e481ae83bed1ec7025513be0e3eaecb7
        // We use Address::zero() for native ETH instead of WETH contract address
        let eth: Address = Address::zero(); // Native ETH (0x0000...)
        let usdc: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
            .parse()
            .unwrap();

        Self {
            currency0: eth,
            currency1: usdc,
            fee: 500,
            tick_spacing: 10,
            hooks: Address::zero(),
        }
    }

    #[must_use]
    pub fn to_id(&self) -> [u8; 32] {
        use ethers::abi::encode;
        use ethers::utils::keccak256;

        let encoded = encode(&[
            ethers::abi::Token::Address(self.currency0),
            ethers::abi::Token::Address(self.currency1),
            ethers::abi::Token::Uint(self.fee.into()),
            ethers::abi::Token::Int(self.tick_spacing.into()),
            ethers::abi::Token::Address(self.hooks),
        ]);

        let mut pool_id = [0u8; 32];
        pool_id.copy_from_slice(&keccak256(encoded));
        pool_id
    }
}
