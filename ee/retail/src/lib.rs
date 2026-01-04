//! Retail Platform Bridge
//!
//! Generic e-commerce integration (Amazon SP-API, Shopify, Walmart, etc.)
//! Technology-focused, vendor-neutral design
//!
//! Graceful Degradation: Works with credentials, demo mode without

pub mod adapter;
pub mod listings;
pub mod orders;
pub mod fulfillment;
pub mod demo;

pub use adapter::{RetailPlatform, PlatformConfig, RetailError, PlatformType};
pub use listings::{Listing, ListingUpdate, PriceUpdate};
pub use orders::{Order, OrderItem, OrderStatus};
pub use fulfillment::{Fulfillment, ShipmentStatus, TrackingInfo};
pub use demo::{DemoRetailPlatform, RetailFactory};

