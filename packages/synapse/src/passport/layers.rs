//! Memory Layers - Hierarchical agent memory storage
//!
//! Implements the 4-layer memory model:
//! 1. Episodic - Specific interaction history
//! 2. Semantic - General knowledge and facts
//! 3. Skills - Learned capabilities
//! 4. Preferences - User-specific settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// All memory layers combined.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryLayers {
    /// Episodic memory (interaction history)
    pub episodic: EpisodicMemory,
    /// Semantic memory (knowledge)
    pub semantic: SemanticMemory,
    /// Skill memory (capabilities)
    pub skills: SkillMemory,
    /// Preference memory (user settings)
    pub preferences: PreferenceMemory,
}

impl MemoryLayers {
    /// Create empty memory layers.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get total entry count.
    pub fn total_entries(&self) -> usize {
        self.episodic.entries.len()
            + self.semantic.facts.len()
            + self.skills.skills.len()
            + self.preferences.items.len()
    }
    
    /// Check if all layers are empty.
    pub fn is_empty(&self) -> bool {
        self.total_entries() == 0
    }
}

// ============================================================================
// EPISODIC MEMORY - Interaction History
// ============================================================================

/// Episodic memory - stores specific interactions and events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EpisodicMemory {
    /// Memory entries
    pub entries: Vec<EpisodicEntry>,
    /// Maximum entries to retain
    pub max_entries: usize,
}

/// Single episodic memory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicEntry {
    /// Entry ID
    pub id: String,
    /// Timestamp (Unix ms)
    pub timestamp: u64,
    /// Event type
    pub event_type: String,
    /// Summary of the interaction
    pub summary: String,
    /// Participants (agent DIDs)
    pub participants: Vec<String>,
    /// Importance score (0-1)
    pub importance: f32,
    /// Associated context
    pub context: HashMap<String, String>,
    /// Vector embedding for similarity search
    pub embedding: Option<Vec<f32>>,
}

impl EpisodicMemory {
    /// Create with capacity.
    pub fn with_capacity(max: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max),
            max_entries: max,
        }
    }
    
    /// Add entry, evicting oldest if at capacity.
    pub fn add(&mut self, entry: EpisodicEntry) {
        if self.max_entries > 0 && self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }
    
    /// Get recent entries.
    pub fn recent(&self, count: usize) -> &[EpisodicEntry] {
        let start = self.entries.len().saturating_sub(count);
        &self.entries[start..]
    }
    
    /// Filter by importance threshold.
    pub fn important(&self, threshold: f32) -> Vec<&EpisodicEntry> {
        self.entries.iter().filter(|e| e.importance >= threshold).collect()
    }
}

// ============================================================================
// SEMANTIC MEMORY - Knowledge Base
// ============================================================================

/// Semantic memory - stores general knowledge and facts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SemanticMemory {
    /// Knowledge facts
    pub facts: HashMap<String, SemanticFact>,
    /// Categories for organization
    pub categories: Vec<String>,
}

/// Single semantic fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFact {
    /// Fact ID
    pub id: String,
    /// Category
    pub category: String,
    /// Subject
    pub subject: String,
    /// Predicate
    pub predicate: String,
    /// Object/value
    pub object: String,
    /// Confidence score (0-1)
    pub confidence: f32,
    /// Source of this knowledge
    pub source: String,
    /// Last verified timestamp
    pub verified_at: Option<u64>,
    /// Vector embedding
    pub embedding: Option<Vec<f32>>,
}

impl SemanticMemory {
    /// Add a fact.
    pub fn add_fact(&mut self, fact: SemanticFact) {
        if !self.categories.contains(&fact.category) {
            self.categories.push(fact.category.clone());
        }
        self.facts.insert(fact.id.clone(), fact);
    }
    
    /// Get facts by category.
    pub fn by_category(&self, category: &str) -> Vec<&SemanticFact> {
        self.facts.values().filter(|f| f.category == category).collect()
    }
    
    /// Get facts by subject.
    pub fn by_subject(&self, subject: &str) -> Vec<&SemanticFact> {
        self.facts.values().filter(|f| f.subject == subject).collect()
    }
}

// ============================================================================
// SKILL MEMORY - Learned Capabilities
// ============================================================================

/// Skill memory - stores learned agent capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillMemory {
    /// Skill definitions
    pub skills: HashMap<String, LearnedSkill>,
}

/// A learned skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedSkill {
    /// Skill ID
    pub id: String,
    /// Skill name
    pub name: String,
    /// Description
    pub description: String,
    /// Proficiency level (0-1)
    pub proficiency: f32,
    /// Number of times used
    pub usage_count: u64,
    /// Last used timestamp
    pub last_used: Option<u64>,
    /// Required tools/APIs
    pub required_tools: Vec<String>,
    /// Example invocations
    pub examples: Vec<String>,
}

