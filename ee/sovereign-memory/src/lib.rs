//! Sovereign Memory Enterprise Features
//!
//! Cross-cloud memory encryption and migration for Memory Passports.
//! KMS integration for AWS, GCP, and Azure.

pub mod aws_kms;
pub mod encryption;
pub mod migration;

pub use aws_kms::{AwsKmsWrapper, KmsError};
pub use encryption::{
    EncryptedBlob, EncryptionConfig, EncryptionError, KeyProvider, MemoryEncryptor,
};
pub use migration::{CloudMigrator, CloudTarget, MigrationConfig, MigrationError};
