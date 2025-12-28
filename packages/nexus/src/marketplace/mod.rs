//! Agent Marketplace
//!
//! Task bidding and settlement for agent-to-agent commerce.
//! Per Roadmap: "Agent Marketplace/Router"
//!
//! # Architecture
//!
//! 1. Task Announcement → Agents can bid
//! 2. Bid Evaluation → Select winner
//! 3. Escrow Lock → Payment secured
//! 4. Task Execution → Agent performs work
//! 5. Settlement → Payment released

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bid on a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bid {
    /// Bid ID
    pub id: String,
    /// Task ID
    pub task_id: String,
    /// Bidding agent
    pub agent_id: String,
    /// Bid amount (in credits)
    pub amount: f64,
    /// Estimated completion time (seconds)
    pub estimated_time_secs: u64,
    /// Confidence score (0-100)
    pub confidence: u8,
    /// Bid message
    pub message: Option<String>,
    /// Status
    pub status: BidStatus,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Bid status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BidStatus {
    /// Pending review
    Pending,
    /// Accepted
    Accepted,
    /// Rejected
    Rejected,
    /// Withdrawn
    Withdrawn,
}

impl Bid {
    /// Create a new bid.
    pub fn new(task_id: &str, agent_id: &str, amount: f64, estimated_time_secs: u64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            agent_id: agent_id.to_string(),
            amount,
            estimated_time_secs,
            confidence: 80,
            message: None,
            status: BidStatus::Pending,
            created_at: Utc::now(),
        }
    }

    /// Add confidence score.
    pub fn with_confidence(mut self, confidence: u8) -> Self {
        self.confidence = confidence.min(100);
        self
    }

    /// Add message.
    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Calculate value score (lower is better).
    pub fn value_score(&self) -> f64 {
        // Factor in price, time, and confidence
        let time_factor = self.estimated_time_secs as f64 / 3600.0; // Hours
        let confidence_factor = (100 - self.confidence) as f64 / 100.0;

        self.amount * (1.0 + time_factor * 0.1) * (1.0 + confidence_factor * 0.5)
    }
}

/// Task auction for marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAuction {
    /// Auction ID
    pub id: String,
    /// Task being auctioned
    pub task_id: String,
    /// Task description
    pub description: String,
    /// Required skills
    pub required_skills: Vec<String>,
    /// Maximum budget
    pub max_budget: f64,
    /// Deadline for bids
    pub bid_deadline: DateTime<Utc>,
    /// Execution deadline
    pub execution_deadline: DateTime<Utc>,
    /// Bids received
    pub bids: Vec<Bid>,
    /// Winning bid ID
    pub winning_bid: Option<String>,
    /// Status
    pub status: AuctionStatus,
    /// Created by agent
    pub created_by: String,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Auction status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuctionStatus {
    /// Open for bids
    Open,
    /// Bid deadline passed, evaluating
    Evaluating,
    /// Winner selected
    Awarded,
    /// Work in progress
    InProgress,
    /// Completed
    Completed,
    /// Cancelled
    Cancelled,
}

impl TaskAuction {
    /// Create a new auction.
    pub fn new(
        task_id: &str,
        description: &str,
        max_budget: f64,
        bid_hours: i64,
        execution_hours: i64,
        created_by: &str,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            description: description.to_string(),
            required_skills: vec![],
            max_budget,
            bid_deadline: now + chrono::Duration::hours(bid_hours),
            execution_deadline: now + chrono::Duration::hours(bid_hours + execution_hours),
            bids: vec![],
            winning_bid: None,
            status: AuctionStatus::Open,
            created_by: created_by.to_string(),
            created_at: now,
        }
    }

    /// Add required skill.
    pub fn with_skill(mut self, skill: impl Into<String>) -> Self {
        self.required_skills.push(skill.into());
        self
    }

    /// Submit a bid.
    pub fn submit_bid(&mut self, bid: Bid) -> Result<(), MarketplaceError> {
        if self.status != AuctionStatus::Open {
            return Err(MarketplaceError::AuctionClosed);
        }

        if Utc::now() > self.bid_deadline {
            return Err(MarketplaceError::BidDeadlinePassed);
        }

        if bid.amount > self.max_budget {
            return Err(MarketplaceError::BidExceedsBudget);
        }

        // Check for duplicate bid from same agent
        if self
            .bids
            .iter()
            .any(|b| b.agent_id == bid.agent_id && b.status == BidStatus::Pending)
        {
            return Err(MarketplaceError::DuplicateBid);
        }

        self.bids.push(bid);
        Ok(())
    }

    /// Evaluate bids and select winner.
    pub fn evaluate(&mut self) -> Option<&Bid> {
        self.status = AuctionStatus::Evaluating;

        // Get pending bids and find best
        let mut best_score = f64::MAX;
        let mut winner_id: Option<String> = None;

        for bid in self.bids.iter().filter(|b| b.status == BidStatus::Pending) {
            let score = bid.value_score();
            if score < best_score {
                best_score = score;
                winner_id = Some(bid.id.clone());
            }
        }

        if winner_id.is_none() {
            self.status = AuctionStatus::Cancelled;
            return None;
        }

        let winner_id = winner_id.unwrap();
        self.winning_bid = Some(winner_id.clone());
        self.status = AuctionStatus::Awarded;

        // Update bid statuses
        for bid in &mut self.bids {
            if bid.id == winner_id {
                bid.status = BidStatus::Accepted;
            } else if bid.status == BidStatus::Pending {
                bid.status = BidStatus::Rejected;
            }
        }

        self.bids.iter().find(|b| b.status == BidStatus::Accepted)
    }

    /// Get winning bid.
    pub fn get_winning_bid(&self) -> Option<&Bid> {
        self.winning_bid
            .as_ref()
            .and_then(|id| self.bids.iter().find(|b| &b.id == id))
    }

    /// Start execution.
    pub fn start_execution(&mut self) -> Result<(), MarketplaceError> {
        if self.status != AuctionStatus::Awarded {
            return Err(MarketplaceError::NotAwarded);
        }
        self.status = AuctionStatus::InProgress;
        Ok(())
    }

    /// Complete execution.
    pub fn complete(&mut self) -> Result<f64, MarketplaceError> {
        if self.status != AuctionStatus::InProgress {
            return Err(MarketplaceError::NotInProgress);
        }
        self.status = AuctionStatus::Completed;

        // Return amount to settle
        Ok(self.get_winning_bid().map(|b| b.amount).unwrap_or(0.0))
    }
}

