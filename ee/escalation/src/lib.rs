//! Enterprise Escalation Integrations
//!
//! Per LICENSING.md: Native Slack, Teams, PagerDuty integrations
//! Per licensing_split.md: Pro/Enterprise tier

pub mod slack;
pub mod teams;
pub mod pagerduty;

// Re-exports
pub use slack::{SlackIntegration, SlackConfig};
pub use teams::{TeamsIntegration, TeamsConfig};
pub use pagerduty::{PagerDutyIntegration, PagerDutyConfig};
