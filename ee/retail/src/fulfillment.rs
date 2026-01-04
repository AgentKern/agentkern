//! Fulfillment Management
//!
//! Shipping, tracking, and delivery management

use serde::{Deserialize, Serialize};

/// Fulfillment request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fulfillment {
    /// Order ID
    pub order_id: String,
    /// Items to fulfill
    pub items: Vec<FulfillmentItem>,
    /// Carrier
    pub carrier: String,
    /// Shipping method
    pub shipping_method: Option<String>,
    /// Tracking number
    pub tracking_number: String,
    /// Ship date
    pub ship_date: String,
}

/// Fulfillment item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentItem {
    /// Order item ID
    pub item_id: String,
    /// Quantity fulfilled
    pub quantity: u32,
}

/// Shipment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipmentStatus {
    LabelCreated,
    PickedUp,
    InTransit,
    OutForDelivery,
    Delivered,
    Attempted,
    Exception,
    Returned,
}

/// Tracking info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingInfo {
    /// Tracking number
    pub tracking_number: String,
    /// Carrier
    pub carrier: String,
    /// Status
    pub status: ShipmentStatus,
    /// Estimated delivery
    pub estimated_delivery: Option<String>,
    /// Events
    pub events: Vec<TrackingEvent>,
}

/// Tracking event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEvent {
    /// Timestamp
    pub timestamp: String,
    /// Status
    pub status: ShipmentStatus,
    /// Location
    pub location: Option<String>,
    /// Message
    pub message: String,
}

/// Common carriers.
pub mod carriers {
    pub const UPS: &str = "UPS";
    pub const FEDEX: &str = "FedEx";
    pub const USPS: &str = "USPS";
    pub const DHL: &str = "DHL";
    pub const AMAZON: &str = "AMZN";
    pub const ONTRAC: &str = "OnTrac";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shipment_status() {
        assert_ne!(ShipmentStatus::InTransit, ShipmentStatus::Delivered);
    }

    #[test]
    fn test_fulfillment() {
        let f = Fulfillment {
            order_id: "ORD123".into(),
            items: vec![FulfillmentItem {
                item_id: "ITEM1".into(),
                quantity: 1,
            }],
            carrier: carriers::UPS.into(),
            shipping_method: Some("Ground".into()),
            tracking_number: "1Z999AA10123456784".into(),
            ship_date: "2025-12-26".into(),
        };
        assert_eq!(f.carrier, "UPS");
    }
}
