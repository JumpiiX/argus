/*
 * Main arbitrage service that coordinates all components
 */

use chrono::Utc;
use rust_decimal::Decimal;
use std::sync::Arc;
use std::str::FromStr;
use tracing::info;
use std::sync::Mutex;
use crate::{
    analytics::ArbitrageAnalyzer,
    cex::{CexClient, create_cex_client},
    config::Config,
    dex::{DexClient, SwapQuote},
    models::{ArbitrageOpportunity, Result},
    rpc::RpcClient,
};

pub struct ArbitrageService {
    eth_rpc: Arc<RpcClient>,
    base_rpc: Arc<RpcClient>,
    cex_client: Arc<dyn CexClient>,
    uniswap_client: Arc<dyn DexClient>,
    aerodrome_client: Arc<dyn DexClient>,
    analyzer: Arc<Mutex<ArbitrageAnalyzer>>,
}

impl ArbitrageService {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Arbitrage Service");
        
        let eth_rpc = Arc::new(
            RpcClient::new(&config.ethereum.rpc_url, config.ethereum.chain_id).await?
        );
        info!("Connected to Ethereum RPC");
        
        let base_rpc = Arc::new(
            RpcClient::new(&config.base.rpc_url, config.base.chain_id).await?
        );
        info!("Connected to Base RPC");
        
        let cex_client: Arc<dyn CexClient> = Arc::from(create_cex_client(&config.cex.provider));
        info!("CEX client initialized");
        
        let uniswap_client = Arc::new(
            crate::dex::uniswap_v4::UniswapV4Client::new(eth_rpc.clone()).await?
        );
        info!("Uniswap V4 client initialized");
        
        let aerodrome_client = Arc::new(
            crate::dex::aerodrome::AerodromeClient::new(base_rpc.clone()).await?
        );
        info!("Aerodrome client initialized");
        
