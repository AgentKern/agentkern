#![allow(unused)]
#![allow(dead_code)]
//! AgentKern Enterprise: Cross-Agent Payment Rails (Treasury)
//!
//! Per Gap Analysis: "Cross-Agent Payment Rails - No one is solving this yet"
//!
//! **License**: AgentKern Enterprise License
//!
//! Features:
//! - Agent-to-Agent micropayments
//! - L402 Protocol integration (HTTP 402 Payment Required)
//! - Multi-currency support (fiat, crypto, stablecoins)
//! - Payment channels and escrow
//! - Real-time settlement
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_treasury::{Treasury, PaymentRequest};
//!
//! let treasury = Treasury::new("org-123")?;
//! treasury.pay_agent("agent-A", "agent-B", 0.001)?;
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use thiserror::Error;

mod license {
    #[derive(Debug, thiserror::Error)]
    pub enum LicenseError {
        #[error("Enterprise license required for treasury")]
        LicenseRequired,
    }

    pub fn require(feature: &str) -> Result<(), LicenseError> {
        let key = std::env::var("AGENTKERN_LICENSE_KEY")
            .map_err(|_| LicenseError::LicenseRequired)?;
        
        if key.is_empty() {
            return Err(LicenseError::LicenseRequired);
        }
        
        tracing::debug!(feature = %feature, "Enterprise treasury feature accessed");
        Ok(())
    }
}

/// Treasury errors.
#[derive(Debug, Error)]
pub enum TreasuryError {
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: f64, available: f64 },
    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: String },
    #[error("Payment failed: {reason}")]
    PaymentFailed { reason: String },
    #[error("Invalid amount: {amount}")]
    InvalidAmount { amount: f64 },
    #[error("Channel not open")]
    ChannelNotOpen,
    #[error("Payment expired")]
    PaymentExpired,
}

/// Supported currencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    /// US Dollar (fiat)
    Usd,
    /// Euro (fiat)
    Eur,
    /// Bitcoin
    Btc,
    /// Bitcoin Satoshis
    Sats,
    /// Ethereum
    Eth,
    /// USDC Stablecoin
    Usdc,
    /// USDT Stablecoin
    Usdt,
    /// AgentKern Credits (internal)
    Credits,
}

impl Currency {
    /// Get decimal places.
    pub fn decimals(&self) -> u8 {
        match self {
            Self::Usd | Self::Eur | Self::Usdc | Self::Usdt => 2,
            Self::Btc => 8,
            Self::Sats => 0,
            Self::Eth => 18,
            Self::Credits => 6,
        }
    }

    /// Convert to base units.
    pub fn to_base_units(&self, amount: f64) -> u64 {
        let multiplier = 10_u64.pow(self.decimals() as u32);
        (amount * multiplier as f64) as u64
    }

    /// Convert from base units.
    pub fn from_base_units(&self, units: u64) -> f64 {
        let multiplier = 10_u64.pow(self.decimals() as u32);
        units as f64 / multiplier as f64
    }
}

impl Default for Currency {
    fn default() -> Self {
        Self::Credits
    }
}

/// Agent wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentWallet {
    /// Agent ID
    pub agent_id: String,
    /// Balances by currency
    pub balances: HashMap<Currency, u64>,
    /// Pending incoming payments
    pub pending_incoming: u64,
    /// Pending outgoing payments
    pub pending_outgoing: u64,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Last activity
    pub last_activity: DateTime<Utc>,
}

impl AgentWallet {
    /// Create a new wallet for an agent.
    pub fn new(agent_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            agent_id: agent_id.into(),
            balances: HashMap::new(),
            pending_incoming: 0,
            pending_outgoing: 0,
            created_at: now,
            last_activity: now,
        }
    }

    /// Get balance for a currency.
    pub fn balance(&self, currency: Currency) -> f64 {
        let units = self.balances.get(&currency).copied().unwrap_or(0);
        currency.from_base_units(units)
    }

    /// Deposit funds.
    pub fn deposit(&mut self, currency: Currency, amount: f64) {
        let units = currency.to_base_units(amount);
        *self.balances.entry(currency).or_insert(0) += units;
        self.last_activity = Utc::now();
    }

    /// Withdraw funds.
    pub fn withdraw(&mut self, currency: Currency, amount: f64) -> Result<(), TreasuryError> {
        let units = currency.to_base_units(amount);
        let balance = self.balances.entry(currency).or_insert(0);
        
        if *balance < units {
            return Err(TreasuryError::InsufficientBalance {
                required: amount,
                available: currency.from_base_units(*balance),
            });
        }
        
        *balance -= units;
        self.last_activity = Utc::now();
        Ok(())
    }
}

