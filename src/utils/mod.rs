/*
 * Utility functions and helpers
 */

use num_bigint::BigUint;
use rust_decimal::Decimal;
use std::str::FromStr;
use crate::models::{ArgusError, Result};

pub fn sqrt_price_x96_to_price(sqrt_price_x96: u128, decimals0: u8, decimals1: u8) -> Result<Decimal> {
    if sqrt_price_x96 == 0 {
        return Err(ArgusError::CalculationError("Invalid sqrt price: zero".to_string()));
    }
    
    let q96 = BigUint::from(2u128).pow(96);
    let sqrt_price = BigUint::from(sqrt_price_x96);
    let price_x192 = sqrt_price.pow(2u32);
    let price_x96 = price_x192 / &q96;
    
    if price_x96 == BigUint::from(0u128) {
        return Err(ArgusError::CalculationError("Price calculation resulted in zero".to_string()));
    }
    
    let decimal_adjustment = if decimals1 >= decimals0 {
        10_i32.pow(u32::from(decimals1 - decimals0))
    } else {
        10_i32.pow(u32::from(decimals0 - decimals1))
    };
    let price_str = (price_x96 / &q96).to_string();
    
    let price = Decimal::from_str(&price_str)
        .map_err(|e| ArgusError::CalculationError(format!("Failed to parse price: {e}")))?;
    
    if decimals1 > decimals0 {
        Ok(price * Decimal::from(decimal_adjustment))
    } else {
        Ok(price / Decimal::from(decimal_adjustment))
    }
}

#[must_use]
pub fn calculate_price_impact(
    amount_in: Decimal,
    amount_out: Decimal,
    spot_price: Decimal,
) -> Decimal {
    let expected_out = amount_in * spot_price;
    if expected_out == Decimal::ZERO {
        return Decimal::ZERO;
    }
    let impact = (expected_out - amount_out) / expected_out * Decimal::from(100);
    impact
}

pub fn format_address(address: &str) -> Result<String> {
    if !address.starts_with("0x") || address.len() != 42 {
        return Err(ArgusError::ConfigError(format!("Invalid address format: {address}")));
    }
    Ok(address.to_lowercase())
}