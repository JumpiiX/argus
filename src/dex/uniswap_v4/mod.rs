/*
 * Uniswap V4 integration module
 */

mod pool;
mod types;

pub use pool::UniswapV4Client;
pub use types::PoolKey;

pub const POOL_MANAGER_ADDRESS: &str = "0x000000000004444c5dc75cb358380d2e3de08a90";
pub const STATE_VIEW_ADDRESS: &str = "0x7ffe42c4a5deea5b0fec41c94c136cf115597227";
pub const WETH_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
pub const USDC_ADDRESS: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