/// Payment request (L402-style).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    /// Request ID
    pub id: String,
    /// From agent
    pub from_agent: String,
    /// To agent
    pub to_agent: String,
    /// Amount
    pub amount: f64,
    /// Currency
    pub currency: Currency,
    /// Description
    pub description: Option<String>,
    /// Expires at
    pub expires_at: DateTime<Utc>,
    /// Macaroon (L402 authentication token)
    pub macaroon: Option<String>,
    /// Invoice (Lightning-style payment hash)
    pub invoice: Option<String>,
    /// Status
    pub status: PaymentStatus,
    /// Created at
    pub created_at: DateTime<Utc>,
}

impl PaymentRequest {
    /// Create a new payment request.
    pub fn new(
        from_agent: impl Into<String>,
        to_agent: impl Into<String>,
        amount: f64,
        currency: Currency,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from_agent: from_agent.into(),
            to_agent: to_agent.into(),
            amount,
            currency,
            description: None,
            expires_at: now + chrono::Duration::minutes(10),
            macaroon: None,
            invoice: None,
            status: PaymentStatus::Pending,
            created_at: now,
        }
    }

    /// Set description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Generate L402 invoice.
    pub fn generate_invoice(&mut self) -> String {
        // Generate payment hash (simulated)
        let hash = format!("lnbc{}u1p{}", 
            (self.amount * 100.0) as u64,
            &self.id[..8]
        );
        self.invoice = Some(hash.clone());
        hash
    }

    /// Check if expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Payment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    /// Pending payment
    Pending,
    /// Processing
    Processing,
    /// Completed successfully
    Completed,
    /// Failed
    Failed,
    /// Expired
    Expired,
    /// Refunded
    Refunded,
}

/// Payment channel for high-frequency micropayments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentChannel {
    /// Channel ID
    pub id: String,
    /// Party A (initiator)
    pub party_a: String,
    /// Party B
    pub party_b: String,
    /// Total capacity
    pub capacity: u64,
    /// Balance of party A
    pub balance_a: u64,
    /// Balance of party B
    pub balance_b: u64,
    /// Currency
    pub currency: Currency,
    /// Is open
    pub is_open: bool,
    /// Transaction count
    pub tx_count: u64,
    /// Created at
    pub created_at: DateTime<Utc>,
}

impl PaymentChannel {
    /// Create a new payment channel.
    pub fn new(
        party_a: impl Into<String>,
        party_b: impl Into<String>,
        capacity: f64,
        currency: Currency,
    ) -> Self {
        let capacity_units = currency.to_base_units(capacity);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            party_a: party_a.into(),
            party_b: party_b.into(),
            capacity: capacity_units,
            balance_a: capacity_units,
            balance_b: 0,
            currency,
            is_open: true,
            tx_count: 0,
            created_at: Utc::now(),
        }
    }

    /// Transfer from A to B.
    pub fn transfer_a_to_b(&mut self, amount: f64) -> Result<(), TreasuryError> {
        if !self.is_open {
            return Err(TreasuryError::ChannelNotOpen);
        }
        
        let units = self.currency.to_base_units(amount);
        
        if self.balance_a < units {
            return Err(TreasuryError::InsufficientBalance {
                required: amount,
                available: self.currency.from_base_units(self.balance_a),
            });
        }
        
        self.balance_a -= units;
        self.balance_b += units;
        self.tx_count += 1;
        
        Ok(())
    }

    /// Transfer from B to A.
    pub fn transfer_b_to_a(&mut self, amount: f64) -> Result<(), TreasuryError> {
        if !self.is_open {
            return Err(TreasuryError::ChannelNotOpen);
        }
        
        let units = self.currency.to_base_units(amount);
        
        if self.balance_b < units {
            return Err(TreasuryError::InsufficientBalance {
                required: amount,
                available: self.currency.from_base_units(self.balance_b),
            });
        }
        
        self.balance_b -= units;
        self.balance_a += units;
        self.tx_count += 1;
        
        Ok(())
    }

    /// Close the channel and settle.
    pub fn close(&mut self) -> (f64, f64) {
        self.is_open = false;
        (
            self.currency.from_base_units(self.balance_a),
            self.currency.from_base_units(self.balance_b),
        )
    }
}

