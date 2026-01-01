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

pub mod export;
pub mod gdpr;
pub mod import;
pub mod layers;
pub mod schema;

// Re-exports
pub use export::{ExportFormat, ExportOptions, PassportExporter};
pub use gdpr::{DataCategory, GdprExport, ProcessingEvent};
pub use import::{ImportOptions, ImportResult, PassportImporter};
pub use layers::{EpisodicMemory, MemoryLayers, PreferenceMemory, SemanticMemory, SkillMemory};
pub use schema::{MemoryPassport, PassportError, PassportVersion};