        Ok(Self {
            eth_rpc,
            base_rpc,
            cex_client,
            uniswap_client,
            aerodrome_client,
            analyzer: Arc::new(Mutex::new(ArbitrageAnalyzer::new())),
        })
    }
    
    pub async fn check_arbitrage_opportunity(&self, trade_size_eth: Decimal) -> Result<ArbitrageOpportunity> {
        info!("Checking arbitrage opportunity for {} ETH", trade_size_eth);
        
        let (cex_price, uniswap_quote, aerodrome_quote) = tokio::try_join!(
            self.fetch_cex_price(),
            self.get_uniswap_quote(trade_size_eth),
            self.get_aerodrome_quote(trade_size_eth)
        )?;
        
        self.analyzer.lock().unwrap().update_eth_price(cex_price.price);
        
        let uniswap_swap_calldata = self.build_uniswap_swap_calldata(trade_size_eth);
        let aerodrome_swap_calldata = self.build_aerodrome_swap_calldata(trade_size_eth);
        
        let eth_gas_cost_usd = self.estimate_gas_usd_eth_swap(
            uniswap_swap_calldata.clone(),
            cex_price.price
        ).await?;
        
        let base_gas_cost_usd = self.estimate_gas_usd_base_swap(
            aerodrome_swap_calldata.clone(),
            cex_price.price
        ).await?;
        
        info!("Gas cost in USD - ETH: ${:.4}, Base total: ${:.4}", 
              eth_gas_cost_usd, base_gas_cost_usd);
        
        let analyzer = self.analyzer.lock().unwrap();
        let arbitrage_summary = analyzer.analyze_opportunity_with_gas(
            &uniswap_quote,
            &aerodrome_quote,
            trade_size_eth,
            cex_price.price,
            eth_gas_cost_usd,
            base_gas_cost_usd,
        )?;
        
        let opportunity = ArbitrageOpportunity {
            timestamp_utc: Utc::now(),
            trade_size_eth,
            reference_cex_price_usd: cex_price.price,
            uniswap_v4_details: analyzer.create_dex_details(&uniswap_quote, eth_gas_cost_usd),
            aerodrome_details: analyzer.create_dex_details(&aerodrome_quote, base_gas_cost_usd),
            arbitrage_summary,
        };
        
        info!("Arbitrage check completed: {:?}", opportunity.arbitrage_summary.recommended_action);
        
        Ok(opportunity)
    }
    
    async fn fetch_cex_price(&self) -> Result<crate::models::CexPrice> {
        self.cex_client.get_spot_price("ETH", "USDC").await
    }
    
    async fn get_uniswap_quote(&self, amount_eth: Decimal) -> Result<SwapQuote> {
        self.uniswap_client.calculate_swap_output(amount_eth, true).await
    }
    
    async fn get_aerodrome_quote(&self, amount_eth: Decimal) -> Result<SwapQuote> {
        self.aerodrome_client.calculate_swap_output(amount_eth, true).await
    }
    
    fn build_uniswap_swap_calldata(&self, _trade_size_eth: Decimal) -> Vec<u8> {
        
        let mut calldata = Vec::new();
        calldata.extend_from_slice(&[0x12, 0x34, 0x56, 0x78]);
        calldata.extend_from_slice(&[0xAA; 200]);
        
        calldata
    }
    
    fn build_aerodrome_swap_calldata(&self, _trade_size_eth: Decimal) -> Vec<u8> {
        
        let mut calldata = Vec::new();
        calldata.extend_from_slice(&[0x87, 0x65, 0x43, 0x21]);
        calldata.extend_from_slice(&[0xBB; 180]);
        
        calldata
    }
    
    async fn estimate_gas_usd_eth_swap(&self, _calldata: Vec<u8>, eth_price_usd: Decimal) -> Result<Decimal> {
        let latest_block = self.eth_rpc.get_latest_block().await?;
        let base_fee_per_gas = latest_block.base_fee_per_gas
            .ok_or_else(|| crate::models::ArgusError::RpcError("Cannot get base fee from RPC".to_string()))?;
        
        let priority_fee = ethers::types::U256::from(
            self.eth_rpc.get_max_priority_fee_per_gas().await?
        );
        
        let gas_price_wei = base_fee_per_gas + priority_fee;

        let gas_estimate_raw = self.eth_rpc.get_typical_swap_gas().await?;
        
        let gas_with_buffer = ethers::types::U256::from(gas_estimate_raw) * 110 / 100;
        
        let cost_wei: ethers::types::U256 = gas_with_buffer * gas_price_wei;
        
        let cost_eth = Decimal::from_str(&cost_wei.to_string())
            .map_err(|e| crate::models::ArgusError::CalculationError(format!("U256 conversion error: {e}")))?
            / Decimal::from_str("1000000000000000000").unwrap();
        
        let cost_usd = cost_eth * eth_price_usd;
        
        #[allow(clippy::cast_precision_loss)]
        info!("ETH swap: raw_gas={}, buffered_gas={}, gas_price={:.3} gwei, cost=${:.4}", 
              gas_estimate_raw, gas_with_buffer, gas_price_wei.as_u128() as f64 / 1e9, cost_usd);
        
        Ok(cost_usd)
    }
    
    async fn estimate_gas_usd_base_swap(&self, calldata: Vec<u8>, eth_price_usd: Decimal) -> Result<Decimal> {
        let latest_block = self.base_rpc.get_latest_block().await?;
        let base_fee_per_gas = latest_block.base_fee_per_gas
            .ok_or_else(|| crate::models::ArgusError::RpcError("Cannot get base fee from Base RPC".to_string()))?;
        
        let priority_fee = ethers::types::U256::from(
            self.base_rpc.get_max_priority_fee_per_gas().await?
        );

        let l2_gas_price_wei = base_fee_per_gas + priority_fee;

        let l2_gas_estimate_raw = self.base_rpc.get_typical_swap_gas().await?;
        
        let l2_gas_with_buffer = ethers::types::U256::from(l2_gas_estimate_raw) * 110 / 100;
        
        let l2_cost_wei: ethers::types::U256 = l2_gas_with_buffer * l2_gas_price_wei;

        let dummy_address: ethers::types::Address = ethers::types::Address::zero();
        let l1_data_fee_wei: ethers::types::U256 = ethers::types::U256::from(
            self.base_rpc.estimate_l1_data_fee(dummy_address, calldata).await?
        );
        
        let total_cost_wei: ethers::types::U256 = l2_cost_wei + l1_data_fee_wei;
        
        let total_cost_eth = Decimal::from_str(&total_cost_wei.to_string())
            .map_err(|e| crate::models::ArgusError::CalculationError(format!("U256 conversion error: {e}")))?
            / Decimal::from_str("1000000000000000000").unwrap();
        
        let l2_cost_eth = Decimal::from_str(&l2_cost_wei.to_string()).unwrap()
            / Decimal::from_str("1000000000000000000").unwrap();
        let l1_data_fee_eth = Decimal::from_str(&l1_data_fee_wei.to_string()).unwrap()
            / Decimal::from_str("1000000000000000000").unwrap();
        
        let total_cost_usd = total_cost_eth * eth_price_usd;
        
        #[allow(clippy::cast_precision_loss)]
        info!("Base swap: l2_raw_gas={}, l2_buffered={}, l2_price={:.3} gwei, l2_cost={:.6} ETH, l1_fee={:.6} ETH, total=${:.4}", 
              l2_gas_estimate_raw, l2_gas_with_buffer, l2_gas_price_wei.as_u128() as f64 / 1e9, 
              l2_cost_eth, l1_data_fee_eth, total_cost_usd);
        
        Ok(total_cost_usd)
    }
}