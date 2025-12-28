#![allow(unused)]
//! AgentKern Enterprise: Usage-Based Billing & Metering
//!
//! Per Deep Analysis: "No billing/metering for usage-based pricing"
//! Per LICENSING_STRATEGY.md: "Usage-Based (x402)"
//!
//! **License**: AgentKern Enterprise License
//!
//! Features:
//! - Usage event recording
//! - Real-time metering
//! - Stripe Meter API integration
//! - Billing alerts
//! - Invoice generation
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_billing::{Meter, UsageEvent};
//!
//! let mut meter = Meter::new("org-123")?;
//! meter.record(UsageEvent::api_call("policy.check", 1))?;
//! ```

use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod license {
    #[derive(Debug, thiserror::Error)]
    pub enum LicenseError {
        #[error("Enterprise license required for billing")]
        LicenseRequired,
    }

    pub fn require(feature: &str) -> Result<(), LicenseError> {
        let key =
            std::env::var("AGENTKERN_LICENSE_KEY").map_err(|_| LicenseError::LicenseRequired)?;

        if key.is_empty() {
            return Err(LicenseError::LicenseRequired);
        }

        tracing::debug!(feature = %feature, "Enterprise billing feature accessed");
        Ok(())
    }
}

/// Usage metric types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    /// API calls
    ApiCalls,
    /// Policy checks
    PolicyChecks,
    /// Neural inferences
    NeuralInferences,
    /// Agent actions
    AgentActions,
    /// Storage used (bytes)
    StorageBytes,
    /// Compute time (ms)
    ComputeMs,
    /// Data transfer (bytes)
    TransferBytes,
    /// Tokens processed (LLM)
    TokensProcessed,
}

impl MetricType {
    /// Get default price per unit (in cents).
    pub fn default_price_cents(&self) -> f64 {
        match self {
            Self::ApiCalls => 0.001,           // $0.00001 per call
            Self::PolicyChecks => 0.0001,      // $0.000001 per check
            Self::NeuralInferences => 0.01,    // $0.0001 per inference
            Self::AgentActions => 0.001,       // $0.00001 per action
            Self::StorageBytes => 0.00000001,  // $0.10 per GB/month
            Self::ComputeMs => 0.00001,        // $0.0001 per second
            Self::TransferBytes => 0.00000008, // $0.08 per GB
            Self::TokensProcessed => 0.00003,  // Similar to OpenAI
        }
    }

    /// Get unit name.
    pub fn unit_name(&self) -> &'static str {
        match self {
            Self::ApiCalls => "calls",
            Self::PolicyChecks => "checks",
            Self::NeuralInferences => "inferences",
            Self::AgentActions => "actions",
            Self::StorageBytes => "bytes",
            Self::ComputeMs => "ms",
            Self::TransferBytes => "bytes",
            Self::TokensProcessed => "tokens",
        }
    }
}

/// Usage event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    /// Event ID
    pub id: String,
    /// Tenant ID (customer identifier)
    pub tenant_id: String,
    /// Metric type
    pub metric: MetricType,
    /// Quantity
    pub quantity: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Event properties
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

impl UsageEvent {
    /// Create a new usage event.
    pub fn new(tenant_id: impl Into<String>, metric: MetricType, quantity: u64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tenant_id: tenant_id.into(),
            metric,
            quantity,
            timestamp: Utc::now(),
            properties: HashMap::new(),
        }
    }

    /// Create an API call event.
    pub fn api_call(tenant_id: impl Into<String>, endpoint: &str) -> Self {
        let mut event = Self::new(tenant_id, MetricType::ApiCalls, 1);
        event
            .properties
            .insert("endpoint".to_string(), endpoint.to_string());
        event
    }

    /// Create a policy check event.
    pub fn policy_check(tenant_id: impl Into<String>, policy_id: &str) -> Self {
        let mut event = Self::new(tenant_id, MetricType::PolicyChecks, 1);
        event
            .properties
            .insert("policy_id".to_string(), policy_id.to_string());
        event
    }

    /// Create a neural inference event.
    pub fn neural_inference(tenant_id: impl Into<String>, model: &str, tokens: u64) -> Self {
        let mut event = Self::new(tenant_id, MetricType::NeuralInferences, 1);
        event
            .properties
            .insert("model".to_string(), model.to_string());
        event
            .properties
            .insert("tokens".to_string(), tokens.to_string());
        event
    }

    /// Create a compute event.
    pub fn compute(tenant_id: impl Into<String>, duration_ms: u64) -> Self {
        Self::new(tenant_id, MetricType::ComputeMs, duration_ms)
    }

    /// Add property.
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }
}

/// Aggregated usage for a period.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageAggregate {
    /// Total quantity
    pub total_quantity: u64,
    /// Event count
    pub event_count: u64,
    /// First event timestamp
    pub first_event: Option<DateTime<Utc>>,
    /// Last event timestamp
    pub last_event: Option<DateTime<Utc>>,
}

