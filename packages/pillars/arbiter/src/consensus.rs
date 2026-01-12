//! AgentKern-Arbiter: Consensus Engine
//!
//! Per Phase 5: Self-Evolving Governance
//!
//! Implements multi-agent voting mechanisms to allow collective
//! decision making and policy overrides (e.g. overriding a budget
//! during an emergency if a majority of trusted agents agree).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Types of proposals that can be voted on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalType {
    /// Override security gate for a specific resource
    SecurityOverride { resource: String, reason: String },
    /// Temporary budget increase for an agent or team
    BudgetIncrease {
        agent_id: String,
        amount_usd: Decimal,
    },
    /// Higher priority for a task category
    PriorityBoost { category: String },
    /// Emergency kill-switch activation/deactivation
    KillSwitch {
        activate: bool,
        target_agent: Option<String>,
    },
}

/// Status of a proposal.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalStatus {
    Open,
    Passed,
    Failed,
    Expired,
}

/// Agent's vote.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Vote {
    Aye,
    Nay,
    Abstain,
}

/// A proposal for collective governance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: Uuid,
    pub proposer_id: String,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    /// Set of agents who voted 'Aye'
    pub ayes: HashSet<String>,
    /// Set of agents who voted 'Nay'
    pub nays: HashSet<String>,
    /// Threshold of 'Aye' votes needed to pass (count or percentage logic)
    pub threshold: u32,
}

impl Proposal {
    pub fn new(
        proposer_id: impl Into<String>,
        proposal_type: ProposalType,
        threshold: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            proposer_id: proposer_id.into(),
            proposal_type,
            status: ProposalStatus::Open,
            created_at: now,
            expires_at: now + chrono::Duration::hours(1),
            ayes: HashSet::new(),
            nays: HashSet::new(),
            threshold,
        }
    }

    pub fn cast_vote(&mut self, agent_id: String, vote: Vote) {
        if self.status != ProposalStatus::Open || Utc::now() > self.expires_at {
            return;
        }

        match vote {
            Vote::Aye => {
                self.nays.remove(&agent_id);
                self.ayes.insert(agent_id);
            }
            Vote::Nay => {
                self.ayes.remove(&agent_id);
                self.nays.insert(agent_id);
            }
            Vote::Abstain => {
                self.ayes.remove(&agent_id);
                self.nays.remove(&agent_id);
            }
        }

        self.update_status();
    }

    fn update_status(&mut self) {
        if self.ayes.len() as u32 >= self.threshold {
            self.status = ProposalStatus::Passed;
        } else if self.expires_at < Utc::now() {
            self.status = ProposalStatus::Expired;
        }
    }
}

/// Manager for collective governance proposals.
pub struct ConsensusEngine {
    proposals: Arc<RwLock<HashMap<Uuid, Proposal>>>,
}

impl ConsensusEngine {
    pub fn new() -> Self {
        Self {
            proposals: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn propose(&self, proposer_id: &str, p_type: ProposalType, threshold: u32) -> Uuid {
        let proposal = Proposal::new(proposer_id, p_type, threshold);
        let id = proposal.id;
        let mut proposals = self.proposals.write().await;
        proposals.insert(id, proposal);
        id
    }

    pub async fn vote(
        &self,
        proposal_id: Uuid,
        agent_id: &str,
        vote: Vote,
    ) -> Option<ProposalStatus> {
        let mut proposals = self.proposals.write().await;
        if let Some(proposal) = proposals.get_mut(&proposal_id) {
            proposal.cast_vote(agent_id.to_string(), vote);
            Some(proposal.status)
        } else {
            None
        }
    }

    pub async fn get_proposal(&self, id: Uuid) -> Option<Proposal> {
        let proposals = self.proposals.read().await;
        proposals.get(&id).cloned()
    }

    /// Check if a specific security override has passed.
    pub async fn is_security_override_active(&self, resource: &str) -> bool {
        let proposals = self.proposals.read().await;
        proposals.values().any(|p| {
            p.status == ProposalStatus::Passed
                && match &p.proposal_type {
                    ProposalType::SecurityOverride { resource: r, .. } => r == resource,
                    _ => false,
                }
        })
    }

    /// Check if a budget override is active for an agent.
    pub async fn get_budget_override(&self, agent_id: &str) -> Decimal {
        let proposals = self.proposals.read().await;
        proposals
            .values()
            .filter(|p| {
                p.status == ProposalStatus::Passed
                    && match &p.proposal_type {
                        ProposalType::BudgetIncrease { agent_id: id, .. } => id == agent_id,
                        _ => false,
                    }
            })
            .map(|p| match &p.proposal_type {
                ProposalType::BudgetIncrease { amount_usd, .. } => *amount_usd,
                _ => Decimal::ZERO,
            })
            .sum()
    }
}

impl Default for ConsensusEngine {
    fn default() -> Self {
        Self::new()
    }
}
