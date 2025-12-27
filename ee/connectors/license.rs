//! License Check for Enterprise Features
//!
//! All enterprise features must call check_license() before use.

use std::env;
use thiserror::Error;

/// License errors.
#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("Enterprise license required. Set AGENTKERN_LICENSE_KEY.")]
    LicenseRequired,
    
    #[error("Invalid license key")]
    InvalidLicense,
    
    #[error("License expired")]
    LicenseExpired,
    
    #[error("Feature not included in license: {0}")]
    FeatureNotIncluded(String),
}

/// Check if valid enterprise license is present.
pub fn check_license() -> Result<(), LicenseError> {
    let key = env::var("AGENTKERN_LICENSE_KEY")
        .map_err(|_| LicenseError::LicenseRequired)?;
    
    // Validate key format (production would verify with license server)
    if key.len() < 32 {
        return Err(LicenseError::InvalidLicense);
    }
    
    // Check expiration (production would decode JWT or check server)
    // For now, accept any valid-looking key
    
    Ok(())
}

/// Check if specific feature is licensed.
pub fn check_feature_license(feature: &str) -> Result<(), LicenseError> {
    check_license()?;
    
    // Production would verify feature entitlement
    // For now, allow all features with valid license
    let _ = feature;
    
    Ok(())
}

/// Get license tier.
pub fn get_license_tier() -> Option<LicenseTier> {
    let key = env::var("AGENTKERN_LICENSE_KEY").ok()?;
    
    // Parse tier from key (simplified)
    if key.contains("ENTERPRISE") || key.starts_with("ENT-") {
        Some(LicenseTier::Enterprise)
    } else if key.contains("PRO") || key.starts_with("PRO-") {
        Some(LicenseTier::Pro)
    } else if key.len() >= 32 {
        Some(LicenseTier::Pro)
    } else {
        None
    }
}

/// License tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseTier {
    Pro,
    Enterprise,
}

impl LicenseTier {
    /// Check if tier includes a feature.
    pub fn includes(&self, feature: &str) -> bool {
        match self {
            LicenseTier::Enterprise => true, // Enterprise includes everything
            LicenseTier::Pro => {
                // Pro tier features
                matches!(feature, 
                    "salesforce" | "dynamics365" | "slack" | "teams"
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_license() {
        env::remove_var("AGENTKERN_LICENSE_KEY");
        assert!(check_license().is_err());
    }

    #[test]
    fn test_license_tier() {
        assert!(LicenseTier::Enterprise.includes("sap"));
        assert!(LicenseTier::Enterprise.includes("swift"));
        assert!(LicenseTier::Pro.includes("slack"));
        assert!(!LicenseTier::Pro.includes("swift"));
    }
}
