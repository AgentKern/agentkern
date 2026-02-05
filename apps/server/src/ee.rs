use axum::{routing::{get, post}, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use agentkern_energy_ee::GridFactory;
use agentkern_sovereign_memory_ee::{MemoryEncryptor, EncryptionConfig, EncryptedBlob};

pub fn router() -> Router<()> {
    Router::new()
        .route("/energy/intensity", get(get_intensity))
        .route("/license/check", get(check_license))
        .route("/trust/stats", get(trust_stats))
        .route("/cloud/mesh", get(mesh_stats))
        .route("/memory/encrypt", post(memory_encrypt))
        .route("/memory/decrypt", post(memory_decrypt))
}

/// Get real-time energy intensity (Energy Pillar)
async fn get_intensity() -> Json<Value> {
    let api = GridFactory::get();
    // Default to US-East-1 for the API demo
    let data = api.get_intensity("us-east-1");
    Json(json!({
        "region": "us-east-1",
        "data": data,
        "source": "DemoGridApi (GreenOps)"
    }))
}

/// Check license status (Cloud Pillar)
async fn check_license() -> Json<Value> {
    // Using the License struct from agentkern-cloud
    // We assume the crate re-exports License or we use the internal logic
    // For now, we perform a safe check via env var as the License struct might be expensive to instantiate per request
    let key = std::env::var("AGENTKERN_LICENSE_KEY");
    match key {
        Ok(k) if !k.is_empty() => Json(json!({
            "status": "active",
            "tier": "enterprise_detected"
        })),
        _ => Json(json!({
            "status": "missing",
            "tier": "demo",
            "message": "Set AGENTKERN_LICENSE_KEY for Enterprise features"
        }))
    }
}

/// Get Trust Network stats (Trust Pillar)
/// Requires License
async fn trust_stats() -> Json<Value> {
    // Attempt to instantiate ephemeral trust network to read stats
    // In a real app, this would be a shared state singleton
    match agentkern_trust::TrustNetwork::new() {
        Ok(network) => {
            let stats = network.get_stats();
            Json(json!({
                "status": "online",
                "stats": stats
            }))
        },
        Err(e) => Json(json!({
            "status": "offline",
            "error": e.to_string(),
            "hint": "Enterprise License Required"
        }))
    }
}

/// Get Cloud Mesh stats (Cloud Pillar)
/// Requires License
async fn mesh_stats() -> Json<Value> {
    // Similar check for Mesh Coordinator
    // We use default config just to check license access
    let config = agentkern_cloud::MeshConfig::default();
    match agentkern_cloud::MeshCoordinator::new(config) {
        Ok(coordinator) => {
            Json(json!({
                "status": "online",
                "cells": coordinator.healthy_cell_count(),
                "cluster": "agentkern-mesh"
            }))
        },
        Err(e) => Json(json!({
            "status": "offline",
            "error": e.to_string(),
            "hint": "Enterprise License Required"
        }))
    }
}
/// Request to encrypt memory
#[derive(Deserialize)]
struct EncryptRequest {
    plaintext: String,
}

/// Request to decrypt memory
#[derive(Deserialize)]
struct DecryptRequest {
    blob: EncryptedBlob,
}

/// Encrypt agent memory (Sovereign Memory Pillar)
/// Requires License & AGENTKERN_LOCAL_KEK
async fn memory_encrypt(Json(payload): Json<EncryptRequest>) -> Json<Value> {
    let config = EncryptionConfig::default();
    match MemoryEncryptor::new(config) {
        Ok(encryptor) => {
            match encryptor.encrypt(payload.plaintext.as_bytes()).await {
                Ok(blob) => Json(json!({
                    "status": "encrypted",
                    "blob": blob
                })),
                Err(e) => Json(json!({
                    "status": "error",
                    "error": e.to_string()
                }))
            }
        },
        Err(e) => Json(json!({
            "status": "offline",
            "error": e.to_string(),
            "hint": "Enterprise License Required"
        }))
    }
}

/// Decrypt agent memory (Sovereign Memory Pillar)
/// Requires License & AGENTKERN_LOCAL_KEK
async fn memory_decrypt(Json(payload): Json<DecryptRequest>) -> Json<Value> {
    let config = EncryptionConfig::default();
    match MemoryEncryptor::new(config) {
        Ok(encryptor) => {
            match encryptor.decrypt(&payload.blob).await {
                Ok(plaintext) => {
                    let text = String::from_utf8_lossy(&plaintext).to_string();
                    Json(json!({
                        "status": "decrypted",
                        "plaintext": text
                    }))
                },
                Err(e) => Json(json!({
                    "status": "error",
                    "error": e.to_string()
                }))
            }
        },
        Err(e) => Json(json!({
            "status": "offline",
            "error": e.to_string(),
            "hint": "Enterprise License Required"
        }))
    }
}
