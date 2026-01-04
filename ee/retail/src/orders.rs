//! Order Management
//!
//! Order processing and status management

use serde::{Deserialize, Serialize};

/// Order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Order ID
    pub order_id: String,
    /// Purchase date
    pub purchase_date: String,
    /// Status
    pub status: OrderStatus,
    /// Items
    pub items: Vec<OrderItem>,
    /// Shipping address
    pub shipping_address: Option<Address>,
    /// Buyer info
    pub buyer_name: Option<String>,
    /// Order total
    pub order_total: OrderTotal,
    /// Fulfillment channel
    pub fulfillment_channel: FulfillmentChannel,
}

/// Order item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    /// Item ID
    pub item_id: String,
    /// SKU
    pub sku: String,
    /// Product ID (ASIN, etc.)
    pub product_id: String,
    /// Title
    pub title: String,
    /// Quantity ordered
    pub quantity_ordered: u32,
    /// Quantity shipped
    pub quantity_shipped: u32,
    /// Item price
    pub item_price: f64,
    /// Currency
    pub currency: String,
}

/// Order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Unshipped,
    PartiallyShipped,
    Shipped,
    Delivered,
    Canceled,
    Returned,
}

/// Address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub name: String,
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country_code: String,
    pub phone: Option<String>,
}

/// Order total.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderTotal {
    pub amount: f64,
    pub currency: String,
}

/// Fulfillment channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FulfillmentChannel {
    /// Seller fulfilled
    Merchant,
    /// Platform fulfilled (FBA, SFP, etc.)
    Platform,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_status() {
        assert_ne!(OrderStatus::Pending, OrderStatus::Shipped);
    }

    #[test]
    fn test_order_item() {
        let item = OrderItem {
            item_id: "ITEM1".into(),
            sku: "SKU123".into(),
            product_id: "B0EXAMPLE".into(),
            title: "Test Product".into(),
            quantity_ordered: 2,
            quantity_shipped: 0,
            item_price: 29.99,
            currency: "USD".into(),
        };
        assert_eq!(item.quantity_ordered, 2);
    }
}
