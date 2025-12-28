//! Treasury Types

use rust_decimal::Decimal;
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
}
