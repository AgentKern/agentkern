//! Enterprise Escalation Integrations
//!
//! Per LICENSING.md: Native Slack, Teams, PagerDuty integrations
//! Per licensing_split.md: Pro/Enterprise tier

pub mod pagerduty;
pub mod slack;
pub mod teams;

// Re-exports
pub use pagerduty::{PagerDutyConfig, PagerDutyIntegration};
pub use slack::{SlackConfig, SlackIntegration};
pub use teams::{TeamsConfig, TeamsIntegration};
