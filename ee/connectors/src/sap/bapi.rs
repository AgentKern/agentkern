//! SAP BAPI Caller
//!
//! Business Application Programming Interface for SAP

use super::{RfcConnection, SapError, BapiResult, BapiReturn};
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
    
    /// Call a BAPI function.
    pub fn call(&self, bapi_name: &str, params: HashMap<String, serde_json::Value>) -> Result<BapiResult, SapError> {
        if !self.rfc.is_connected() {
            return Err(SapError::NotConnected);
        }
        
        // Production would call RFC with BAPI parameters
        Ok(BapiResult {
            success: true,
            return_table: vec![BapiReturn {
                message_type: "S".to_string(),
                message_id: "00".to_string(),
                message_number: "000".to_string(),
                message: format!("BAPI {} executed successfully", bapi_name),
            }],
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
