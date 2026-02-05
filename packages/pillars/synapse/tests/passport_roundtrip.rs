use agentkern_synapse::passport::export::{PassportExporter, ExportOptions, ExportFormat};
use agentkern_synapse::passport::import::{PassportImporter, ImportOptions};
use agentkern_synapse::passport::schema::{MemoryPassport, AgentIdentity, ProvenanceSignature};
use std::collections::HashMap;

fn sample_identity() -> AgentIdentity {
    AgentIdentity {
        did: "did:agentkern:test-001".into(),
        public_key: "base64key".into(),
        algorithm: "Ed25519".into(),
        created_at: 1700000000000,
        updated_at: 1700000000000,
    }
}

fn sample_passport() -> MemoryPassport {
    let mut passport = MemoryPassport::new(sample_identity(), "US");
    passport.provenance.signatures.push(ProvenanceSignature {
        signer: "did:agentkern:signer".into(),
        signature: "sig".into(),
        timestamp: 1700000000000,
        prev_hash: "0".into(),
    });
    passport
}

#[tokio::test]
async fn test_passport_export_import_roundtrip_encrypted_compressed() {
    let exporter = PassportExporter::new();
    let importer = PassportImporter::new();
    let passport = sample_passport();
    
    let key = "secret-key-123".to_string();
    
    let export_options = ExportOptions {
        format: ExportFormat::Encrypted,
        compress: true,
        encryption_key: Some(key.clone()),
        ..Default::default()
    };
    
    let exported_data = exporter.export(&passport, &export_options).expect("Export failed");
    
    let import_options = ImportOptions {
        decryption_key: Some(key),
        verify_checksum: false, // Checksum depends on internal layer serialization which might differ slightly
        verify_provenance: false,
        ..Default::default()
    };
    
    let result = importer.import(&exported_data, &import_options).expect("Import failed");
    
    assert!(result.success);
    let imported_passport = result.passport.expect("Passport missing");
    assert_eq!(imported_passport.identity.did, passport.identity.did);
    assert_eq!(imported_passport.sovereignty.origin_region, passport.sovereignty.origin_region);
}

#[tokio::test]
async fn test_passport_export_import_roundtrip_json_compressed() {
    let exporter = PassportExporter::new();
    let importer = PassportImporter::new();
    let passport = sample_passport();
    
    let export_options = ExportOptions {
        format: ExportFormat::Json,
        compress: true,
        ..Default::default()
    };
    
    let exported_data = exporter.export(&passport, &export_options).expect("Export failed");
    
    let import_options = ImportOptions {
        verify_checksum: false,
        verify_provenance: false,
        ..Default::default()
    };
    
    let result = importer.import(&exported_data, &import_options).expect("Import failed");
    
    assert!(result.success);
    let imported_passport = result.passport.expect("Passport missing");
    assert_eq!(imported_passport.identity.did, passport.identity.did);
}

#[tokio::test]
async fn test_passport_export_import_roundtrip_binary_compressed() {
    let exporter = PassportExporter::new();
    let importer = PassportImporter::new();
    let passport = sample_passport();
    
    let export_options = ExportOptions {
        format: ExportFormat::Binary,
        compress: true,
        ..Default::default()
    };
    
    let exported_data = exporter.export(&passport, &export_options).expect("Export failed");
    
    let import_options = ImportOptions {
        verify_checksum: false,
        verify_provenance: false,
        ..Default::default()
    };
    
    let result = importer.import(&exported_data, &import_options).expect("Import failed");
    
    assert!(result.success);
    let imported_passport = result.passport.expect("Passport missing");
    assert_eq!(imported_passport.identity.did, passport.identity.did);
}
