//! Memory Passport - Portable agent state for cross-cloud sovereignty
//!
//! Per Strategic Roadmap: Universal Memory Passport for agent portability
//! Per MANDATE.md Section 2: GDPR compliance (Right to Portability)
//!
//! Features:
//! - DID-anchored identity
//! - Hierarchical memory (episodic, semantic, skills, preferences)
//! - Encrypted cross-cloud migration
//! - GDPR Article 20 compliance export

pub mod schema;
pub mod layers;
pub mod export;
pub mod import;
pub mod gdpr;

// Re-exports
pub use schema::{MemoryPassport, PassportVersion, PassportError};
pub use layers::{MemoryLayers, EpisodicMemory, SemanticMemory, SkillMemory, PreferenceMemory};
pub use export::{PassportExporter, ExportOptions, ExportFormat};
pub use import::{PassportImporter, ImportOptions, ImportResult};
pub use gdpr::{GdprExport, DataCategory, ProcessingEvent};