/// Escrow for conditional payments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escrow {
    /// Escrow ID
    pub id: String,
    /// From agent
    pub from_agent: String,
    /// To agent
    pub to_agent: String,
    /// Amount held
    pub amount: f64,
    /// Currency
    pub currency: Currency,
    /// Release condition (serialized)
    pub condition: String,
    /// Status
    pub status: EscrowStatus,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: DateTime<Utc>,
}

/// Escrow status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EscrowStatus {
    /// Funds locked
    Locked,
    /// Released to recipient
    Released,
    /// Refunded to sender
    Refunded,
    /// Expired
    Expired,
}

impl Escrow {
    /// Create a new escrow.
    pub fn new(
        from_agent: impl Into<String>,
        to_agent: impl Into<String>,
        amount: f64,
        currency: Currency,
        condition: impl Into<String>,
        duration_hours: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from_agent: from_agent.into(),
            to_agent: to_agent.into(),
            amount,
            currency,
            condition: condition.into(),
            status: EscrowStatus::Locked,
            created_at: now,
            expires_at: now + chrono::Duration::hours(duration_hours),
        }
    }

    /// Release funds to recipient.
    pub fn release(&mut self) -> Result<f64, TreasuryError> {
        if self.status != EscrowStatus::Locked {
            return Err(TreasuryError::PaymentFailed {
                reason: "Escrow not locked".to_string(),
            });
        }
        
        self.status = EscrowStatus::Released;
        Ok(self.amount)
    }

    /// Refund to sender.
    pub fn refund(&mut self) -> Result<f64, TreasuryError> {
        if self.status != EscrowStatus::Locked {
            return Err(TreasuryError::PaymentFailed {
                reason: "Escrow not locked".to_string(),
            });
        }
        
        self.status = EscrowStatus::Refunded;
        Ok(self.amount)
    }
}

/// L402 Response for payment-required APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L402Result {
    /// HTTP 402 Payment Required
    pub status: u16,
    /// WWW-Authenticate header value
    pub www_authenticate: String,
    /// Invoice to pay
    pub invoice: String,
    /// Price in sats
    pub price_sats: u64,
    /// Macaroon (after payment)
    pub macaroon: Option<String>,
}

impl L402Result {
    /// Create a new L402 response.
    pub fn new(invoice: &str, price_sats: u64) -> Self {
        Self {
            status: 402,
            www_authenticate: format!(
                "L402 macaroon=\"\", invoice=\"{}\"",
                invoice
            ),
            invoice: invoice.to_string(),
            price_sats,
            macaroon: None,
        }
    }
}

/// Treasury service (requires enterprise license).
pub struct Treasury {
    tenant_id: String,
    wallets: HashMap<String, AgentWallet>,
    channels: HashMap<String, PaymentChannel>,
    escrows: HashMap<String, Escrow>,
    pending_payments: Vec<PaymentRequest>,
}

impl Treasury {
    /// Create a new treasury (requires enterprise license).
    pub fn new(tenant_id: impl Into<String>) -> Result<Self, license::LicenseError> {
        license::require("TREASURY")?;
        
        Ok(Self {
            tenant_id: tenant_id.into(),
            wallets: HashMap::new(),
            channels: HashMap::new(),
            escrows: HashMap::new(),
            pending_payments: Vec::new(),
        })
    }

    /// Register an agent wallet.
    pub fn register_agent(&mut self, agent_id: &str) {
        if !self.wallets.contains_key(agent_id) {
            self.wallets.insert(agent_id.to_string(), AgentWallet::new(agent_id));
        }
    }

