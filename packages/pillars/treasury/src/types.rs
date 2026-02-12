//! Treasury Types

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Agent identifier.
pub type AgentId = String;

/// Transaction identifier.
pub type TransactionId = Uuid;

/// Monetary amount with precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Amount {
    /// Value in smallest unit (e.g., cents)
    pub value: i64,
    /// Decimal places (2 for USD, 8 for BTC)
    pub decimals: u8,
}

impl Amount {
    /// Create a new amount.
    pub fn new(value: i64, decimals: u8) -> Self {
        Self { value, decimals }
    }

    /// Create from a float (for convenience).
    pub fn from_float(value: f64, decimals: u8) -> Self {
        let multiplier = 10i64.pow(decimals as u32);
        Self {
            value: (value * multiplier as f64).round() as i64,
            decimals,
        }
    }

    /// Convert to float.
    pub fn to_float(&self) -> f64 {
        let divisor = 10i64.pow(self.decimals as u32) as f64;
        self.value as f64 / divisor
    }

    /// Convert to Decimal for precise calculations.
    pub fn to_decimal(&self) -> Decimal {
        Decimal::new(self.value, self.decimals as u32)
    }

    /// Convert to micros (smallest unit)
    /// Assumes standard 6 decimal places for internal storage if needed
    pub fn as_micros(&self) -> i64 {
        self.value
    }

    /// Check if amount is zero.
    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    /// Check if amount is negative.
    pub fn is_negative(&self) -> bool {
        self.value < 0
    }

    /// Add two amounts (must have same decimals).
    pub fn add(&self, other: &Amount) -> Option<Amount> {
        if self.decimals != other.decimals {
            return None;
        }
        Some(Amount {
            value: self.value.saturating_add(other.value),
            decimals: self.decimals,
        })
    }

    /// Subtract two amounts (must have same decimals).
    pub fn sub(&self, other: &Amount) -> Option<Amount> {
        if self.decimals != other.decimals {
            return None;
        }
        Some(Amount {
            value: self.value.saturating_sub(other.value),
            decimals: self.decimals,
        })
    }
    /// Create a zero amount with default precision (2 decimals).
    pub fn zero() -> Self {
        Self {
            value: 0,
            decimals: 2,
        }
    }

    /// Create a zero amount with specific precision.
    pub fn zero_with_decimals(decimals: u8) -> Self {
        Self { value: 0, decimals }
    }

    fn from_decimal_with_scale(value: Decimal, decimals: u8) -> Self {
        let multiplier = Decimal::from(10i64.pow(decimals as u32));
        let scaled = (value * multiplier).round();
        let raw = scaled.to_i64().unwrap_or_else(|| {
            if scaled.is_sign_negative() {
                i64::MIN
            } else {
                i64::MAX
            }
        });
        Self {
            value: raw,
            decimals,
        }
    }
}

impl std::ops::Add for Amount {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let decimals = self.decimals.max(other.decimals);
        let sum = self.to_decimal() + other.to_decimal();
        Self::from_decimal_with_scale(sum, decimals)
    }
}

impl std::ops::Sub for Amount {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let decimals = self.decimals.max(other.decimals);
        let diff = self.to_decimal() - other.to_decimal();
        Self::from_decimal_with_scale(diff, decimals)
    }
}

impl std::fmt::Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.width$}",
            self.to_float(),
            width = self.decimals as usize
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_from_float() {
        let amt = Amount::from_float(10.50, 2);
        assert_eq!(amt.value, 1050);
        assert_eq!(amt.decimals, 2);
    }

    #[test]
    fn test_amount_to_float() {
        let amt = Amount::new(1050, 2);
        assert!((amt.to_float() - 10.50).abs() < 0.001);
    }

    #[test]
    fn test_amount_add() {
        let a = Amount::new(1000, 2);
        let b = Amount::new(500, 2);
        let c = a.add(&b).unwrap();
        assert_eq!(c.value, 1500);
    }

    #[test]
    fn test_amount_sub() {
        let a = Amount::new(1000, 2);
        let b = Amount::new(300, 2);
        let c = a.sub(&b).unwrap();
        assert_eq!(c.value, 700);
    }

    #[test]
    fn test_add_with_mixed_decimals() {
        let a = Amount::new(100, 2); // 1.00
        let b = Amount::new(1, 1); // 0.1
        let c = a + b;
        assert_eq!(c.decimals, 2);
        assert_eq!(c.value, 110);
    }

    #[test]
    fn test_sub_with_mixed_decimals() {
        let a = Amount::new(110, 2); // 1.10
        let b = Amount::new(1, 1); // 0.1
        let c = a - b;
        assert_eq!(c.decimals, 2);
        assert_eq!(c.value, 100);
    }
}