/// Settlement record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    /// Settlement ID
    pub id: String,
    /// Auction ID
    pub auction_id: String,
    /// From agent (task creator)
    pub from_agent: String,
    /// To agent (task executor)
    pub to_agent: String,
    /// Amount
    pub amount: f64,
    /// Status
    pub status: SettlementStatus,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Settled at
    pub settled_at: Option<DateTime<Utc>>,
}

/// Settlement status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettlementStatus {
    /// Escrow locked
    Escrowed,
    /// Released to executor
    Released,
    /// Refunded to creator
    Refunded,
    /// Disputed
    Disputed,
}

/// Marketplace service.
pub struct Marketplace {
    auctions: HashMap<String, TaskAuction>,
    settlements: HashMap<String, Settlement>,
}

impl Marketplace {
    /// Create a new marketplace.
    pub fn new() -> Self {
        Self {
            auctions: HashMap::new(),
            settlements: HashMap::new(),
        }
    }

    /// Create an auction.
    pub fn create_auction(&mut self, auction: TaskAuction) -> String {
        let id = auction.id.clone();
        self.auctions.insert(id.clone(), auction);
        id
    }

    /// Get auction.
    pub fn get_auction(&self, id: &str) -> Option<&TaskAuction> {
        self.auctions.get(id)
    }

    /// Get mutable auction.
    pub fn get_auction_mut(&mut self, id: &str) -> Option<&mut TaskAuction> {
        self.auctions.get_mut(id)
    }

    /// List open auctions.
    pub fn list_open_auctions(&self) -> Vec<&TaskAuction> {
        self.auctions
            .values()
            .filter(|a| a.status == AuctionStatus::Open)
            .collect()
    }

    /// List auctions requiring a skill.
    pub fn find_auctions_by_skill(&self, skill: &str) -> Vec<&TaskAuction> {
        self.auctions
            .values()
            .filter(|a| a.status == AuctionStatus::Open)
            .filter(|a| {
                a.required_skills
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(skill))
            })
            .collect()
    }

    /// Create settlement for completed auction.
    pub fn create_settlement(&mut self, auction: &TaskAuction) -> Option<String> {
        let winning_bid = auction.get_winning_bid()?;

        let settlement = Settlement {
            id: uuid::Uuid::new_v4().to_string(),
            auction_id: auction.id.clone(),
            from_agent: auction.created_by.clone(),
            to_agent: winning_bid.agent_id.clone(),
            amount: winning_bid.amount,
            status: SettlementStatus::Escrowed,
            created_at: Utc::now(),
            settled_at: None,
        };

        let id = settlement.id.clone();
        self.settlements.insert(id.clone(), settlement);
        Some(id)
    }

    /// Release settlement.
    pub fn release_settlement(&mut self, id: &str) -> Result<f64, MarketplaceError> {
        let settlement = self
            .settlements
            .get_mut(id)
            .ok_or(MarketplaceError::SettlementNotFound)?;

        if settlement.status != SettlementStatus::Escrowed {
            return Err(MarketplaceError::InvalidSettlementState);
        }

        settlement.status = SettlementStatus::Released;
        settlement.settled_at = Some(Utc::now());

        Ok(settlement.amount)
    }

    /// Refund settlement.
    pub fn refund_settlement(&mut self, id: &str) -> Result<f64, MarketplaceError> {
        let settlement = self
            .settlements
            .get_mut(id)
            .ok_or(MarketplaceError::SettlementNotFound)?;

        if settlement.status != SettlementStatus::Escrowed {
            return Err(MarketplaceError::InvalidSettlementState);
        }

        settlement.status = SettlementStatus::Refunded;
        settlement.settled_at = Some(Utc::now());

        Ok(settlement.amount)
    }
}