    /// Deposit funds to an agent.
    pub fn deposit(&mut self, agent_id: &str, currency: Currency, amount: f64) -> Result<(), TreasuryError> {
        let wallet = self.wallets.get_mut(agent_id).ok_or(TreasuryError::AgentNotFound {
            agent_id: agent_id.to_string(),
        })?;
        
        wallet.deposit(currency, amount);
        Ok(())
    }

    /// Get agent balance.
    pub fn balance(&self, agent_id: &str, currency: Currency) -> Result<f64, TreasuryError> {
        let wallet = self.wallets.get(agent_id).ok_or(TreasuryError::AgentNotFound {
            agent_id: agent_id.to_string(),
        })?;
        
        Ok(wallet.balance(currency))
    }

    /// Pay from one agent to another.
    pub fn pay(
        &mut self,
        from_agent: &str,
        to_agent: &str,
        amount: f64,
        currency: Currency,
    ) -> Result<String, TreasuryError> {
        if amount <= 0.0 {
            return Err(TreasuryError::InvalidAmount { amount });
        }
        
        // Check sender balance
        let from_balance = self.balance(from_agent, currency)?;
        if from_balance < amount {
            return Err(TreasuryError::InsufficientBalance {
                required: amount,
                available: from_balance,
            });
        }
        
        // Check recipient exists
        if !self.wallets.contains_key(to_agent) {
            return Err(TreasuryError::AgentNotFound {
                agent_id: to_agent.to_string(),
            });
        }
        
        // Execute transfer
        let from_wallet = self.wallets.get_mut(from_agent).unwrap();
        from_wallet.withdraw(currency, amount)?;
        
        let to_wallet = self.wallets.get_mut(to_agent).unwrap();
        to_wallet.deposit(currency, amount);
        
        // Create payment record
        let mut request = PaymentRequest::new(from_agent, to_agent, amount, currency);
        request.status = PaymentStatus::Completed;
        let payment_id = request.id.clone();
        self.pending_payments.push(request);
        
        Ok(payment_id)
    }

    /// Create a payment channel.
    pub fn open_channel(
        &mut self,
        party_a: &str,
        party_b: &str,
        capacity: f64,
        currency: Currency,
    ) -> Result<String, TreasuryError> {
        // Check party A has funds
        let balance = self.balance(party_a, currency)?;
        if balance < capacity {
            return Err(TreasuryError::InsufficientBalance {
                required: capacity,
                available: balance,
            });
        }
        
        // Lock funds
        let wallet = self.wallets.get_mut(party_a).unwrap();
        wallet.withdraw(currency, capacity)?;
        
        // Create channel
        let channel = PaymentChannel::new(party_a, party_b, capacity, currency);
        let channel_id = channel.id.clone();
        self.channels.insert(channel_id.clone(), channel);
        
        Ok(channel_id)
    }

    /// Transfer within a channel.
    pub fn channel_transfer(
        &mut self,
        channel_id: &str,
        from_a_to_b: bool,
        amount: f64,
    ) -> Result<(), TreasuryError> {
        let channel = self.channels.get_mut(channel_id).ok_or(TreasuryError::ChannelNotOpen)?;
        
        if from_a_to_b {
            channel.transfer_a_to_b(amount)
        } else {
            channel.transfer_b_to_a(amount)
        }
    }

    /// Close a payment channel.
    pub fn close_channel(&mut self, channel_id: &str) -> Result<(f64, f64), TreasuryError> {
        let channel = self.channels.get_mut(channel_id).ok_or(TreasuryError::ChannelNotOpen)?;
        
        let (balance_a, balance_b) = channel.close();
        let currency = channel.currency;
        let party_a = channel.party_a.clone();
        let party_b = channel.party_b.clone();
        
        // Return funds to wallets
        if let Some(wallet) = self.wallets.get_mut(&party_a) {
            wallet.deposit(currency, balance_a);
        }
        if let Some(wallet) = self.wallets.get_mut(&party_b) {
            wallet.deposit(currency, balance_b);
        }
        
        Ok((balance_a, balance_b))
    }

