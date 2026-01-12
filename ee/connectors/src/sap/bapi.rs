//! SAP BAPI Caller
//!
//! Business Application Programming Interface for SAP

use super::{BapiResult, BapiReturn, RfcConnection, SapError};
use std::collections::HashMap;

/// BAPI caller wrapping RFC connection.
pub struct BapiCaller<'a> {
    rfc: &'a RfcConnection,
}

impl<'a> BapiCaller<'a> {
    /// Create new BAPI caller.
    pub fn new(rfc: &'a RfcConnection) -> Self {
        Self { rfc }
    }

    /// Call a BAPI function with realistic validation and error simulation.
    pub fn call(
        &self,
        bapi_name: &str,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<BapiResult, SapError> {
        if !self.rfc.is_connected() {
            return Err(SapError::NotConnected);
        }

        // Simulate name validation
        if bapi_name.is_empty() || !bapi_name.starts_with("BAPI_") {
            return Err(SapError::BapiError(format!(
                "Invalid BAPI name: {}",
                bapi_name
            )));
        }

        // Simulate mandatory parameter check (e.g., almost all BAPIs need some ID)
        let mut return_table = Vec::new();
        let mut success = true;

        if params.is_empty() && !bapi_name.contains("GETLIST") {
            success = false;
            return_table.push(BapiReturn {
                message_type: "E".to_string(),
                message_id: "AR".to_string(),
                message_number: "001".to_string(),
                message: "Mandatory parameters are missing".to_string(),
            });
        }

        // Simulate backend error if a specific "FAULTY" parameter is passed (for testing)
        if params.contains_key("FAULTY") {
            success = false;
            return_table.push(BapiReturn {
                message_type: "E".to_string(),
                message_id: "SY".to_string(),
                message_number: "500".to_string(),
                message: "Internal SAP System Error during BAPI execution".to_string(),
            });
        }

        if success {
            return_table.push(BapiReturn {
                message_type: "S".to_string(),
                message_id: "00".to_string(),
                message_number: "000".to_string(),
                message: format!("BAPI {} executed successfully", bapi_name),
            });
        }

        Ok(BapiResult {
            success,
            return_table,
            export_params: params,
            tables: HashMap::new(),
        })
    }

    /// Get BAPI list for object.
    pub fn get_bapi_list(&self, object_type: &str) -> Result<Vec<BapiInfo>, SapError> {
        // Would call BAPI_OBJECT_GET_BAPI_LIST
        Ok(vec![
            BapiInfo {
                name: format!("BAPI_{}_CREATE", object_type.to_uppercase()),
                description: format!("Create {}", object_type),
            },
            BapiInfo {
                name: format!("BAPI_{}_CHANGE", object_type.to_uppercase()),
                description: format!("Change {}", object_type),
            },
            BapiInfo {
                name: format!("BAPI_{}_GETDETAIL", object_type.to_uppercase()),
                description: format!("Get {} details", object_type),
            },
        ])
    }

    /// Commit BAPI transaction.
    pub fn commit(&self) -> Result<(), SapError> {
        // Would call BAPI_TRANSACTION_COMMIT
        Ok(())
    }

    /// Rollback BAPI transaction.
    pub fn rollback(&self) -> Result<(), SapError> {
        // Would call BAPI_TRANSACTION_ROLLBACK
        Ok(())
    }
}

/// BAPI information.
#[derive(Debug, Clone)]
pub struct BapiInfo {
    pub name: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sap::{RfcConnection, SapConfig};

    #[test]
    fn test_bapi_validation() {
        // We use a mock-like approach since RfcConnection is also a structural mock in dev
        let config = SapConfig::default();
        let rfc = RfcConnection::new(&config, "password").unwrap();
        let caller = BapiCaller::new(&rfc);

        // Test invalid name
        let result = caller.call("INVALID_NAME", HashMap::new());
        assert!(matches!(result, Err(SapError::BapiError(_))));

        // Test missing parameters
        let result = caller.call("BAPI_USER_CREATE", HashMap::new());
        let bapi_res = result.unwrap();
        assert!(!bapi_res.success);
        assert_eq!(bapi_res.return_table[0].message_type, "E");

        // Test faulty parameter
        let mut params = HashMap::new();
        params.insert("FAULTY".to_string(), serde_json::json!(true));
        let result = caller.call("BAPI_USER_CREATE", params);
        let bapi_res = result.unwrap();
        assert!(!bapi_res.success);
        assert!(bapi_res.return_table.iter().any(|r| r.message_id == "SY"));

        // Test success
        let mut params = HashMap::new();
        params.insert("USERNAME".to_string(), serde_json::json!("AGENT_007"));
        let result = caller.call("BAPI_USER_CREATE", params);
        let bapi_res = result.unwrap();
        assert!(bapi_res.success);
    }
}
