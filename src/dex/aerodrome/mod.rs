/*
 * Aerodrome Finance integration module
 */

mod pool;

pub use pool::AerodromeClient;

pub const POOL_ADDRESS: &str = "0xcDAC0d6c6C59727a65F871236188350531885C43";
pub const WETH_ADDRESS: &str = "0x4200000000000000000000000000000000000006";
pub const USDC_ADDRESS: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";