impl UsageAggregate {
    /// Add an event to the aggregate.
    pub fn add(&mut self, quantity: u64, timestamp: DateTime<Utc>) {
        self.total_quantity += quantity;
        self.event_count += 1;

        if self.first_event.is_none() || timestamp < self.first_event.unwrap() {
            self.first_event = Some(timestamp);
        }
        if self.last_event.is_none() || timestamp > self.last_event.unwrap() {
            self.last_event = Some(timestamp);
        }
    }
}

/// Billing period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BillingPeriod {
    pub year: i32,
    pub month: u32,
}

impl BillingPeriod {
    /// Get current billing period.
    pub fn current() -> Self {
        let now = Utc::now();
        Self {
            year: now.year(),
            month: now.month(),
        }
    }

    /// Get period key (for storage).
    pub fn key(&self) -> String {
        format!("{:04}-{:02}", self.year, self.month)
    }
}

/// Usage meter.
pub struct Meter {
    tenant_id: String,
    events: Vec<UsageEvent>,
    aggregates: HashMap<(BillingPeriod, MetricType), UsageAggregate>,
    prices: HashMap<MetricType, f64>,
}

impl Meter {
    /// Create a new meter (requires enterprise license).
    pub fn new(tenant_id: impl Into<String>) -> Result<Self, license::LicenseError> {
        license::require("BILLING")?;

        let mut prices = HashMap::new();
        for metric in [
            MetricType::ApiCalls,
            MetricType::PolicyChecks,
            MetricType::NeuralInferences,
            MetricType::AgentActions,
            MetricType::StorageBytes,
            MetricType::ComputeMs,
            MetricType::TransferBytes,
            MetricType::TokensProcessed,
        ] {
            prices.insert(metric, metric.default_price_cents());
        }

        Ok(Self {
            tenant_id: tenant_id.into(),
            events: Vec::new(),
            aggregates: HashMap::new(),
            prices,
        })
    }

    /// Record a usage event.
    pub fn record(&mut self, event: UsageEvent) {
        let period = BillingPeriod {
            year: event.timestamp.year(),
            month: event.timestamp.month(),
        };

        let aggregate = self.aggregates.entry((period, event.metric)).or_default();

        aggregate.add(event.quantity, event.timestamp);

        self.events.push(event);
    }

    /// Get usage for current period.
    pub fn current_usage(&self) -> HashMap<MetricType, u64> {
        let period = BillingPeriod::current();

        self.aggregates
            .iter()
            .filter(|((p, _), _)| *p == period)
            .map(|((_, metric), agg)| (*metric, agg.total_quantity))
            .collect()
    }

    /// Calculate current period cost in cents.
    pub fn current_cost_cents(&self) -> f64 {
        let usage = self.current_usage();

        usage
            .iter()
            .map(|(metric, quantity)| {
                let price = self.prices.get(metric).copied().unwrap_or(0.0);
                *quantity as f64 * price
            })
            .sum()
    }

    /// Get events (for export).
    pub fn events(&self) -> &[UsageEvent] {
        &self.events
    }

    /// Set custom price for a metric.
    pub fn set_price(&mut self, metric: MetricType, price_cents: f64) {
        self.prices.insert(metric, price_cents);
    }
}

/// Invoice line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub description: String,
    pub metric: MetricType,
    pub quantity: u64,
    pub unit_price_cents: f64,
    pub amount_cents: f64,
}

/// Invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub tenant_id: String,
    pub period: BillingPeriod,
    pub line_items: Vec<InvoiceLineItem>,
    pub subtotal_cents: f64,
    pub tax_cents: f64,
    pub total_cents: f64,
    pub status: InvoiceStatus,
    pub created_at: DateTime<Utc>,
}

/// Invoice status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InvoiceStatus {
    Draft,
    Open,
    Paid,
    Void,
    Uncollectible,
}

impl Invoice {
    /// Generate invoice from meter.
    pub fn generate(meter: &Meter, period: BillingPeriod) -> Self {
        let mut line_items = Vec::new();
        let mut subtotal = 0.0;

        for ((p, metric), aggregate) in &meter.aggregates {
            if *p != period {
                continue;
            }

            let price = meter.prices.get(metric).copied().unwrap_or(0.0);
            let amount = aggregate.total_quantity as f64 * price;

            line_items.push(InvoiceLineItem {
                description: format!("{} ({})", metric.unit_name(), aggregate.total_quantity),
                metric: *metric,
                quantity: aggregate.total_quantity,
                unit_price_cents: price,
                amount_cents: amount,
            });

            subtotal += amount;
        }

        // No tax for simplicity (would be calculated based on location)
        let tax = 0.0;

        Self {
            id: format!("inv_{}", uuid::Uuid::new_v4()),
            tenant_id: meter.tenant_id.clone(),
            period,
            line_items,
            subtotal_cents: subtotal,
            tax_cents: tax,
            total_cents: subtotal + tax,
            status: InvoiceStatus::Draft,
            created_at: Utc::now(),
        }
    }
}

/// Billing alert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAlert {
    /// Alert ID
    pub id: String,
    /// Tenant ID
    pub tenant_id: String,
    /// Metric to monitor (None = total spend)
    pub metric: Option<MetricType>,
    /// Threshold value
    pub threshold: f64,
    /// Alert type
    pub alert_type: AlertType,
    /// Is enabled
    pub enabled: bool,
    /// Notification channels
    pub notify: Vec<String>,
}

