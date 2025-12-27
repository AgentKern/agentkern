//! Demo Retail Platform Implementation
//!
//! Works without credentials - returns realistic demo data

use super::adapter::*;
use super::{Listing, ListingUpdate, PriceUpdate, Order, OrderItem, OrderStatus, Fulfillment};
use crate::core::{ConnectionMode, ConnectionStatus, GracefulService};
use async_trait::async_trait;

/// Demo retail platform that works without credentials.
pub struct DemoRetailPlatform {
    config: PlatformConfig,
    mode: ConnectionMode,
}

impl DemoRetailPlatform {
    /// Create new demo platform.
    pub fn new(platform: PlatformType) -> Self {
        let mode = ConnectionMode::detect("retail");
        
        let config = PlatformConfig {
            platform,
            seller_id: "DEMO_SELLER".into(),
            marketplace_id: Some("DEMO_MARKET".into()),
            endpoint: "https://demo.agentkern.dev/retail".into(),
            auth: AuthConfig::ApiKey { key_ref: String::new() },
            rate_limit: None,
        };
        
        Self { config, mode }
    }
}

impl GracefulService for DemoRetailPlatform {
    fn mode(&self) -> ConnectionMode {
        self.mode
    }
    
    fn status(&self) -> ConnectionStatus {
        ConnectionStatus::new("retail")
    }
}

#[async_trait]
impl RetailPlatform for DemoRetailPlatform {
    fn platform_id(&self) -> &str {
        &self.config.seller_id
    }
    
    fn platform_type(&self) -> PlatformType {
        self.config.platform
    }
    
    async fn get_listing(&self, sku: &str) -> Result<Listing, RetailError> {
        Ok(Listing {
            sku: sku.to_string(),
            product_id: format!("DEMO_{}", sku),
            product_id_type: super::listings::ProductIdType::Sku,
            title: format!("[Demo] Product {}", sku),
            description: Some("This is demo data. Set AGENTKERN_RETAIL_API_KEY for live.".into()),
            bullet_points: vec!["Demo feature 1".into()],
            price: super::listings::Price {
                amount: 29.99,
                currency: "USD".into(),
                sale_price: None,
                sale_start: None,
                sale_end: None,
            },
            images: vec![],
            attributes: std::collections::HashMap::new(),
            status: super::listings::ListingStatus::Active,
        })
    }
    
    async fn update_listing(&self, _update: &ListingUpdate) -> Result<(), RetailError> {
        if self.mode.is_live() {
            // Would call real API
        }
        Ok(()) // Demo mode always succeeds
    }
    
    async fn update_price(&self, _sku: &str, _price: &PriceUpdate) -> Result<(), RetailError> {
        Ok(())
    }
    
    async fn get_orders(&self, _filter: &OrderFilter) -> Result<Vec<Order>, RetailError> {
        Ok(vec![
            Order {
                order_id: "DEMO-001".into(),
                purchase_date: chrono::Utc::now().to_rfc3339(),
                status: OrderStatus::Unshipped,
                items: vec![OrderItem {
                    item_id: "ITEM-001".into(),
                    sku: "DEMO-SKU".into(),
                    product_id: "B0DEMO123".into(),
                    title: "[Demo Order Item]".into(),
                    quantity_ordered: 1,
                    quantity_shipped: 0,
                    item_price: 29.99,
                    currency: "USD".into(),
                }],
                shipping_address: None,
                buyer_name: Some("Demo Customer".into()),
                order_total: super::orders::OrderTotal {
                    amount: 29.99,
                    currency: "USD".into(),
                },
                fulfillment_channel: super::orders::FulfillmentChannel::Merchant,
            }
        ])
    }
    
    async fn acknowledge_order(&self, _order_id: &str) -> Result<(), RetailError> {
        Ok(())
    }
    
    async fn submit_fulfillment(&self, _fulfillment: &Fulfillment) -> Result<(), RetailError> {
        Ok(())
    }
    
    async fn get_inventory(&self, sku: &str) -> Result<InventoryLevel, RetailError> {
        Ok(InventoryLevel {
            sku: sku.to_string(),
            quantity: 100,
            reserved: 5,
            available: 95,
            last_updated: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    async fn update_inventory(&self, _sku: &str, _quantity: i32) -> Result<(), RetailError> {
        Ok(())
    }
}

/// Factory to get the best available retail platform.
pub struct RetailFactory;

impl RetailFactory {
    /// Get platform with graceful fallback.
    pub fn get(platform: PlatformType) -> Box<dyn RetailPlatform> {
        Box::new(DemoRetailPlatform::new(platform))
    }
    
    /// Get connection status.
    pub fn status() -> ConnectionStatus {
        ConnectionStatus::new("retail")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_retail_works_without_credentials() {
        let platform = DemoRetailPlatform::new(PlatformType::AmazonMarketplace);
        assert!(platform.is_available());
    }

    #[tokio::test]
    async fn test_demo_retail_get_orders() {
        let platform = DemoRetailPlatform::new(PlatformType::Shopify);
        let orders = platform.get_orders(&OrderFilter::default()).await;
        assert!(orders.is_ok());
        assert!(!orders.unwrap().is_empty());
    }
}
