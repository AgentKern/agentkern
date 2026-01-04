//! Sovereign Memory Enterprise Features
//!
//! Cross-cloud memory encryption and migration for Memory Passports.
//! KMS integration for AWS, GCP, and Azure.

pub mod encryption;
pub mod migration;
pub mod aws_kms;

pub use encryption::{MemoryEncryptor, EncryptionConfig, EncryptedBlob, KeyProvider, EncryptionError};
pub use migration::{CloudMigrator, MigrationConfig, CloudTarget, MigrationError};
pub use aws_kms::{AwsKmsWrapper, KmsError};
