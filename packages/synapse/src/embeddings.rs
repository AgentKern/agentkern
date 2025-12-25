//! VeriMantle-Synapse: Polyglot Embedding Configuration
//!
//! Per GLOBAL_GAPS.md §2: Native language support in Synapse
//!
//! Features:
//! - Configurable embedding models per region
//! - Native embeddings for Arabic (Jais), Japanese, Hindi, etc.
//! - Cross-lingual intent verification
//!
//! # Example
//!
//! ```rust,ignore
//! use verimantle_synapse::embeddings::{EmbeddingConfig, EmbeddingProvider};
//!
//! let config = EmbeddingConfig::new()
//!     .with_provider(DataRegion::Mena, EmbeddingProvider::Jais)
//!     .with_provider(DataRegion::AsiaPac, EmbeddingProvider::Multilingual);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Embedding model provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EmbeddingProvider {
    /// OpenAI text-embedding-3-small (default, English-optimized)
    OpenAI,
    /// Sentence Transformers multilingual (paraphrase-multilingual-MiniLM)
    Multilingual,
    /// Jais (Arabic-optimized, UAE TII)
    Jais,
    /// BGE-M3 (BAAI, supports 100+ languages)
    BgeM3,
    /// E5-Multilingual (Microsoft)
    E5Multilingual,
    /// Custom local model (path to ONNX)
    Custom(String),
}

impl Default for EmbeddingProvider {
    fn default() -> Self {
        Self::Multilingual
    }
}

impl EmbeddingProvider {
    /// Get the model name for this provider.
    pub fn model_name(&self) -> &str {
        match self {
            Self::OpenAI => "text-embedding-3-small",
            Self::Multilingual => "paraphrase-multilingual-MiniLM-L12-v2",
            Self::Jais => "jais-13b-chat",
            Self::BgeM3 => "BAAI/bge-m3",
            Self::E5Multilingual => "intfloat/multilingual-e5-large",
            Self::Custom(path) => path,
        }
    }

    /// Get the embedding dimension for this provider.
    pub fn dimension(&self) -> usize {
        match self {
            Self::OpenAI => 1536,
            Self::Multilingual => 384,
            Self::Jais => 5120,
            Self::BgeM3 => 1024,
            Self::E5Multilingual => 1024,
            Self::Custom(_) => 384, // Default assumption
        }
    }

    /// Get supported languages for this provider.
    pub fn supported_languages(&self) -> Vec<&'static str> {
        match self {
            Self::OpenAI => vec!["en"],
            Self::Multilingual => vec!["en", "de", "fr", "es", "it", "pt", "nl", "pl", "ru", "ja", "zh", "ko"],
            Self::Jais => vec!["ar", "en"],
            Self::BgeM3 => vec!["en", "zh", "ar", "ja", "ko", "hi", "th", "vi", "id", "ms"],
            Self::E5Multilingual => vec!["en", "de", "fr", "es", "it", "pt", "nl", "pl", "ru", "ja", "zh", "ko", "ar"],
            Self::Custom(_) => vec!["*"], // Assume all
        }
    }
}

/// Data region enum (mirrors gate::types::DataRegion for convenience).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SynapseRegion {
    Us,
    Eu,
    Cn,
    Mena,
    India,
    Brazil,
    AsiaPac,
    Africa,
    Global,
}

/// Configuration for polyglot embeddings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Default embedding provider
    pub default_provider: EmbeddingProvider,
    /// Region-specific overrides
    #[serde(default)]
    pub region_providers: HashMap<SynapseRegion, EmbeddingProvider>,
    /// Cache embeddings locally
    #[serde(default = "default_true")]
    pub cache_enabled: bool,
    /// Maximum cache size in entries
    #[serde(default = "default_cache_size")]
    pub max_cache_size: usize,
}

