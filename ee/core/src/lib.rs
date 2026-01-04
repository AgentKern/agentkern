//! Enterprise Core Utilities
//!
//! Shared patterns for all enterprise features

pub mod connection;

pub use connection::{
    ConnectionMode, 
    ConnectionStatus, 
    GracefulService, 
    GracefulResult
};
