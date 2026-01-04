//! Productivity Platform Integration
//!
//! Connectors for email, calendar, document management
//! Supports: Microsoft 365, Google Workspace, Zoho, etc.
//!
//! Graceful Degradation: Works with credentials, demo mode without

pub mod outlook;
pub mod sharepoint;
pub mod demo;

// Generic names - outlook.rs/sharepoint.rs are implementation details
// Could add google_workspace.rs, zoho.rs, etc.

pub use outlook::{OutlookConnector, OutlookConfig, EmailMessage, CalendarEvent};
pub use sharepoint::{SharePointConnector, SharePointConfig, Document, SearchResult};
pub use demo::{DemoProductivity, ProductivityFactory};

