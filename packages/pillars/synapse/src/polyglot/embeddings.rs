//! Polyglot Embedding Engine
//!
//! Language-specific embedding adapters.

use super::Language;
use serde::{Deserialize, Serialize};

/// Embedding result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResult {
    /// The embedding vector
    pub vector: Vec<f32>,
    /// Detected language
    pub language: Language,
    /// Model used
    pub model: String,
    /// Dimensions
    pub dimensions: usize,
}

/// Polyglot embedder.
pub struct PolyglotEmbedder {
    /// Target language
    language: Language,
    /// Model identifier
    model: String,
    /// Embedding dimensions
    dimensions: usize,
}

impl PolyglotEmbedder {
    /// Create a new embedder for a language.
    pub fn new(language: Language) -> Self {
        let model = language.embedding_model().to_string();
        let dimensions = match language {
            Language::Arabic => 768,   // Jais
            Language::Chinese => 1024, // BGE
            _ => 1024,                 // E5-large
        };

        Self {
            language,
            model,
            dimensions,
        }
    }

    /// Embed text.
    pub async fn embed(&self, text: &str) -> EmbeddingResult {
        // In production, this calls the actual embedding API
        let vector = self.mock_embed(text);

        EmbeddingResult {
            vector,
            language: self.language,
            model: self.model.clone(),
            dimensions: self.dimensions,
        }
    }

    /// Mock embedding for testing.
    fn mock_embed(&self, text: &str) -> Vec<f32> {
        // Generate deterministic mock embeddings based on text hash
        let hash = text
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));

        (0..self.dimensions)
            .map(|i| {
                let seed = hash.wrapping_add(i as u64);
                ((seed % 1000) as f32 / 1000.0) - 0.5
            })
            .collect()
    }

    /// Get model name.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get dimensions.
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }
}

/// Cross-lingual intent verifier.
pub struct IntentVerifier {
    /// Mapping of intent IDs to embeddings
    intent_embeddings: Vec<(String, Vec<f32>)>,
}

impl IntentVerifier {
    /// Create a new intent verifier.
    pub fn new() -> Self {
        Self {
            intent_embeddings: Vec::new(),
        }
    }

    /// Register an intent with its embedding.
    pub fn register_intent(&mut self, intent_id: &str, embedding: Vec<f32>) {
        self.intent_embeddings
            .push((intent_id.to_string(), embedding));
    }

    /// Verify if translated text preserves original intent.
    pub fn verify_intent(
        &self,
        original: &EmbeddingResult,
        translated: &EmbeddingResult,
    ) -> IntentVerification {
        let similarity = Self::cosine_similarity(&original.vector, &translated.vector);

        IntentVerification {
            preserved: similarity > 0.85,
            similarity,
            warning: if similarity < 0.7 {
                Some("Significant semantic drift detected".to_string())
            } else {
                None
            },
        }
    }

    /// Cosine similarity between two vectors.
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }

        dot / (mag_a * mag_b)
    }
}

impl Default for IntentVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Intent verification result.
#[derive(Debug, Clone)]
pub struct IntentVerification {
    /// Was intent preserved?
    pub preserved: bool,
    /// Similarity score (0-1)
    pub similarity: f32,
    /// Warning message if drift detected
    pub warning: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_dimensions() {
        let embedder = PolyglotEmbedder::new(Language::English);
        let result = embedder.embed("Hello world").await;

        assert_eq!(result.dimensions, 1024);
        assert_eq!(result.vector.len(), 1024);
    }

    #[tokio::test]
    async fn test_arabic_embedder() {
        let embedder = PolyglotEmbedder::new(Language::Arabic);
        let result = embedder.embed("مرحبا").await;

        assert_eq!(result.model, "jais-embedding-v1");
        assert_eq!(result.language, Language::Arabic);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((IntentVerifier::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!(IntentVerifier::cosine_similarity(&a, &c).abs() < 0.001);
    }

    #[test]
    fn test_intent_preservation() {
        let verifier = IntentVerifier::new();

        let original = EmbeddingResult {
            vector: vec![0.5; 100],
            language: Language::English,
            model: "test".to_string(),
            dimensions: 100,
        };

        let translated = EmbeddingResult {
            vector: vec![0.5; 100],
            language: Language::Arabic,
            model: "test".to_string(),
            dimensions: 100,
        };

        let result = verifier.verify_intent(&original, &translated);
        assert!(result.preserved);
    }
}
