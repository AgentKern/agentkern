//! Balance Ledger for Agent Accounts
//!
//! Manages agent balances with atomic operations.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::types::{AgentId, Amount};

/// Supported currencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    /// US Dollars (2 decimals)
    USD,
    /// AgentKern Credits (internal, 6 decimals)
    VMC,
    /// Bitcoin (8 decimals)
    BTC,
    /// Ethereum (18 decimals, stored as 8)
    ETH,
    /// USDC Stablecoin (6 decimals)
    USDC,
}

impl Currency {
    /// Get decimal places for this currency.
    pub fn decimals(&self) -> u8 {
        match self {
            Currency::USD => 2,
            Currency::VMC => 6,
            Currency::BTC => 8,
            Currency::ETH => 8,
            Currency::USDC => 6,
        }
    }

    /// Create zero amount in this currency.
    pub fn zero(&self) -> Amount {
        Amount::new(0, self.decimals())
    }
}

/// Agent account balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBalance {
    /// Agent ID
    pub agent_id: AgentId,
    /// Current balance
    pub balance: Amount,
    /// Currency
    pub currency: Currency,
    /// Balance held in pending transactions
    pub pending: Amount,
    /// Last updated
    pub updated_at: DateTime<Utc>,
    /// Total deposited
    pub total_deposited: Amount,
    /// Total withdrawn
    pub total_withdrawn: Amount,
}

impl AgentBalance {
    /// Create a new zero balance account.
    pub fn new(agent_id: impl Into<AgentId>, currency: Currency) -> Self {
        let zero = currency.zero();
        Self {
            agent_id: agent_id.into(),
            balance: zero,
            currency,
            pending: zero,
            updated_at: Utc::now(),
            total_deposited: zero,
            total_withdrawn: zero,
        }
    }

    /// Get available balance (balance - pending).
    pub fn available(&self) -> Amount {
        self.balance
            .sub(&self.pending)
            .unwrap_or(self.currency.zero())
    }

    /// Check if agent can spend amount.
    pub fn can_spend(&self, amount: &Amount) -> bool {
        let available = self.available();
        available.value >= amount.value
    }
}

/// Balance ledger for all agents.
pub struct BalanceLedger {
    /// Balances by agent ID
    balances: Arc<RwLock<HashMap<AgentId, AgentBalance>>>,
    /// Default currency
    default_currency: Currency,
}

impl Default for BalanceLedger {
    fn default() -> Self {
        Self::new(Currency::VMC)
    }
}

impl BalanceLedger {
    /// Create a new balance ledger.
    pub fn new(default_currency: Currency) -> Self {
        Self {
            balances: Arc::new(RwLock::new(HashMap::new())),
            default_currency,
        }
    }

    /// Get or create balance for an agent.
    pub fn get_balance(&self, agent_id: &str) -> AgentBalance {
        let balances = self.balances.read();
        balances
            .get(agent_id)
            .cloned()
            .unwrap_or_else(|| AgentBalance::new(agent_id, self.default_currency))
    }

    /// Deposit funds to an agent's account.
    pub fn deposit(&self, agent_id: &str, amount: Amount) -> Result<AgentBalance, LedgerError> {
        if amount.is_negative() {
            return Err(LedgerError::InvalidAmount);
        }

        let mut balances = self.balances.write();
        let balance = balances
            .entry(agent_id.to_string())
            .or_insert_with(|| AgentBalance::new(agent_id, self.default_currency));

        balance.balance = balance
            .balance
            .add(&amount)
            .ok_or(LedgerError::InvalidAmount)?;
        balance.total_deposited = balance
            .total_deposited
            .add(&amount)
            .ok_or(LedgerError::InvalidAmount)?;
        balance.updated_at = Utc::now();

        Ok(balance.clone())
    }

