//! AgentKern-Synapse: CRDT (Conflict-Free Replicated Data Types)
//!
//! Per COMPETITIVE_LANDSCAPE.md: "Local-First (CRDTs)"
//!
//! This module provides conflict-free replicated data types for
//! local-first agent state that syncs without coordination.
//!
//! Features:
//! - G-Counter (grow-only counter)
//! - PN-Counter (increment/decrement counter)
//! - LWW-Register (last-writer-wins register)
//! - OR-Set (observed-remove set)
//! - LWW-Map (last-writer-wins map)
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_synapse::crdt::{GCounter, PNCounter, LwwRegister};
//!
//! let mut counter = GCounter::new("node-1");
//! counter.increment(5);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Node identifier for CRDT operations.
pub type NodeId = String;

/// Timestamp for ordering operations.
pub type Timestamp = u64;

/// Get current timestamp.
///
/// Returns 0 if system time is before UNIX epoch (should never happen in practice).
fn now() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

// ============================================
// G-Counter (Grow-Only Counter)
// ============================================

/// Grow-only counter CRDT.
///
/// Each node can only increment, never decrement.
/// The value is the sum of all node increments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCounter {
    /// Node ID of this replica
    node_id: NodeId,
    /// Per-node counts
    counts: HashMap<NodeId, u64>,
}

impl GCounter {
    /// Create a new G-Counter for a node.
    pub fn new(node_id: impl Into<NodeId>) -> Self {
        Self {
            node_id: node_id.into(),
            counts: HashMap::new(),
        }
    }

    /// Increment the counter.
    pub fn increment(&mut self, amount: u64) {
        *self.counts.entry(self.node_id.clone()).or_insert(0) += amount;
    }

    /// Get the total value.
    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Merge with another G-Counter.
    pub fn merge(&mut self, other: &GCounter) {
        for (node, count) in &other.counts {
            let entry = self.counts.entry(node.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }
    }
}

// ============================================
// PN-Counter (Positive-Negative Counter)
// ============================================

/// Positive-Negative counter CRDT.
///
/// Supports both increment and decrement.
/// Uses two G-Counters internally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PNCounter {
    /// Node ID
    node_id: NodeId,
    /// Positive counts
    positive: GCounter,
    /// Negative counts
    negative: GCounter,
}

impl PNCounter {
    /// Create a new PN-Counter.
    pub fn new(node_id: impl Into<NodeId>) -> Self {
        let id: NodeId = node_id.into();
        Self {
            node_id: id.clone(),
            positive: GCounter::new(id.clone()),
            negative: GCounter::new(id),
        }
    }

    /// Increment the counter.
    pub fn increment(&mut self, amount: u64) {
        self.positive.increment(amount);
    }

    /// Decrement the counter.
    pub fn decrement(&mut self, amount: u64) {
        self.negative.increment(amount);
    }

    /// Get the value (can be negative).
    pub fn value(&self) -> i64 {
        self.positive.value() as i64 - self.negative.value() as i64
    }

    /// Merge with another PN-Counter.
    pub fn merge(&mut self, other: &PNCounter) {
        self.positive.merge(&other.positive);
        self.negative.merge(&other.negative);
    }
}

// ============================================
// LWW-Register (Last-Writer-Wins Register)
// ============================================

/// Last-Writer-Wins Register CRDT.
///
/// Stores a single value; conflicts resolved by timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LwwRegister<T: Clone> {
    /// Current value
    value: Option<T>,
    /// Timestamp of last write
    timestamp: Timestamp,
    /// Node that wrote the value
    writer: NodeId,
}

impl<T: Clone> LwwRegister<T> {
    /// Create an empty register.
    pub fn new() -> Self {
        Self {
            value: None,
            timestamp: 0,
            writer: String::new(),
        }
    }

    /// Set the value.
    pub fn set(&mut self, value: T, node_id: impl Into<NodeId>) {
        let ts = now();
        if ts > self.timestamp {
            self.value = Some(value);
            self.timestamp = ts;
            self.writer = node_id.into();
        }
    }

    /// Get the current value.
    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Get the timestamp.
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    /// Merge with another register.
    pub fn merge(&mut self, other: &LwwRegister<T>) {
        if other.timestamp > self.timestamp {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.writer = other.writer.clone();
        }
    }
}

impl<T: Clone> Default for LwwRegister<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// OR-Set (Observed-Remove Set)
// ============================================

