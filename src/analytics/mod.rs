/*
 * Analytics engine for arbitrage detection and calculation
 */

use crate::dex::SwapQuote;
use crate::models::{ArbitrageSummary, ArgusError, DexDetails, RecommendedAction, Result};
use rust_decimal::Decimal;
use std::str::FromStr;

pub struct ArbitrageAnalyzer {
    eth_price_usd: Decimal,
}

impl Default for ArbitrageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ArbitrageAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            eth_price_usd: Decimal::ZERO,
        }
    }

    pub fn update_eth_price(&mut self, price: Decimal) {
        self.eth_price_usd = price;
    }

    pub fn analyze_opportunity_with_gas(
        &self,
        uniswap_quote: &SwapQuote,
        aerodrome_quote: &SwapQuote,
        trade_size_eth: Decimal,
        _cex_price: Decimal,
        eth_gas_cost_usd: Decimal,
        base_gas_cost_usd: Decimal,
    ) -> Result<ArbitrageSummary> {
        let uniswap_price = uniswap_quote.effective_price;
        let aerodrome_price = aerodrome_quote.effective_price;

        let price_diff_per_eth = (uniswap_price - aerodrome_price).abs();
        let potential_profit_usd = price_diff_per_eth * trade_size_eth;

        let total_gas_cost_usd = eth_gas_cost_usd + base_gas_cost_usd;

        let net_profit_usd = potential_profit_usd - total_gas_cost_usd;

        let recommended_action = if net_profit_usd > Decimal::ZERO {
            RecommendedAction::ArbitrageDetected
        } else {
            RecommendedAction::NoArbitrage
        };

        Ok(ArbitrageSummary {
            potential_profit_usd,
            total_gas_cost_usd,
            net_profit_usd,
            recommended_action,
        })
    }

    pub fn wei_to_usd(&self, wei: u64) -> Result<Decimal> {
        let eth_amount = Decimal::from(wei)
            / Decimal::from_str("1000000000000000000")
                .map_err(|e| ArgusError::CalculationError(format!("Failed to convert wei: {e}")))?;
        Ok(eth_amount * self.eth_price_usd)
    }

    #[must_use]
    pub fn create_dex_details(&self, quote: &SwapQuote, gas_cost_usd: Decimal) -> DexDetails {
        DexDetails {
            effective_price_usd: quote.effective_price,
            price_impact_percent: quote.price_impact,
            estimated_gas_cost_usd: gas_cost_usd,
        }
    }
}
