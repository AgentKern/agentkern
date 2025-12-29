//! AgentKern-Gate: PCI-DSS Compliance Module
//!
//! Per EXECUTION_MANDATE.md §2: "PCI-DSS: Automated compliance for financial transactions"
//!
//! PCI-DSS v4.0.1 requirements (March 2025 deadline)
//!
//! Features:
//! - Card data detection (PAN, CVV, expiry)
//! - Tokenization support
//! - Encryption requirements
//! - Audit logging
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_gate::pci::{PciValidator, CardData};
//!
//! let validator = PciValidator::new();
//! validator.scan_for_card_data(&text)?;
//! let token = validator.tokenize(&card_data)?;
//! ```

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// PCI-DSS compliance error.
#[derive(Debug, Error)]
pub enum PciError {
    #[error("Card data detected in cleartext: {data_type}")]
    CardDataExposed { data_type: String },
    #[error("CVV/CVC must not be stored after authorization")]
    CvvStorageViolation,
    #[error("Encryption required for card data in transit")]
    EncryptionRequired,
    #[error("Card data must be masked for display: {pan}")]
    MaskingRequired { pan: String },
    #[error("Tokenization failed: {reason}")]
    TokenizationFailed { reason: String },
    #[error("Invalid card format")]
    InvalidCardFormat,
}

/// Card data types per PCI-DSS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardDataType {
    /// Primary Account Number (PAN)
    Pan,
    /// Expiry date
    ExpiryDate,
    /// Cardholder name
    CardholderName,
    /// Service code
    ServiceCode,
    /// CVV/CVC (sensitive authentication data)
    Cvv,
    /// PIN (sensitive authentication data)
    Pin,
    /// Full magnetic stripe data
    TrackData,
}

impl CardDataType {
    /// Check if this is sensitive authentication data (SAD).
    pub fn is_sensitive_auth_data(&self) -> bool {
        matches!(self, Self::Cvv | Self::Pin | Self::TrackData)
    }
}

/// Card data scan result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardScanResult {
    /// Was card data detected?
    pub contains_card_data: bool,
    /// Types of card data found
    pub data_types_found: Vec<CardDataType>,
    /// Is sensitive auth data present?
    pub has_sad: bool,
    /// Risk level
    pub risk: RiskLevel,
    /// Recommended action
    pub action: String,
}

/// Risk level for PCI compliance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// No card data found
    None,
    /// Potential card data, needs review
    Low,
    /// Card data confirmed
    Medium,
    /// Sensitive auth data present (CRITICAL)
    Critical,
}

/// Tokenized card reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardToken {
    /// Token ID (safe to store)
    pub token: String,
    /// Masked PAN (for display)
    pub masked_pan: String,
    /// Card brand
    pub brand: CardBrand,
    /// Expiry month
    pub exp_month: u8,
    /// Expiry year
    pub exp_year: u16,
    /// Token creation timestamp
    pub created_at: u64,
}

/// Card brand detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardBrand {
    Visa,
    Mastercard,
    Amex,
    Discover,
    UnionPay,
    Jcb,
    Unknown,
}

impl CardBrand {
    /// Detect card brand from PAN prefix.
    pub fn from_pan(pan: &str) -> Self {
        let digits: String = pan.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.is_empty() {
            return Self::Unknown;
        }

        match digits.chars().next().unwrap() {
            '4' => Self::Visa,
            '5' => Self::Mastercard,
            '3' => {
                if digits.starts_with("34") || digits.starts_with("37") {
                    Self::Amex
                } else {
                    Self::Jcb
                }
            }
            '6' => Self::Discover,
            _ => Self::Unknown,
        }
    }
}

/// PCI-DSS compliance validator.
#[derive(Debug, Default)]
pub struct PciValidator {
    /// Allow test card numbers
    allow_test_numbers: bool,
}

impl PciValidator {
    /// Create a new PCI validator.
    pub fn new() -> Self {
        Self {
            allow_test_numbers: false,
        }
    }

    /// Create a validator that allows test card numbers.
    pub fn with_test_mode() -> Self {
        Self {
            allow_test_numbers: true,
        }
    }

    /// Check if test mode is enabled.
    pub fn is_test_mode(&self) -> bool {
        self.allow_test_numbers
    }

    /// Scan text for card data.
    pub fn scan_for_card_data(&self, text: &str) -> CardScanResult {
        let mut data_types = Vec::new();

        // Extract digits only for PAN detection
        let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();

        // Check for PAN (13-19 digits)
        if digits.len() >= 13 && digits.len() <= 19
            && self.luhn_check(&digits) {
                data_types.push(CardDataType::Pan);
            }

        // Check for CVV (3-4 digits in context)
        let text_lower = text.to_lowercase();
        if text_lower.contains("cvv") || text_lower.contains("cvc") || text_lower.contains("cid") {
            data_types.push(CardDataType::Cvv);
        }

        // Check for expiry date patterns
        if text.contains('/') {
            let parts: Vec<&str> = text.split('/').collect();
            if parts.len() >= 2 {
                let first = parts[0].trim();
                let second = parts[1].trim();
                if first.len() <= 2 && (second.len() == 2 || second.len() == 4)
                    && first
                        .parse::<u8>()
                        .map(|m| (1..=12).contains(&m))
                        .unwrap_or(false)
                    {
                        data_types.push(CardDataType::ExpiryDate);
                    }
            }
        }

        let has_sad = data_types.iter().any(|d| d.is_sensitive_auth_data());
        let contains_card_data = !data_types.is_empty();

        let risk = if has_sad {
            RiskLevel::Critical
        } else if data_types.contains(&CardDataType::Pan) {
            RiskLevel::Medium
        } else if contains_card_data {
            RiskLevel::Low
        } else {
            RiskLevel::None
        };

        CardScanResult {
            contains_card_data,
            data_types_found: data_types,
            has_sad,
            risk,
            action: match risk {
                RiskLevel::Critical => "IMMEDIATELY remove sensitive auth data".to_string(),
                RiskLevel::Medium => "Tokenize PAN before storage".to_string(),
                RiskLevel::Low => "Review for potential card data".to_string(),
                RiskLevel::None => "No action required".to_string(),
            },
        }
    }