/// Element with unique tag for OR-Set.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct Tagged<T: Clone + Eq + std::hash::Hash> {
    value: T,
    tag: String,
}

/// Observed-Remove Set CRDT.
///
/// Add-wins semantics with unique tags per add.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrSet<T: Clone + Eq + std::hash::Hash> {
    /// Node ID
    node_id: NodeId,
    /// Counter for generating unique tags
    counter: u64,
    /// Elements with their tags
    elements: HashSet<Tagged<T>>,
    /// Tombstones (removed tags)
    tombstones: HashSet<String>,
}

impl<T: Clone + Eq + std::hash::Hash> OrSet<T> {
    /// Create a new OR-Set.
    pub fn new(node_id: impl Into<NodeId>) -> Self {
        Self {
            node_id: node_id.into(),
            counter: 0,
            elements: HashSet::new(),
            tombstones: HashSet::new(),
        }
    }

    /// Add an element.
    pub fn add(&mut self, value: T) {
        self.counter += 1;
        let tag = format!("{}:{}", self.node_id, self.counter);
        self.elements.insert(Tagged { value, tag });
    }

    /// Remove an element.
    pub fn remove(&mut self, value: &T) {
        let to_remove: Vec<_> = self
            .elements
            .iter()
            .filter(|e| e.value == *value)
            .cloned()
            .collect();

        for elem in to_remove {
            self.tombstones.insert(elem.tag.clone());
            self.elements.remove(&elem);
        }
    }

    /// Check if set contains a value.
    pub fn contains(&self, value: &T) -> bool {
        self.elements.iter().any(|e| e.value == *value)
    }

    /// Get all values.
    pub fn values(&self) -> Vec<&T> {
        self.elements.iter().map(|e| &e.value).collect()
    }

    /// Get the count of elements.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Merge with another OR-Set.
    pub fn merge(&mut self, other: &OrSet<T>) {
        // Add all elements from other (except tombstoned)
        for elem in &other.elements {
            if !self.tombstones.contains(&elem.tag) && !other.tombstones.contains(&elem.tag) {
                self.elements.insert(elem.clone());
            }
        }

        // Merge tombstones
        self.tombstones.extend(other.tombstones.iter().cloned());

        // Remove tombstoned elements
        self.elements.retain(|e| !self.tombstones.contains(&e.tag));
    }
}

// ============================================
// LWW-Map (Last-Writer-Wins Map)
// ============================================

/// Last-Writer-Wins Map CRDT.
///
/// Each key has an LWW-Register for its value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LwwMap<K: Clone + Eq + std::hash::Hash, V: Clone> {
    /// Node ID
    node_id: NodeId,
    /// Key-value registers
    entries: HashMap<K, LwwRegister<V>>,
    /// Tombstones for removed keys
    tombstones: HashMap<K, Timestamp>,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> LwwMap<K, V> {
    /// Create a new LWW-Map.
    pub fn new(node_id: impl Into<NodeId>) -> Self {
        Self {
            node_id: node_id.into(),
            entries: HashMap::new(),
            tombstones: HashMap::new(),
        }
    }

    /// Set a key-value pair.
    pub fn set(&mut self, key: K, value: V) {
        let ts = now();

        // Only set if newer than tombstone
        if let Some(&tomb_ts) = self.tombstones.get(&key) {
            if ts <= tomb_ts {
                return;
            }
        }

        let register = self.entries.entry(key).or_insert_with(LwwRegister::new);
        register.set(value, &self.node_id);
    }

    /// Get a value.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).and_then(|r| r.get())
    }

    /// Remove a key.
    pub fn remove(&mut self, key: &K) {
        let ts = now();
        self.tombstones.insert(key.clone(), ts);
        self.entries.remove(key);
    }

    /// Check if key exists.
    pub fn contains_key(&self, key: &K) -> bool {
        self.entries.contains_key(key) && !self.tombstones.contains_key(key)
    }

    /// Get all keys.
    pub fn keys(&self) -> Vec<&K> {
        self.entries.keys().collect()
    }

    /// Merge with another LWW-Map.
    pub fn merge(&mut self, other: &LwwMap<K, V>) {
        // Merge entries
        for (key, register) in &other.entries {
            let entry = self
                .entries
                .entry(key.clone())
                .or_insert_with(LwwRegister::new);
            entry.merge(register);
        }

        // Merge tombstones
        for (key, &ts) in &other.tombstones {
            let entry = self.tombstones.entry(key.clone()).or_insert(0);
            *entry = (*entry).max(ts);
        }

        // Remove entries older than tombstones
        self.entries.retain(|k, r| {
            if let Some(&tomb_ts) = self.tombstones.get(k) {
                r.timestamp() > tomb_ts
            } else {
                true
            }
        });
    }
}

