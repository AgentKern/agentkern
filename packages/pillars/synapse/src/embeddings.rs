//! AgentKern-Synapse: Polyglot Embedding Configuration
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
//! use agentkern_synapse::embeddings::{EmbeddingConfig, EmbeddingProvider};
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
            Self::Multilingual => vec![
                "en", "de", "fr", "es", "it", "pt", "nl", "pl", "ru", "ja", "zh", "ko",
            ],
            Self::Jais => vec!["ar", "en"],
            Self::BgeM3 => vec!["en", "zh", "ar", "ja", "ko", "hi", "th", "vi", "id", "ms"],
            Self::E5Multilingual => vec![
                "en", "de", "fr", "es", "it", "pt", "nl", "pl", "ru", "ja", "zh", "ko", "ar",
            ],
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

fn default_true() -> bool {
    true
}
fn default_cache_size() -> usize {
    10_000
}

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

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("API request failed: {0}")]
    ApiError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("JSON parse error: {0}")]
    ParseError(String),
    #[error("Missing embedding in response")]
    MissingEmbedding,
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
    pub async fn embed(
        &self,
        text: &str,
        region: SynapseRegion,
    ) -> Result<Vec<f32>, EmbeddingError> {
        let provider = self.provider_for(region);

        // Check for API key
        let api_key = std::env::var("AGENTKERN_EMBEDDINGS_API_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .map_err(|_| {
                EmbeddingError::ConfigError(
                    "No API key found in AGENTKERN_EMBEDDINGS_API_KEY or OPENAI_API_KEY".into(),
                )
            })?;

        if api_key.is_empty() {
            return Err(EmbeddingError::ConfigError("Empty API key found".into()));
        }

        // Try real API call
        self.call_embedding_api(&api_key, text, provider).await
    }

    /// Call actual embedding API.
    #[cfg(not(test))]
    async fn call_embedding_api(
        &self,
        api_key: &str,
        text: &str,
        provider: &EmbeddingProvider,
    ) -> Result<Vec<f32>, EmbeddingError> {
        let client = reqwest::Client::new();

        let response = client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": provider.model_name(),
                "input": text
            }))
            .send()
            .await
            .map_err(|e| EmbeddingError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(EmbeddingError::ApiError(format!(
                "API error: {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ParseError(e.to_string()))?;

        let embedding = body["data"][0]["embedding"]
            .as_array()
            .ok_or(EmbeddingError::MissingEmbedding)?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        Ok(embedding)
    }

    #[cfg(test)]
    async fn call_embedding_api(
        &self,
        _api_key: &str,
        text: &str,
        provider: &EmbeddingProvider,
    ) -> Result<Vec<f32>, EmbeddingError> {
        // Return deterministic mock vector in tests
        let dimension = provider.dimension();
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        let mut embedding = Vec::with_capacity(dimension);
        let mut seed = hash;
        for _ in 0..dimension {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = ((seed >> 33) as f32) / (u32::MAX as f32) - 0.5;
            embedding.push(val);
        }

        Ok(embedding)
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
        assert_eq!(
            config.get_provider(SynapseRegion::Mena),
            &EmbeddingProvider::Jais
        );

        // US should use default (Multilingual)
        assert_eq!(
            config.get_provider(SynapseRegion::Us),
            &EmbeddingProvider::Multilingual
        );
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

        assert_eq!(
            config.get_provider(SynapseRegion::Eu),
            &EmbeddingProvider::OpenAI
        );
        assert_eq!(
            config.get_provider(SynapseRegion::Africa),
            &EmbeddingProvider::E5Multilingual
        );
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
        std::env::set_var("AGENTKERN_EMBEDDINGS_API_KEY", "test");
        let embedding = embedder
            .embed("مرحبا بالعالم", SynapseRegion::Mena)
            .await
            .unwrap();

        // Jais has 5120 dimensions
        assert_eq!(embedding.len(), 5120);
    }
}
