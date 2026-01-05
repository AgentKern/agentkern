//! Retail Platform Bridge
//!
//! Generic e-commerce integration (Amazon SP-API, Shopify, Walmart, etc.)
//! Technology-focused, vendor-neutral design
//!
//! Graceful Degradation: Works with credentials, demo mode without

pub mod adapter;
pub mod demo;
pub mod fulfillment;
pub mod listings;
pub mod orders;

pub use adapter::{PlatformConfig, PlatformType, RetailError, RetailPlatform};
pub use demo::{DemoRetailPlatform, RetailFactory};
pub use fulfillment::{Fulfillment, ShipmentStatus, TrackingInfo};
pub use listings::{Listing, ListingUpdate, PriceUpdate};
pub use orders::{Order, OrderItem, OrderStatus};