// ============================================
// Agent State CRDT (Composite)
// ============================================

/// Agent state using CRDTs for local-first sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateCrdt {
    /// Agent ID
    pub agent_id: String,
    /// Action counter
    pub action_count: GCounter,
    /// Budget remaining (can go negative)
    pub budget: PNCounter,
    /// Current task
    pub current_task: LwwRegister<String>,
    /// Tags/labels
    pub tags: OrSet<String>,
    /// Metadata
    pub metadata: LwwMap<String, String>,
}

impl AgentStateCrdt {
    /// Create new agent state.
    pub fn new(agent_id: impl Into<String>, node_id: impl Into<NodeId>) -> Self {
        let id: String = agent_id.into();
        let node: NodeId = node_id.into();

        Self {
            agent_id: id,
            action_count: GCounter::new(node.clone()),
            budget: PNCounter::new(node.clone()),
            current_task: LwwRegister::new(),
            tags: OrSet::new(node.clone()),
            metadata: LwwMap::new(node),
        }
    }

    /// Merge with another agent state.
    pub fn merge(&mut self, other: &AgentStateCrdt) {
        if self.agent_id != other.agent_id {
            return; // Can't merge different agents
        }

        self.action_count.merge(&other.action_count);
        self.budget.merge(&other.budget);
        self.current_task.merge(&other.current_task);
        self.tags.merge(&other.tags);
        self.metadata.merge(&other.metadata);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcounter() {
        let mut c1 = GCounter::new("node-1");
        let mut c2 = GCounter::new("node-2");

        c1.increment(5);
        c2.increment(3);

        c1.merge(&c2);
        assert_eq!(c1.value(), 8);
    }

    #[test]
    fn test_pncounter() {
        let mut counter = PNCounter::new("node-1");

        counter.increment(10);
        counter.decrement(3);

        assert_eq!(counter.value(), 7);
    }

    #[test]
    fn test_lww_register() {
        let mut r1: LwwRegister<String> = LwwRegister::new();
        let mut r2: LwwRegister<String> = LwwRegister::new();

        r1.set("first".to_string(), "node-1");
        std::thread::sleep(std::time::Duration::from_micros(10));
        r2.set("second".to_string(), "node-2");

        r1.merge(&r2);
        assert_eq!(r1.get(), Some(&"second".to_string()));
    }

    #[test]
    fn test_or_set() {
        let mut s1 = OrSet::new("node-1");
        let mut s2 = OrSet::new("node-2");

        s1.add("apple");
        s1.add("banana");
        s2.add("banana");
        s2.add("cherry");

        s1.merge(&s2);
        assert!(s1.contains(&"apple"));
        assert!(s1.contains(&"banana"));
        assert!(s1.contains(&"cherry"));
    }

    #[test]
    fn test_or_set_remove() {
        let mut s1 = OrSet::new("node-1");
        s1.add("apple");
        s1.add("banana");

        s1.remove(&"apple");

        assert!(!s1.contains(&"apple"));
        assert!(s1.contains(&"banana"));
    }

    #[test]
    fn test_lww_map() {
        let mut m1: LwwMap<String, i32> = LwwMap::new("node-1");
        let mut m2: LwwMap<String, i32> = LwwMap::new("node-2");

        m1.set("a".to_string(), 1);
        m2.set("b".to_string(), 2);

        m1.merge(&m2);

        assert_eq!(m1.get(&"a".to_string()), Some(&1));
        assert_eq!(m1.get(&"b".to_string()), Some(&2));
    }

    #[test]
    fn test_agent_state_crdt() {
        let mut state1 = AgentStateCrdt::new("agent-42", "node-1");
        let mut state2 = AgentStateCrdt::new("agent-42", "node-2");

        state1.action_count.increment(5);
        state1.budget.increment(100);
        state1.tags.add("priority".to_string());

        state2.action_count.increment(3);
        state2.budget.decrement(20);
        state2.tags.add("verified".to_string());

        state1.merge(&state2);

        assert_eq!(state1.action_count.value(), 8);
        assert_eq!(state1.budget.value(), 80);
        assert!(state1.tags.contains(&"priority".to_string()));
        assert!(state1.tags.contains(&"verified".to_string()));
    }
}
