//! Takaful (Mutual Risk Sharing)
//!
//! Per Phase 3 Roadmap: "Implement Takaful (Mutual Risk Sharing) in Treasury"
//!
//! Takaful is a Sharia-compliant mutual assistance model where agents 
//! contribute to a common pool to share risks.

use std::collections::HashMap;
use crate::types::{AgentId, Amount};
use agentkern_governance::industry::finance::shariah::{ShariahComplianceValidator, TransactionDetails, TransactionType};

/// A Takaful Pool for mutual risk sharing between agents.
pub struct TakafulPool {
    pub id: String,
    pub name: String,
    /// Total balance in the pool (Tabarru)
    pub balance: Amount,
    /// Contributions per agent
    pub contributions: HashMap<AgentId, Amount>,
    /// Compliance validator
    validator: ShariahComplianceValidator,
}

impl TakafulPool {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            balance: Amount::zero(),
            contributions: HashMap::new(),
            validator: ShariahComplianceValidator::new(),
        }
    }

    /// Contribute to the pool (Tabarru - donation).
    pub fn contribute(&mut self, agent_id: AgentId, amount: Amount) -> Result<(), String> {
        // Sharia Check: Ensure it's treated as a donation/contribution, not interest-bearing
        let details = TransactionDetails {
            transaction_type: TransactionType::Takaful,
            amount: amount.to_float(),
            ..Default::default()
        };

        match self.validator.validate(&details) {
            Ok(res) if res.compliant => {
                let entry = self.contributions.entry(agent_id).or_insert(Amount::zero());
                *entry = *entry + amount;
                self.balance = self.balance + amount;
                Ok(())
            }
            Ok(_) => Err("Transaction not Shariah-compliant".into()),
            Err(e) => Err(format!("Compliance error: {}", e)),
        }
    }

    /// Claim from the pool for a loss or penalty.
    pub fn claim(&mut self, agent_id: AgentId, amount: Amount) -> Result<(), String> {
        if amount > self.balance {
            return Err("Insufficient pool balance".into());
        }

        // Claims are granted based on mutual assistance rules.
        // For now, we allow any participant to claim (simplified).
        if !self.contributions.contains_key(&agent_id) {
            return Err("Agent is not a participant in this Takaful pool".into());
        }

        self.balance = self.balance - amount;
        Ok(())
    }

    pub fn get_balance(&self) -> Amount {
        self.balance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_takaful_mutual_assistance() {
        let mut pool = TakafulPool::new("pool-1", "Standard Risk Pool");
        let agent_a = AgentId::from("agent-a");
        let agent_b = AgentId::from("agent-b");

        // Agent A contributes $10
        pool.contribute(agent_a.clone(), Amount::from_float(10.0, 2)).unwrap();
        // Agent B contributes $5
        pool.contribute(agent_b.clone(), Amount::from_float(5.0, 2)).unwrap();

        assert_eq!(pool.get_balance().to_float(), 15.0);

        // Agent A suffers a failure and claims $12 (more than they put in, but less than pool total)
        pool.claim(agent_a.clone(), Amount::from_float(12.0, 2)).unwrap();

        assert_eq!(pool.get_balance().to_float(), 3.0);
    }
}