    /// Luhn algorithm check for valid card numbers.
    fn luhn_check(&self, digits: &str) -> bool {
        let mut sum = 0;
        let mut double = false;

        for c in digits.chars().rev() {
            if let Some(d) = c.to_digit(10) {
                let mut d = d;
                if double {
                    d *= 2;
                    if d > 9 {
                        d -= 9;
                    }
                }
                sum += d;
                double = !double;
            } else {
                return false;
            }
        }

        sum % 10 == 0
    }

    /// Mask a PAN for safe display (show first 6 and last 4).
    pub fn mask_pan(&self, pan: &str) -> String {
        let digits: String = pan.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() < 13 {
            return "INVALID".to_string();
        }

        let first6 = &digits[..6];
        let last4 = &digits[digits.len() - 4..];
        let masked_len = digits.len() - 10;

        format!("{}{}{}", first6, "*".repeat(masked_len), last4)
    }

    /// Tokenize card data (returns a safe token).
    pub fn tokenize(&self, pan: &str, exp_month: u8, exp_year: u16) -> Result<CardToken, PciError> {
        let digits: String = pan.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() < 13 || digits.len() > 19 {
            return Err(PciError::InvalidCardFormat);
        }

        if !self.luhn_check(&digits) {
            return Err(PciError::InvalidCardFormat);
        }

        let token = format!(
            "tok_{}_{}",
            uuid::Uuid::new_v4().to_string().replace('-', ""),
            &digits[digits.len() - 4..]
        );

        Ok(CardToken {
            token,
            masked_pan: self.mask_pan(&digits),
            brand: CardBrand::from_pan(&digits),
            exp_month,
            exp_year,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Validate that CVV is not being stored (PCI-DSS 3.2.2).
    pub fn reject_cvv_storage(&self, data: &str) -> Result<(), PciError> {
        let text_lower = data.to_lowercase();

        if text_lower.contains("cvv") || text_lower.contains("cvc") {
            // Check if it's just a field name or actual data
            let has_digits = data.chars().any(|c| c.is_ascii_digit());
            if has_digits {
                return Err(PciError::CvvStorageViolation);
            }
        }

        Ok(())
    }

    /// Validate data for PCI compliance before storage.
    pub fn validate_for_storage(&self, data: &str) -> Result<(), PciError> {
        // Check for CVV
        self.reject_cvv_storage(data)?;

        // Scan for unencrypted card data
        let scan = self.scan_for_card_data(data);

        if scan.has_sad {
            return Err(PciError::CvvStorageViolation);
        }

        if scan.data_types_found.contains(&CardDataType::Pan) {
            return Err(PciError::CardDataExposed {
                data_type: "PAN".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luhn_check() {
        let validator = PciValidator::new();

        // Valid test card numbers
        assert!(validator.luhn_check("4111111111111111")); // Visa
        assert!(validator.luhn_check("5500000000000004")); // Mastercard

        // Invalid
        assert!(!validator.luhn_check("1234567890123456"));
    }

    #[test]
    fn test_card_brand_detection() {
        assert_eq!(CardBrand::from_pan("4111111111111111"), CardBrand::Visa);
        assert_eq!(
            CardBrand::from_pan("5500000000000004"),
            CardBrand::Mastercard
        );
        assert_eq!(CardBrand::from_pan("371449635398431"), CardBrand::Amex);
    }

    #[test]
    fn test_pan_masking() {
        let validator = PciValidator::new();
        let masked = validator.mask_pan("4111111111111111");

        assert_eq!(masked, "411111******1111");
    }

    #[test]
    fn test_tokenization() {
        let validator = PciValidator::new();
        let result = validator.tokenize("4111111111111111", 12, 2025);

        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(token.token.starts_with("tok_"));
        assert_eq!(token.brand, CardBrand::Visa);
        assert_eq!(token.masked_pan, "411111******1111");
    }

    #[test]
    fn test_cvv_storage_rejection() {
        let validator = PciValidator::new();

        assert!(validator.reject_cvv_storage("cvv: 123").is_err());
        assert!(validator.reject_cvv_storage("cvc_field").is_ok()); // No digits
    }

    #[test]
    fn test_scan_for_card_data() {
        let validator = PciValidator::new();

        let result = validator.scan_for_card_data("Card: 4111-1111-1111-1111, CVV: 123");

        assert!(result.contains_card_data);
        assert!(result.has_sad);
        assert_eq!(result.risk, RiskLevel::Critical);
    }

    #[test]
    fn test_no_card_data() {
        let validator = PciValidator::new();
        let result = validator.scan_for_card_data("No payment info here");

        assert!(!result.contains_card_data);
        assert_eq!(result.risk, RiskLevel::None);
    }
}