    /// Create an escrow.
    pub fn create_escrow(
        &mut self,
        from_agent: &str,
        to_agent: &str,
        amount: f64,
        currency: Currency,
        condition: &str,
        duration_hours: i64,
    ) -> Result<String, TreasuryError> {
        // Check and lock funds
        let wallet = self.wallets.get_mut(from_agent).ok_or(TreasuryError::AgentNotFound {
            agent_id: from_agent.to_string(),
        })?;
        
        wallet.withdraw(currency, amount)?;
        
        // Create escrow
        let escrow = Escrow::new(from_agent, to_agent, amount, currency, condition, duration_hours);
        let escrow_id = escrow.id.clone();
        self.escrows.insert(escrow_id.clone(), escrow);
        
        Ok(escrow_id)
    }

    /// Release escrow to recipient.
    pub fn release_escrow(&mut self, escrow_id: &str) -> Result<(), TreasuryError> {
        let escrow = self.escrows.get_mut(escrow_id).ok_or(TreasuryError::PaymentFailed {
            reason: "Escrow not found".to_string(),
        })?;
        
        let amount = escrow.release()?;
        let currency = escrow.currency;
        let to_agent = escrow.to_agent.clone();
        
        // Credit recipient
        if let Some(wallet) = self.wallets.get_mut(&to_agent) {
            wallet.deposit(currency, amount);
        }
        
        Ok(())
    }
}

// ============================================================================
// INSURANCE MODULE - Per Roadmap: "Agent Liability Insurance"
// ============================================================================

/// Insurance policy for agent liability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsurancePolicy {
    /// Policy ID
    pub id: String,
    /// Agent covered
    pub agent_id: String,
    /// Coverage type
    pub coverage_type: CoverageType,
    /// Maximum coverage amount
    pub max_coverage: f64,
    /// Coverage currency
    pub currency: Currency,
    /// Premium amount (per month)
    pub premium: f64,
    /// Deductible
    pub deductible: f64,
    /// Policy status
    pub status: PolicyStatus,
    /// Underwriter (e.g., "Munich Re", "Lloyd's")
    pub underwriter: String,
    /// Start date
    pub start_date: DateTime<Utc>,
    /// End date
    pub end_date: DateTime<Utc>,
    /// Claims history
    pub claims: Vec<InsuranceClaim>,
}

/// Coverage types for agent insurance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageType {
    /// Errors and Omissions
    ErrorsOmissions,
    /// Cyber liability
    CyberLiability,
    /// Professional liability
    ProfessionalLiability,
    /// General liability
    GeneralLiability,
    /// Transaction protection
    TransactionProtection,
    /// Comprehensive (all of above)
    Comprehensive,
}

/// Policy status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyStatus {
    Active,
    Pending,
    Expired,
    Cancelled,
    ClaimInProgress,
}

/// Insurance claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceClaim {
    /// Claim ID
    pub id: String,
    /// Amount claimed
    pub amount: f64,
    /// Reason
    pub reason: String,
    /// Incident date
    pub incident_date: DateTime<Utc>,
    /// Claim status
    pub status: ClaimStatus,
    /// Payout amount (if approved)
    pub payout: Option<f64>,
}

/// Claim status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClaimStatus {
    Submitted,
    UnderReview,
    Approved,
    Denied,
    Paid,
}

