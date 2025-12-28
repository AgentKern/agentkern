//! Global Mesh Sync
//!
//! Multi-region CRDT synchronization with geo-fencing.
//! Per GLOBAL_GAPS.md: "Geo-Fenced Cells"

pub mod geo_fence;
pub mod sync;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use geo_fence::{GeoFence, ResidencyRule, TransferPolicy};
pub use sync::{ConflictResolution, MeshSync, SyncEvent};

/// A mesh cell representing a regional node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshCell {
    /// Unique cell ID
    pub id: String,
    /// Geographic region
    pub region: DataRegion,
    /// Cell endpoint
    pub endpoint: String,
    /// Is this cell active?
    pub active: bool,
    /// Last heartbeat timestamp
    pub last_heartbeat: u64,
}

/// Data region for sovereignty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataRegion {
    UsEast,
    UsWest,
    EuFrankfurt,
    EuIreland,
    AsiaSingapore,
    AsiaJapan,
    MenaRiyadh,
    MenaDubai,
    IndiaMumbai,
    Global,
}

impl DataRegion {
    /// Check if data localization is required.
    pub fn requires_localization(&self) -> bool {
        matches!(
            self,
            DataRegion::EuFrankfurt
                | DataRegion::EuIreland
                | DataRegion::MenaRiyadh
                | DataRegion::MenaDubai
                | DataRegion::IndiaMumbai
        )
    }

    /// Get the governing privacy law.
    pub fn privacy_law(&self) -> &'static str {
        match self {
            DataRegion::EuFrankfurt | DataRegion::EuIreland => "GDPR",
            DataRegion::MenaRiyadh | DataRegion::MenaDubai => "PDPL",
            DataRegion::IndiaMumbai => "DPDP",
            _ => "None",
        }
    }
}

/// Global mesh controller.
pub struct GlobalMesh {
    /// All registered cells
    cells: Arc<RwLock<HashMap<String, MeshCell>>>,
    /// Local cell ID
    local_cell_id: String,
    /// Geo-fence policy
    geo_fence: GeoFence,
    /// Sync engine
    sync: MeshSync,
}

impl GlobalMesh {
    /// Create a new mesh controller.
    pub fn new(local_cell_id: String, region: DataRegion) -> Self {
        Self {
            cells: Arc::new(RwLock::new(HashMap::new())),
            local_cell_id: local_cell_id.clone(),
            geo_fence: GeoFence::new(region),
            sync: MeshSync::new(local_cell_id),
        }
    }

    /// Register a remote cell.
    pub async fn register_cell(&self, cell: MeshCell) {
        let mut cells = self.cells.write().await;
        cells.insert(cell.id.clone(), cell);
    }

    /// Sync data to a target region (with geo-fence check).
    pub async fn sync_to_region(
        &self,
        data_id: &str,
        target_region: DataRegion,
        data: &[u8],
    ) -> Result<SyncResult, MeshError> {
        // Check geo-fence policy
        if !self.geo_fence.can_transfer(target_region, data_id) {
            return Err(MeshError::GeoFenceBlocked {
                reason: format!(
                    "Data {} cannot leave {}",
                    data_id,
                    self.geo_fence.local_region().privacy_law()
                ),
            });
        }

        // Find cells in target region
        let cells = self.cells.read().await;
        let target_cells: Vec<_> = cells
            .values()
            .filter(|c| c.region == target_region && c.active)
            .collect();

        if target_cells.is_empty() {
            return Err(MeshError::NoCellsInRegion(target_region));
        }

        // Sync to all target cells
        let mut synced_count = 0;
        for cell in target_cells {
            if self
                .sync
                .push_to_cell(&cell.endpoint, data_id, data)
                .await
                .is_ok()
            {
                synced_count += 1;
            }
        }

        Ok(SyncResult {
            data_id: data_id.to_string(),
            target_region,
            cells_synced: synced_count,
        })
    }

    /// Get all cells in a region.
    pub async fn cells_in_region(&self, region: DataRegion) -> Vec<MeshCell> {
        let cells = self.cells.read().await;
        cells
            .values()
            .filter(|c| c.region == region)
            .cloned()
            .collect()
    }
}

/// Sync result.
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub data_id: String,
    pub target_region: DataRegion,
    pub cells_synced: usize,
}

/// Mesh errors.
#[derive(Debug)]
pub enum MeshError {
    GeoFenceBlocked { reason: String },
    NoCellsInRegion(DataRegion),
    SyncFailed(String),
    ConnectionError(String),
}

impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GeoFenceBlocked { reason } => write!(f, "Geo-fence blocked: {}", reason),
            Self::NoCellsInRegion(r) => write!(f, "No active cells in region {:?}", r),
            Self::SyncFailed(msg) => write!(f, "Sync failed: {}", msg),
            Self::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
        }
    }
}

impl std::error::Error for MeshError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mesh_creation() {
        let mesh = GlobalMesh::new("cell-eu-1".to_string(), DataRegion::EuFrankfurt);
        assert!(mesh
            .cells_in_region(DataRegion::EuFrankfurt)
            .await
            .is_empty());
    }

    #[tokio::test]
    async fn test_register_cell() {
        let mesh = GlobalMesh::new("cell-eu-1".to_string(), DataRegion::EuFrankfurt);
        mesh.register_cell(MeshCell {
            id: "cell-us-1".to_string(),
            region: DataRegion::UsEast,
            endpoint: "https://us-east.mesh.local".to_string(),
            active: true,
            last_heartbeat: 0,
        })
        .await;

        let us_cells = mesh.cells_in_region(DataRegion::UsEast).await;
        assert_eq!(us_cells.len(), 1);
    }

    #[test]
    fn test_region_localization() {
        assert!(DataRegion::EuFrankfurt.requires_localization());
        assert!(DataRegion::MenaRiyadh.requires_localization());
        assert!(!DataRegion::UsEast.requires_localization());
    }
}