/// Alert type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    /// Alert when usage reaches threshold
    UsageThreshold,
    /// Alert when spend reaches threshold (in cents)
    SpendThreshold,
    /// Alert when projected spend reaches threshold
    ProjectedSpendThreshold,
}

/// Stripe Meter integration.
/// Uses reqwest to call Stripe Billing Meter API v2024-12-18.
#[derive(Debug)]
pub struct StripeMeterSync {
    /// Stripe API key
    api_key: String,
    /// Meter ID in Stripe
    meter_id: String,
    /// HTTP client
    client: reqwest::Client,
    /// Events pending sync
    pending_events: Vec<UsageEvent>,
}

impl StripeMeterSync {
    /// Create a new Stripe sync.
    pub fn new(api_key: impl Into<String>, meter_id: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            meter_id: meter_id.into(),
            client: reqwest::Client::new(),
            pending_events: Vec::new(),
        }
    }

    /// Queue an event for sync.
    pub fn queue(&mut self, event: UsageEvent) {
        self.pending_events.push(event);
    }

    /// Sync pending events to Stripe Billing Meter API.
    pub async fn sync(&mut self) -> Result<usize, BillingError> {
        let count = self.pending_events.len();

        if count == 0 {
            return Ok(0);
        }

        // Build events payload for Stripe API
        let events: Vec<_> = self
            .pending_events
            .iter()
            .map(|e| {
                serde_json::json!({
                    "event_name": format!("{}_{}", e.metric.unit_name(), e.tenant_id),
                    "payload": {
                        "stripe_customer_id": e.tenant_id,
                        "value": e.quantity.to_string(),
                        "timestamp": e.timestamp.timestamp()
                    }
                })
            })
            .collect();

        // POST to Stripe Billing Meter Events API
        let response = self
            .client
            .post(format!(
                "https://api.stripe.com/v1/billing/meters/{}/events",
                self.meter_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Stripe-Version", "2024-12-18")
            .form(&[("events", serde_json::to_string(&events).unwrap_or_default())])
            .send()
            .await
            .map_err(|e| BillingError::StripeError {
                message: format!("HTTP error: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(BillingError::StripeError {
                message: format!("Stripe API error {}: {}", status, body),
            });
        }

        tracing::info!(
            meter_id = %self.meter_id,
            count = count,
            "Synced events to Stripe Billing Meter API"
        );

        self.pending_events.clear();
        Ok(count)
    }

    /// Get pending event count.
    pub fn pending_count(&self) -> usize {
        self.pending_events.len()
    }
}

/// Billing errors.
#[derive(Debug, thiserror::Error)]
pub enum BillingError {
    #[error("Stripe API error: {message}")]
    StripeError { message: String },
    #[error("Invalid metric: {name}")]
    InvalidMetric { name: String },
    #[error("Invoice not found")]
    InvoiceNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_prices() {
        assert!(MetricType::ApiCalls.default_price_cents() > 0.0);
        assert_eq!(MetricType::ApiCalls.unit_name(), "calls");
    }

    #[test]
    fn test_usage_event() {
        let event = UsageEvent::api_call("org-123", "/api/v1/check");

        assert_eq!(event.metric, MetricType::ApiCalls);
        assert_eq!(event.quantity, 1);
        assert!(event.properties.contains_key("endpoint"));
    }

    #[test]
    fn test_meter_requires_license() {
        unsafe {
            std::env::remove_var("AGENTKERN_LICENSE_KEY");
        }
        let result = Meter::new("org-123");
        assert!(result.is_err());
    }

    #[test]
    fn test_meter_with_license() {
        unsafe {
            std::env::set_var("AGENTKERN_LICENSE_KEY", "test-license");
        }

        let mut meter = Meter::new("org-123").unwrap();

        meter.record(UsageEvent::api_call("org-123", "/api/v1/check"));
        meter.record(UsageEvent::policy_check("org-123", "policy-1"));

        let usage = meter.current_usage();
        assert_eq!(usage.get(&MetricType::ApiCalls), Some(&1));

        unsafe {
            std::env::remove_var("AGENTKERN_LICENSE_KEY");
        }
    }

    #[test]
    fn test_invoice_generation() {
        unsafe {
            std::env::set_var("AGENTKERN_LICENSE_KEY", "test-license");
        }

        let mut meter = Meter::new("org-123").unwrap();

        for _ in 0..100 {
            meter.record(UsageEvent::api_call("org-123", "/api/v1/check"));
        }

        let invoice = Invoice::generate(&meter, BillingPeriod::current());

        assert!(!invoice.line_items.is_empty());
        assert!(invoice.total_cents > 0.0);

        unsafe {
            std::env::remove_var("AGENTKERN_LICENSE_KEY");
        }
    }

    #[test]
    fn test_billing_period() {
        let period = BillingPeriod::current();
        assert!(period.year >= 2024);
        assert!(period.month >= 1 && period.month <= 12);
        assert!(period.key().contains('-'));
    }
}