impl InsurancePolicy {
    /// Create a new insurance policy.
    pub fn new(
        agent_id: impl Into<String>,
        coverage_type: CoverageType,
        max_coverage: f64,
        premium: f64,
        underwriter: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.into(),
            coverage_type,
            max_coverage,
            currency: Currency::Usd,
            premium,
            deductible: max_coverage * 0.01, // 1% deductible
            status: PolicyStatus::Active,
            underwriter: underwriter.into(),
            start_date: now,
            end_date: now + chrono::Duration::days(365),
            claims: vec![],
        }
    }

    /// Check if policy is active.
    pub fn is_active(&self) -> bool {
        self.status == PolicyStatus::Active && Utc::now() < self.end_date
    }

    /// Get available coverage (max minus pending claims).
    pub fn available_coverage(&self) -> f64 {
        let pending_claims: f64 = self.claims
            .iter()
            .filter(|c| matches!(c.status, ClaimStatus::Submitted | ClaimStatus::UnderReview))
            .map(|c| c.amount)
            .sum();
        
        (self.max_coverage - pending_claims).max(0.0)
    }

    /// Submit a claim.
    pub fn submit_claim(&mut self, amount: f64, reason: impl Into<String>) -> Result<String, TreasuryError> {
        if !self.is_active() {
            return Err(TreasuryError::PaymentFailed {
                reason: "Policy not active".to_string(),
            });
        }

        if amount > self.available_coverage() {
            return Err(TreasuryError::InsufficientBalance {
                required: amount,
                available: self.available_coverage(),
            });
        }

        let claim = InsuranceClaim {
            id: uuid::Uuid::new_v4().to_string(),
            amount,
            reason: reason.into(),
            incident_date: Utc::now(),
            status: ClaimStatus::Submitted,
            payout: None,
        };

        let claim_id = claim.id.clone();
        self.claims.push(claim);
        self.status = PolicyStatus::ClaimInProgress;

        Ok(claim_id)
    }

    /// Verify coverage for an action.
    pub fn verify_coverage(&self, action: &str, estimated_risk: f64) -> CoverageVerification {
        CoverageVerification {
            covered: self.is_active() && estimated_risk <= self.available_coverage(),
            policy_id: self.id.clone(),
            available_coverage: self.available_coverage(),
            deductible: self.deductible,
            action: action.to_string(),
            estimated_risk,
        }
    }
}

/// Coverage verification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageVerification {
    /// Is the action covered?
    pub covered: bool,
    /// Policy ID
    pub policy_id: String,
    /// Available coverage amount
    pub available_coverage: f64,
    /// Deductible
    pub deductible: f64,
    /// Action being verified
    pub action: String,
    /// Estimated risk amount
    pub estimated_risk: f64,
}

// ============================================================================
// LEGAL ENTITY MODULE - Per Roadmap: "Agent Legal Entity Framework"
// ============================================================================

/// Agent Legal Entity - First-ever legal personhood for AI agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLegalEntity {
    /// Entity ID
    pub id: String,
    /// Associated agent ID
    pub agent_id: String,
    /// Legal name
    pub legal_name: String,
    /// Entity type
    pub entity_type: LegalEntityType,
    /// Jurisdiction
    pub jurisdiction: Jurisdiction,
    /// Registration number
    pub registration_number: Option<String>,
    /// EIN/Tax ID (if applicable)
    pub tax_id: Option<String>,
    /// Registered agent (human or organization)
    pub registered_agent: String,
    /// Status
    pub status: EntityStatus,
    /// Formation date
    pub formation_date: DateTime<Utc>,
    /// Operating agreement hash (on-chain)
    pub operating_agreement_hash: Option<String>,
}

/// Legal entity types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LegalEntityType {
    /// Wyoming DAO LLC (first US jurisdiction for algorithmic entities)
    WyomingDaoLlc,
    /// Delaware LLC
    DelawareLlc,
    /// Cayman Islands Foundation
    CaymanFoundation,
    /// Singapore Variable Capital Company
    SingaporeVcc,
    /// Swiss Association
    SwissAssociation,
    /// Unincorporated (no legal entity)
    Unincorporated,
}

impl LegalEntityType {
    /// Get jurisdictions that support this entity type.
    pub fn jurisdictions(&self) -> Vec<Jurisdiction> {
        match self {
            Self::WyomingDaoLlc => vec![Jurisdiction::UsWyoming],
            Self::DelawareLlc => vec![Jurisdiction::UsDelaware],
            Self::CaymanFoundation => vec![Jurisdiction::CaymanIslands],
            Self::SingaporeVcc => vec![Jurisdiction::Singapore],
            Self::SwissAssociation => vec![Jurisdiction::Switzerland],
            Self::Unincorporated => vec![Jurisdiction::None],
        }
    }

    /// Get minimum requirements.
    pub fn requirements(&self) -> EntityRequirements {
        match self {
            Self::WyomingDaoLlc => EntityRequirements {
                registered_agent_required: true,
                minimum_members: 1,
                annual_report_required: true,
                smart_contract_governance: true,
            },
            Self::DelawareLlc => EntityRequirements {
                registered_agent_required: true,
                minimum_members: 1,
                annual_report_required: true,
                smart_contract_governance: false,
            },
            _ => EntityRequirements::default(),
        }
    }
}