impl SkillMemory {
    /// Add or update a skill.
    pub fn learn(&mut self, skill: LearnedSkill) {
        self.skills.insert(skill.id.clone(), skill);
    }
    
    /// Get skill by ID.
    pub fn get(&self, id: &str) -> Option<&LearnedSkill> {
        self.skills.get(id)
    }
    
    /// List skills by proficiency.
    pub fn by_proficiency(&self, min: f32) -> Vec<&LearnedSkill> {
        self.skills.values().filter(|s| s.proficiency >= min).collect()
    }
    
    /// Record skill usage.
    pub fn record_usage(&mut self, id: &str) {
        if let Some(skill) = self.skills.get_mut(id) {
            skill.usage_count += 1;
            skill.last_used = Some(chrono::Utc::now().timestamp_millis() as u64);
        }
    }
}

// ============================================================================
// PREFERENCE MEMORY - User Settings
// ============================================================================

/// Preference memory - stores user-specific settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreferenceMemory {
    /// Preference items
    pub items: HashMap<String, PreferenceItem>,
}

/// A preference setting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceItem {
    /// Preference key
    pub key: String,
    /// Preference value
    pub value: serde_json::Value,
    /// Category
    pub category: String,
    /// Is this inferred or explicit?
    pub inferred: bool,
    /// Confidence if inferred
    pub confidence: Option<f32>,
    /// Last updated
    pub updated_at: u64,
}

impl PreferenceMemory {
    /// Set a preference.
    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value, category: impl Into<String>) {
        let key = key.into();
        self.items.insert(key.clone(), PreferenceItem {
            key,
            value,
            category: category.into(),
            inferred: false,
            confidence: None,
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
        });
    }
    
    /// Get a preference.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.items.get(key).map(|p| &p.value)
    }
    
    /// Get preferences by category.
    pub fn by_category(&self, category: &str) -> Vec<&PreferenceItem> {
        self.items.values().filter(|p| p.category == category).collect()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_layers_empty() {
        let layers = MemoryLayers::new();
        assert!(layers.is_empty());
        assert_eq!(layers.total_entries(), 0);
    }

    #[test]
    fn test_episodic_memory() {
        let mut mem = EpisodicMemory::with_capacity(10);
        
        mem.add(EpisodicEntry {
            id: "1".into(),
            timestamp: 1700000000000,
            event_type: "conversation".into(),
            summary: "Discussed weather".into(),
            participants: vec![],
            importance: 0.5,
            context: HashMap::new(),
            embedding: None,
        });
        
        assert_eq!(mem.entries.len(), 1);
        assert_eq!(mem.recent(5).len(), 1);
    }

    #[test]
    fn test_episodic_eviction() {
        let mut mem = EpisodicMemory::with_capacity(2);
        
        for i in 0..3 {
            mem.add(EpisodicEntry {
                id: i.to_string(),
                timestamp: 1700000000000 + i as u64,
                event_type: "test".into(),
                summary: format!("Entry {}", i),
                participants: vec![],
                importance: 0.5,
                context: HashMap::new(),
                embedding: None,
            });
        }
        
        assert_eq!(mem.entries.len(), 2);
        assert_eq!(mem.entries[0].id, "1"); // First entry evicted
    }

    #[test]
    fn test_semantic_memory() {
        let mut mem = SemanticMemory::default();
        
        mem.add_fact(SemanticFact {
            id: "fact-1".into(),
            category: "geography".into(),
            subject: "Paris".into(),
            predicate: "is_capital_of".into(),
            object: "France".into(),
            confidence: 1.0,
            source: "knowledge_base".into(),
            verified_at: None,
            embedding: None,
        });
        
        assert_eq!(mem.facts.len(), 1);
        assert!(mem.categories.contains(&"geography".to_string()));
        assert_eq!(mem.by_subject("Paris").len(), 1);
    }

    #[test]
    fn test_skill_memory() {
        let mut mem = SkillMemory::default();
        
        mem.learn(LearnedSkill {
            id: "search".into(),
            name: "Web Search".into(),
            description: "Search the web".into(),
            proficiency: 0.8,
            usage_count: 100,
            last_used: None,
            required_tools: vec!["search_api".into()],
            examples: vec![],
        });
        
        assert!(mem.get("search").is_some());
        assert_eq!(mem.by_proficiency(0.7).len(), 1);
        
        mem.record_usage("search");
        assert_eq!(mem.get("search").unwrap().usage_count, 101);
    }

    #[test]
    fn test_preference_memory() {
        let mut mem = PreferenceMemory::default();
        
        mem.set("language", serde_json::json!("en"), "localization");
        mem.set("theme", serde_json::json!("dark"), "ui");
        
        assert_eq!(mem.get("language"), Some(&serde_json::json!("en")));
        assert_eq!(mem.by_category("ui").len(), 1);
    }
}