fn default_true() -> bool { true }
fn default_cache_size() -> usize { 10_000 }

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingConfig {
    /// Create a new embedding configuration with sensible defaults.
    pub fn new() -> Self {
        let mut region_providers = HashMap::new();
        
        // Per GLOBAL_GAPS.md: Native embeddings for regions
        region_providers.insert(SynapseRegion::Mena, EmbeddingProvider::Jais);
        region_providers.insert(SynapseRegion::Cn, EmbeddingProvider::BgeM3);
        region_providers.insert(SynapseRegion::AsiaPac, EmbeddingProvider::BgeM3);
        region_providers.insert(SynapseRegion::India, EmbeddingProvider::E5Multilingual);
        
        Self {
            default_provider: EmbeddingProvider::Multilingual,
            region_providers,
            cache_enabled: true,
            max_cache_size: 10_000,
        }
    }

    /// Set the default provider.
    pub fn with_default(mut self, provider: EmbeddingProvider) -> Self {
        self.default_provider = provider;
        self
    }

    /// Add a region-specific provider.
    pub fn with_provider(mut self, region: SynapseRegion, provider: EmbeddingProvider) -> Self {
        self.region_providers.insert(region, provider);
        self
    }

    /// Get the provider for a specific region.
    pub fn get_provider(&self, region: SynapseRegion) -> &EmbeddingProvider {
        self.region_providers
            .get(&region)
            .unwrap_or(&self.default_provider)
    }

    /// Get the embedding dimension for a specific region.
    pub fn get_dimension(&self, region: SynapseRegion) -> usize {
        self.get_provider(region).dimension()
    }
}

/// Polyglot embedding service.
#[derive(Debug)]
pub struct PolyglotEmbedder {
    config: EmbeddingConfig,
}

impl Default for PolyglotEmbedder {
    fn default() -> Self {
        Self::new(EmbeddingConfig::default())
    }
}

impl PolyglotEmbedder {
    /// Create a new polyglot embedder with the given configuration.
    pub fn new(config: EmbeddingConfig) -> Self {
        Self { config }
    }

    /// Get the provider for a region.
    pub fn provider_for(&self, region: SynapseRegion) -> &EmbeddingProvider {
        self.config.get_provider(region)
    }

    /// Generate embeddings for text in a specific region.
    /// 
    /// Note: This is a placeholder that returns a zero vector.
    /// Real implementation would call the actual embedding API.
    pub async fn embed(&self, text: &str, region: SynapseRegion) -> Vec<f32> {
        let provider = self.provider_for(region);
        let dimension = provider.dimension();
        
        // Placeholder: return zero vector
        // Real implementation would:
        // 1. Call OpenAI API, or
        // 2. Run local ONNX model, or
        // 3. Call other provider APIs
        
        tracing::debug!(
            provider = %provider.model_name(),
            region = ?region,
            text_len = text.len(),
            "Generating embedding"
        );
        
        vec![0.0; dimension]
    }

    /// Check if a language is supported for a region.
    pub fn supports_language(&self, language: &str, region: SynapseRegion) -> bool {
        let provider = self.provider_for(region);
        let supported = provider.supported_languages();
        supported.contains(&"*") || supported.contains(&language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_config_defaults() {
        let config = EmbeddingConfig::new();
        
        // MENA should use Jais
        assert_eq!(config.get_provider(SynapseRegion::Mena), &EmbeddingProvider::Jais);
        
        // US should use default (Multilingual)
        assert_eq!(config.get_provider(SynapseRegion::Us), &EmbeddingProvider::Multilingual);
    }

    #[test]
    fn test_embedding_provider_dimensions() {
        assert_eq!(EmbeddingProvider::OpenAI.dimension(), 1536);
        assert_eq!(EmbeddingProvider::Multilingual.dimension(), 384);
        assert_eq!(EmbeddingProvider::Jais.dimension(), 5120);
    }

    #[test]
    fn test_embedding_provider_languages() {
        let jais = EmbeddingProvider::Jais;
        let langs = jais.supported_languages();
        assert!(langs.contains(&"ar"));
        assert!(langs.contains(&"en"));
    }

    #[test]
    fn test_custom_config() {
        let config = EmbeddingConfig::new()
            .with_default(EmbeddingProvider::OpenAI)
            .with_provider(SynapseRegion::Africa, EmbeddingProvider::E5Multilingual);
        
        assert_eq!(config.get_provider(SynapseRegion::Eu), &EmbeddingProvider::OpenAI);
        assert_eq!(config.get_provider(SynapseRegion::Africa), &EmbeddingProvider::E5Multilingual);
    }

    #[test]
    fn test_polyglot_embedder() {
        let embedder = PolyglotEmbedder::default();
        
        assert!(embedder.supports_language("ar", SynapseRegion::Mena));
        assert!(embedder.supports_language("ja", SynapseRegion::AsiaPac));
    }

    #[tokio::test]
    async fn test_embed_placeholder() {
        let embedder = PolyglotEmbedder::default();
        let embedding = embedder.embed("مرحبا بالعالم", SynapseRegion::Mena).await;
        
        // Jais has 5120 dimensions
        assert_eq!(embedding.len(), 5120);
    }
}