/// Jurisdiction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    UsWyoming,
    UsDelaware,
    CaymanIslands,
    Singapore,
    Switzerland,
    Eu,
    None,
}

/// Entity requirements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityRequirements {
    pub registered_agent_required: bool,
    pub minimum_members: u32,
    pub annual_report_required: bool,
    pub smart_contract_governance: bool,
}

/// Entity status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityStatus {
    Pending,
    Active,
    Suspended,
    Dissolved,
}

impl AgentLegalEntity {
    /// Create a new legal entity for an agent.
    pub fn new(
        agent_id: impl Into<String>,
        legal_name: impl Into<String>,
        entity_type: LegalEntityType,
        registered_agent: impl Into<String>,
    ) -> Self {
        let jurisdictions = entity_type.jurisdictions();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.into(),
            legal_name: legal_name.into(),
            entity_type,
            jurisdiction: jurisdictions.first().copied().unwrap_or(Jurisdiction::None),
            registration_number: None,
            tax_id: None,
            registered_agent: registered_agent.into(),
            status: EntityStatus::Pending,
            formation_date: Utc::now(),
            operating_agreement_hash: None,
        }
    }

    /// Register the entity (simulate filing).
    pub fn register(&mut self) -> Result<String, TreasuryError> {
        let requirements = self.entity_type.requirements();
        
        if requirements.registered_agent_required && self.registered_agent.is_empty() {
            return Err(TreasuryError::PaymentFailed {
                reason: "Registered agent required".to_string(),
            });
        }

        // Generate registration number
        let reg_number = format!(
            "{}-{}-{}",
            match self.jurisdiction {
                Jurisdiction::UsWyoming => "WY",
                Jurisdiction::UsDelaware => "DE",
                Jurisdiction::CaymanIslands => "KY",
                Jurisdiction::Singapore => "SG",
                Jurisdiction::Switzerland => "CH",
                _ => "XX",
            },
            chrono::Utc::now().format("%Y"),
            &self.id[..8]
        );

        self.registration_number = Some(reg_number.clone());
        self.status = EntityStatus::Active;

        Ok(reg_number)
    }

    /// Check if entity can enter contracts.
    pub fn can_contract(&self) -> bool {
        self.status == EntityStatus::Active && self.registration_number.is_some()
    }

    /// Get liability shield status.
    pub fn has_liability_shield(&self) -> bool {
        matches!(
            self.entity_type,
            LegalEntityType::WyomingDaoLlc
                | LegalEntityType::DelawareLlc
                | LegalEntityType::CaymanFoundation
        ) && self.status == EntityStatus::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_conversion() {
        let btc = Currency::Btc;
        assert_eq!(btc.to_base_units(1.0), 100_000_000);
        assert_eq!(btc.from_base_units(100_000_000), 1.0);
        
        let usd = Currency::Usd;
        assert_eq!(usd.to_base_units(100.50), 10050);
    }

    #[test]
    fn test_agent_wallet() {
        let mut wallet = AgentWallet::new("agent-1");
        
        wallet.deposit(Currency::Credits, 100.0);
        assert_eq!(wallet.balance(Currency::Credits), 100.0);
        
        wallet.withdraw(Currency::Credits, 30.0).unwrap();
        assert_eq!(wallet.balance(Currency::Credits), 70.0);
    }

    #[test]
    fn test_payment_channel() {
        let mut channel = PaymentChannel::new("alice", "bob", 100.0, Currency::Credits);
        
        // Alice pays Bob 30
        channel.transfer_a_to_b(30.0).unwrap();
        assert_eq!(channel.balance_a, 70_000_000);
        assert_eq!(channel.balance_b, 30_000_000);
        
        // Bob pays Alice back 10
        channel.transfer_b_to_a(10.0).unwrap();
        assert_eq!(channel.tx_count, 2);
    }

    #[test]
    fn test_treasury_requires_license() {
        // SAFETY: Only used in tests, no concurrent access
        unsafe { std::env::remove_var("AGENTKERN_LICENSE_KEY") };
        let result = Treasury::new("org-123");
        assert!(result.is_err());
    }

    #[test]
    fn test_treasury_payments() {
        // SAFETY: Only used in tests, no concurrent access
        unsafe { std::env::set_var("AGENTKERN_LICENSE_KEY", "test-license") };
        
        let mut treasury = Treasury::new("org-123").unwrap();
        
        treasury.register_agent("agent-A");
        treasury.register_agent("agent-B");
        
        treasury.deposit("agent-A", Currency::Credits, 100.0).unwrap();
        
        let payment_id = treasury.pay("agent-A", "agent-B", 25.0, Currency::Credits).unwrap();
        
        assert!(!payment_id.is_empty());
        assert_eq!(treasury.balance("agent-A", Currency::Credits).unwrap(), 75.0);
        assert_eq!(treasury.balance("agent-B", Currency::Credits).unwrap(), 25.0);
        
        // SAFETY: Only used in tests, no concurrent access
        unsafe { std::env::remove_var("AGENTKERN_LICENSE_KEY") };
    }

    #[test]
    fn test_escrow() {
        let mut escrow = Escrow::new(
            "seller",
            "buyer",
            50.0,
            Currency::Usdc,
            "delivery_confirmed",
            24
        );
        
        assert_eq!(escrow.status, EscrowStatus::Locked);
        
        let amount = escrow.release().unwrap();
        assert_eq!(amount, 50.0);
        assert_eq!(escrow.status, EscrowStatus::Released);
    }

    #[test]
    fn test_l402_response() {
        let response = L402Result::new("lnbc1000n1...", 1000);
        
        assert_eq!(response.status, 402);
        assert!(response.www_authenticate.contains("L402"));
    }

    // ========== NEW INSURANCE TESTS ==========

    #[test]
    fn test_insurance_policy_creation() {
        let policy = InsurancePolicy::new(
            "agent-1",
            CoverageType::Comprehensive,
            1_000_000.0,
            5_000.0,
            "Munich Re"
        );
        
        assert!(policy.is_active());
        assert_eq!(policy.available_coverage(), 1_000_000.0);
        assert_eq!(policy.deductible, 10_000.0); // 1% of 1M
    }

    #[test]
    fn test_insurance_claim() {
        let mut policy = InsurancePolicy::new(
            "agent-1",
            CoverageType::ErrorsOmissions,
            100_000.0,
            1_000.0,
            "Lloyd's"
        );
        
        let claim_id = policy.submit_claim(25_000.0, "Data breach incident").unwrap();
        
        assert!(!claim_id.is_empty());
        assert_eq!(policy.status, PolicyStatus::ClaimInProgress);
        assert_eq!(policy.available_coverage(), 75_000.0);
    }

    #[test]
    fn test_coverage_verification() {
        let policy = InsurancePolicy::new(
            "agent-1",
            CoverageType::TransactionProtection,
            50_000.0,
            500.0,
            "AgentKern Insurance"
        );
        
        let verification = policy.verify_coverage("high_value_transfer", 10_000.0);
        
        assert!(verification.covered);
        assert_eq!(verification.available_coverage, 50_000.0);
    }

    // ========== NEW LEGAL ENTITY TESTS ==========

    #[test]
    fn test_legal_entity_creation() {
        let entity = AgentLegalEntity::new(
            "agent-1",
            "Agent Alpha LLC",
            LegalEntityType::WyomingDaoLlc,
            "Wyoming Registered Agents Inc"
        );
        
        assert_eq!(entity.jurisdiction, Jurisdiction::UsWyoming);
        assert_eq!(entity.status, EntityStatus::Pending);
        assert!(!entity.can_contract());
    }

    #[test]
    fn test_legal_entity_registration() {
        let mut entity = AgentLegalEntity::new(
            "agent-1",
            "Agent Beta DAO",
            LegalEntityType::WyomingDaoLlc,
            "WY Agents"
        );
        
        let reg_number = entity.register().unwrap();
        
        assert!(reg_number.starts_with("WY-"));
        assert_eq!(entity.status, EntityStatus::Active);
        assert!(entity.can_contract());
        assert!(entity.has_liability_shield());
    }

    #[test]
    fn test_entity_requirements() {
        let wyoming = LegalEntityType::WyomingDaoLlc;
        let reqs = wyoming.requirements();
        
        assert!(reqs.registered_agent_required);
        assert!(reqs.smart_contract_governance);
    }
}