impl Default for Marketplace {
    fn default() -> Self {
        Self::new()
    }
}

/// Marketplace errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum MarketplaceError {
    #[error("Auction is closed")]
    AuctionClosed,
    #[error("Bid deadline has passed")]
    BidDeadlinePassed,
    #[error("Bid exceeds budget")]
    BidExceedsBudget,
    #[error("Duplicate bid from agent")]
    DuplicateBid,
    #[error("Auction not awarded")]
    NotAwarded,
    #[error("Auction not in progress")]
    NotInProgress,
    #[error("Settlement not found")]
    SettlementNotFound,
    #[error("Invalid settlement state")]
    InvalidSettlementState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bid_creation() {
        let bid = Bid::new("task-1", "agent-1", 100.0, 3600)
            .with_confidence(95)
            .with_message("I can do this!");

        assert_eq!(bid.task_id, "task-1");
        assert_eq!(bid.confidence, 95);
        assert!(bid.message.is_some());
    }

    #[test]
    fn test_bid_value_score() {
        let bid1 = Bid::new("task-1", "agent-1", 100.0, 3600).with_confidence(90);
        let bid2 = Bid::new("task-1", "agent-2", 100.0, 7200).with_confidence(80);

        // bid1 should have lower (better) score (faster, more confident)
        assert!(bid1.value_score() < bid2.value_score());
    }

    #[test]
    fn test_auction_workflow() {
        let mut auction =
            TaskAuction::new("task-1", "Data analysis needed", 500.0, 24, 48, "client-1")
                .with_skill("data-analysis");

        // Submit bids
        let bid1 = Bid::new("task-1", "agent-1", 200.0, 3600).with_confidence(85);
        let bid2 = Bid::new("task-1", "agent-2", 150.0, 7200).with_confidence(90);
        let bid3 = Bid::new("task-1", "agent-3", 180.0, 5400).with_confidence(95);

        auction.submit_bid(bid1).unwrap();
        auction.submit_bid(bid2).unwrap();
        auction.submit_bid(bid3).unwrap();

        assert_eq!(auction.bids.len(), 3);

        // Evaluate
        let winner = auction.evaluate().unwrap();

        assert_eq!(auction.status, AuctionStatus::Awarded);
        assert!(auction.winning_bid.is_some());
    }

    #[test]
    fn test_marketplace() {
        let mut market = Marketplace::new();

        let auction = TaskAuction::new("task-1", "Need ML model", 1000.0, 12, 24, "requester-1")
            .with_skill("machine-learning");

        let auction_id = market.create_auction(auction);

        assert!(market.get_auction(&auction_id).is_some());
        assert_eq!(market.list_open_auctions().len(), 1);
        assert_eq!(market.find_auctions_by_skill("machine-learning").len(), 1);
    }

    #[test]
    fn test_settlement() {
        let mut market = Marketplace::new();

        let mut auction = TaskAuction::new("task-1", "Quick task", 100.0, 1, 2, "client-1");

        let bid = Bid::new("task-1", "worker-1", 50.0, 1800);
        auction.submit_bid(bid).unwrap();
        auction.evaluate();
        auction.start_execution().unwrap();
        auction.complete().unwrap();

        let settlement_id = market.create_settlement(&auction).unwrap();
        let amount = market.release_settlement(&settlement_id).unwrap();

        assert_eq!(amount, 50.0);
    }

    #[test]
    fn test_bid_exceeds_budget() {
        let mut auction = TaskAuction::new("task-1", "Limited budget", 100.0, 1, 1, "client-1");

        let expensive_bid = Bid::new("task-1", "agent-1", 150.0, 3600);
        let result = auction.submit_bid(expensive_bid);

        assert!(matches!(result, Err(MarketplaceError::BidExceedsBudget)));
    }

    #[test]
    fn test_duplicate_bid() {
        let mut auction = TaskAuction::new("task-1", "Test task", 200.0, 1, 1, "client-1");

        let bid1 = Bid::new("task-1", "agent-1", 100.0, 3600);
        let bid2 = Bid::new("task-1", "agent-1", 90.0, 3600);

        auction.submit_bid(bid1).unwrap();
        let result = auction.submit_bid(bid2);

        assert!(matches!(result, Err(MarketplaceError::DuplicateBid)));
    }

    #[test]
    fn test_auction_cancelled_no_bids() {
        let mut auction = TaskAuction::new("task-1", "Unpopular task", 500.0, 1, 1, "client-1");

        // Evaluate with no bids
        let winner = auction.evaluate();

        assert!(winner.is_none());
        assert_eq!(auction.status, AuctionStatus::Cancelled);
    }

    #[test]
    fn test_settlement_refund() {
        let mut market = Marketplace::new();

        let mut auction = TaskAuction::new("task-1", "Refundable", 100.0, 1, 1, "client-1");

        let bid = Bid::new("task-1", "worker-1", 75.0, 1800);
        auction.submit_bid(bid).unwrap();
        auction.evaluate();

        let settlement_id = market.create_settlement(&auction).unwrap();
        let refund = market.refund_settlement(&settlement_id).unwrap();

        assert_eq!(refund, 75.0);
    }
}
