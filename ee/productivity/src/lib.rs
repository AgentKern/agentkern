//! Productivity Platform Integration
//!
//! Connectors for email, calendar, document management
//! Supports: Microsoft 365, Google Workspace, Zoho, etc.
//!
//! Graceful Degradation: Works with credentials, demo mode without

pub mod demo;
pub mod outlook;
pub mod sharepoint;

// Generic names - outlook.rs/sharepoint.rs are implementation details
// Could add google_workspace.rs, zoho.rs, etc.

pub use demo::{DemoProductivity, ProductivityFactory};
pub use outlook::{CalendarEvent, EmailMessage, OutlookConfig, OutlookConnector};
pub use sharepoint::{Document, SearchResult, SharePointConfig, SharePointConnector};