    /// Hold funds for a pending transaction.
    pub fn hold(&self, agent_id: &str, amount: Amount) -> Result<(), LedgerError> {
        let mut balances = self.balances.write();
        let balance = balances
            .get_mut(agent_id)
            .ok_or(LedgerError::AccountNotFound)?;

        if !balance.can_spend(&amount) {
            return Err(LedgerError::InsufficientFunds);
        }

        balance.pending = balance
            .pending
            .add(&amount)
            .ok_or(LedgerError::InvalidAmount)?;
        balance.updated_at = Utc::now();

        Ok(())
    }

    /// Release held funds (cancel pending transaction).
    pub fn release(&self, agent_id: &str, amount: Amount) -> Result<(), LedgerError> {
        let mut balances = self.balances.write();
        let balance = balances
            .get_mut(agent_id)
            .ok_or(LedgerError::AccountNotFound)?;

        balance.pending = balance
            .pending
            .sub(&amount)
            .ok_or(LedgerError::InvalidAmount)?;
        balance.updated_at = Utc::now();

        Ok(())
    }

    /// Commit a transfer (from hold -> subtract).
    pub fn commit_transfer(
        &self,
        from_id: &str,
        to_id: &str,
        amount: Amount,
    ) -> Result<(), LedgerError> {
        let mut balances = self.balances.write();

        // Subtract from sender (and pending)
        let from_balance = balances
            .get_mut(from_id)
            .ok_or(LedgerError::AccountNotFound)?;

        from_balance.balance = from_balance
            .balance
            .sub(&amount)
            .ok_or(LedgerError::InvalidAmount)?;
        from_balance.pending = from_balance
            .pending
            .sub(&amount)
            .ok_or(LedgerError::InvalidAmount)?;
        from_balance.total_withdrawn = from_balance
            .total_withdrawn
            .add(&amount)
            .ok_or(LedgerError::InvalidAmount)?;
        from_balance.updated_at = Utc::now();

        // Add to receiver
        let to_balance = balances
            .entry(to_id.to_string())
            .or_insert_with(|| AgentBalance::new(to_id, self.default_currency));

        to_balance.balance = to_balance
            .balance
            .add(&amount)
            .ok_or(LedgerError::InvalidAmount)?;
        to_balance.total_deposited = to_balance
            .total_deposited
            .add(&amount)
            .ok_or(LedgerError::InvalidAmount)?;
        to_balance.updated_at = Utc::now();

        Ok(())
    }
}

/// Ledger errors.
#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    #[error("Account not found")]
    AccountNotFound,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Currency mismatch")]
    CurrencyMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit() {
        let ledger = BalanceLedger::default();
        let amount = Amount::from_float(100.0, 6);

        let balance = ledger.deposit("agent-1", amount).unwrap();
        assert_eq!(balance.balance.value, 100_000_000);
    }

    #[test]
    fn test_hold_and_commit() {
        let ledger = BalanceLedger::default();
        let deposit = Amount::from_float(100.0, 6);
        let transfer = Amount::from_float(30.0, 6);

        ledger.deposit("agent-1", deposit).unwrap();
        ledger.hold("agent-1", transfer).unwrap();

        let balance = ledger.get_balance("agent-1");
        assert_eq!(balance.available().to_float(), 70.0);

        ledger
            .commit_transfer("agent-1", "agent-2", transfer)
            .unwrap();

        let from_balance = ledger.get_balance("agent-1");
        let to_balance = ledger.get_balance("agent-2");

        assert_eq!(from_balance.balance.to_float(), 70.0);
        assert_eq!(to_balance.balance.to_float(), 30.0);
    }

    #[test]
    fn test_insufficient_funds() {
        let ledger = BalanceLedger::default();
        let amount = Amount::from_float(100.0, 6);

        let result = ledger.hold("agent-1", amount);
        assert!(matches!(result, Err(LedgerError::AccountNotFound)));

        ledger
            .deposit("agent-1", Amount::from_float(50.0, 6))
            .unwrap();
        let result = ledger.hold("agent-1", amount);
        assert!(matches!(result, Err(LedgerError::InsufficientFunds)));
    }
}
