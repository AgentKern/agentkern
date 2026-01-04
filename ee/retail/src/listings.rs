//! Listing Management
//!
//! Product listings, pricing, and catalog management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Product listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    /// SKU (Stock Keeping Unit)
    pub sku: String,
    /// ASIN, GTIN, or platform-specific ID
    pub product_id: String,
    /// Product ID type
    pub product_id_type: ProductIdType,
    /// Title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Bullet points
    pub bullet_points: Vec<String>,
    /// Price
    pub price: Price,
    /// Images
    pub images: Vec<String>,
    /// Attributes
    pub attributes: HashMap<String, serde_json::Value>,
    /// Status
    pub status: ListingStatus,
}

/// Product ID type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductIdType {
    Asin,
    Gtin,
    Upc,
    Ean,
    Isbn,
    Sku,
    Custom,
}

/// Price.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub amount: f64,
    pub currency: String,
    pub sale_price: Option<f64>,
    pub sale_start: Option<String>,
    pub sale_end: Option<String>,
}

/// Listing status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListingStatus {
    Active,
    Inactive,
    Suppressed,
    Pending,
    Error,
}

/// Listing update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListingUpdate {
    pub sku: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub bullet_points: Option<Vec<String>>,
    pub images: Option<Vec<String>>,
    pub attributes: Option<HashMap<String, serde_json::Value>>,
}

/// Price update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub amount: f64,
    pub currency: String,
    pub sale_price: Option<f64>,
    pub sale_start: Option<String>,
    pub sale_end: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price() {
        let price = Price {
            amount: 29.99,
            currency: "USD".into(),
            sale_price: Some(19.99),
            sale_start: None,
            sale_end: None,
        };
        assert_eq!(price.currency, "USD");
    }
}
