//! Retail Platform Adapter
//!
//! Generic trait for e-commerce platforms
//! Supports: Amazon SP-API, Shopify, Walmart, eBay, etc.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// Retail platform trait - implement for each marketplace.
#[async_trait]
pub trait RetailPlatform: Send + Sync {
    /// Platform identifier.
    fn platform_id(&self) -> &str;
    
    /// Platform type.
    fn platform_type(&self) -> PlatformType;
    
    /// Get listing by SKU.
    async fn get_listing(&self, sku: &str) -> Result<super::Listing, RetailError>;
    
    /// Update listing.
    async fn update_listing(&self, update: &super::ListingUpdate) -> Result<(), RetailError>;
    
    /// Update price.
    async fn update_price(&self, sku: &str, price: &super::PriceUpdate) -> Result<(), RetailError>;
    
    /// Get orders.
    async fn get_orders(&self, filter: &OrderFilter) -> Result<Vec<super::Order>, RetailError>;
    
    /// Acknowledge order.
    async fn acknowledge_order(&self, order_id: &str) -> Result<(), RetailError>;
    
    /// Submit fulfillment.
    async fn submit_fulfillment(&self, fulfillment: &super::Fulfillment) -> Result<(), RetailError>;
    
    /// Get inventory level.
    async fn get_inventory(&self, sku: &str) -> Result<InventoryLevel, RetailError>;
    
    /// Update inventory.
    async fn update_inventory(&self, sku: &str, quantity: i32) -> Result<(), RetailError>;
}

/// Platform type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlatformType {
    /// Amazon Marketplace (SP-API)
    AmazonMarketplace,
    /// Shopify Store
    Shopify,
    /// Walmart Marketplace
    Walmart,
    /// eBay
    Ebay,
    /// Magento/Adobe Commerce
    Magento,
    /// WooCommerce
    WooCommerce,
    /// Custom/Other
    Custom,
}

/// Platform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// Platform type
    pub platform: PlatformType,
    /// Seller/Merchant ID
    pub seller_id: String,
    /// Marketplace ID (for multi-region)
    pub marketplace_id: Option<String>,
    /// API endpoint
    pub endpoint: String,
    /// Authentication config
    pub auth: AuthConfig,
    /// Rate limit (requests per second)
    pub rate_limit: Option<u32>,
}

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    /// OAuth 2.0
    OAuth2 {
        client_id: String,
        client_secret_ref: String,
        refresh_token_ref: String,
    },
    /// API Key
    ApiKey {
        key_ref: String,
    },
    /// HMAC Signature
    HmacSignature {
        access_key_ref: String,
        secret_key_ref: String,
    },
}

/// Order filter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrderFilter {
    /// Status filter
    pub status: Option<Vec<super::OrderStatus>>,
    /// Created after
    pub created_after: Option<String>,
    /// Created before
    pub created_before: Option<String>,
    /// Max results
    pub limit: Option<u32>,
}

/// Inventory level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryLevel {
    pub sku: String,
    pub quantity: i32,
    pub reserved: i32,
    pub available: i32,
    pub last_updated: String,
}

/// Retail error.
#[derive(Debug, thiserror::Error)]
pub enum RetailError {
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("License error: {0}")]
    LicenseError(#[from] agentkern_connectors_ee::license::LicenseError),
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_type() {
        assert_eq!(PlatformType::Shopify, PlatformType::Shopify);
        assert_ne!(PlatformType::Shopify, PlatformType::AmazonMarketplace);
    }

    #[test]
    fn test_inventory_level() {
        let inv = InventoryLevel {
            sku: "SKU123".into(),
            quantity: 100,
            reserved: 10,
            available: 90,
            last_updated: "2025-12-26T12:00:00Z".into(),
        };
        assert_eq!(inv.available, 90);
    }
}
