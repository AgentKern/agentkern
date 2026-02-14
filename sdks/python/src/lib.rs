use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;

// Pillars
use agentkern_gate::engine::{GateEngine, VerificationRequestBuilder};
use agentkern_gate::prompt_guard::{PromptAction, PromptGuard, ThreatLevel};

// ============================================================================
// PROMPT GUARD
// ============================================================================

#[pyclass(name = "ThreatLevel", eq, ord)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PyThreatLevel {
    None = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl From<ThreatLevel> for PyThreatLevel {
    fn from(t: ThreatLevel) -> Self {
        match t {
            ThreatLevel::None => PyThreatLevel::None,
            ThreatLevel::Low => PyThreatLevel::Low,
            ThreatLevel::Medium => PyThreatLevel::Medium,
            ThreatLevel::High => PyThreatLevel::High,
            ThreatLevel::Critical => PyThreatLevel::Critical,
        }
    }
}

#[pyclass(name = "PromptAction", eq)]
#[derive(Clone, PartialEq, Eq)]
pub enum PyPromptAction {
    Allow,
    AllowWithLog,
    Review,
    Block,
    BlockAndAlert,
}

impl From<PromptAction> for PyPromptAction {
    fn from(a: PromptAction) -> Self {
        match a {
            PromptAction::Allow => PyPromptAction::Allow,
            PromptAction::AllowWithLog => PyPromptAction::AllowWithLog,
            PromptAction::Review => PyPromptAction::Review,
            PromptAction::Block => PyPromptAction::Block,
            PromptAction::BlockAndAlert => PyPromptAction::BlockAndAlert,
        }
    }
}

#[pyclass(name = "PromptAnalysis")]
pub struct PyPromptAnalysis {
    #[pyo3(get)]
    pub threat_level: PyThreatLevel,
    #[pyo3(get)]
    pub confidence: u8,
    #[pyo3(get)]
    pub action: PyPromptAction,
    #[pyo3(get)]
    pub latency_us: u64,
}

#[pyclass(name = "PromptGuard")]
pub struct PyPromptGuard {
    inner: PromptGuard,
}

#[pymethods]
impl PyPromptGuard {
    #[new]
    fn new() -> Self {
        Self {
            inner: PromptGuard::new(),
        }
    }

    fn analyze(&self, prompt: String) -> PyPromptAnalysis {
        let result = self.inner.analyze(&prompt);
        PyPromptAnalysis {
            threat_level: result.threat_level.into(),
            confidence: result.confidence,
            action: result.action.into(),
            latency_us: result.latency_us,
        }
    }

    fn is_safe(&self, prompt: String) -> bool {
        self.inner.is_safe(&prompt)
    }
}

// ============================================================================
// GATE ENGINE
// ============================================================================

#[pyclass(name = "GateEngine")]
pub struct PyGateEngine {
    inner: Arc<GateEngine>,
    rt: tokio::runtime::Runtime,
}

#[pymethods]
impl PyGateEngine {
    #[new]
    fn new() -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        Self {
            inner: Arc::new(GateEngine::new()),
            rt,
        }
    }

    /// Verify an action (blocking call for Python).
    fn verify(
        &self,
        agent_id: String,
        action: String,
        context: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<bool> {
        let mut builder = VerificationRequestBuilder::new(agent_id, action);

        if let Some(ctx) = context {
            for (key, value) in ctx.iter() {
                let key_str: String = key.extract().map_err(|e| {
                    pyo3::exceptions::PyTypeError::new_err(format!(
                        "Context keys must be strings: {}",
                        e
                    ))
                })?;

                let json_value = python_to_json_value(&value)?;
                builder = builder.context(key_str, json_value);
            }
        }

        let request = builder.build();

        // Block on async Rust
        let result = self.rt.block_on(self.inner.verify(request));

        Ok(result.allowed)
    }
}

// ============================================================================
// PYTHON → JSON CONVERSION HELPERS
// ============================================================================

/// Convert a Python object to a serde_json::Value.
///
/// Supports: str, int, float, bool, None, list, and dict (recursively).
fn python_to_json_value(obj: &Bound<'_, pyo3::PyAny>) -> PyResult<serde_json::Value> {
    use pyo3::types::{PyBool, PyFloat, PyInt, PyList, PyNone, PyString};

    // Order matters: PyBool must be checked before PyInt (bool is a subclass of int in Python)
    if obj.is_instance_of::<PyNone>() {
        Ok(serde_json::Value::Null)
    } else if obj.is_instance_of::<PyBool>() {
        let val: bool = obj.extract()?;
        Ok(serde_json::Value::Bool(val))
    } else if obj.is_instance_of::<PyInt>() {
        let val: i64 = obj.extract()?;
        Ok(serde_json::json!(val))
    } else if obj.is_instance_of::<PyFloat>() {
        let val: f64 = obj.extract()?;
        Ok(serde_json::json!(val))
    } else if obj.is_instance_of::<PyString>() {
        let val: String = obj.extract()?;
        Ok(serde_json::Value::String(val))
    } else if obj.is_instance_of::<PyList>() {
        let list = obj
            .downcast::<PyList>()
            .map_err(|e| pyo3::exceptions::PyTypeError::new_err(format!("Expected list: {}", e)))?;
        let items: Result<Vec<_>, _> = list
            .iter()
            .map(|item| python_to_json_value(&item))
            .collect();
        Ok(serde_json::Value::Array(items?))
    } else if obj.is_instance_of::<PyDict>() {
        let dict = obj
            .downcast::<PyDict>()
            .map_err(|e| pyo3::exceptions::PyTypeError::new_err(format!("Expected dict: {}", e)))?;
        let mut map = serde_json::Map::new();
        for (key, value) in dict.iter() {
            let key_str: String = key.extract()?;
            map.insert(key_str, python_to_json_value(&value)?);
        }
        Ok(serde_json::Value::Object(map))
    } else {
        // Fallback: convert to string representation
        let repr: String = obj.str()?.extract()?;
        Ok(serde_json::Value::String(repr))
    }
}

// ============================================================================
// MODULE DEFINITION
// ============================================================================

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPromptGuard>()?;
    m.add_class::<PyGateEngine>()?;
    m.add_class::<PyThreatLevel>()?;
    m.add_class::<PyPromptAction>()?;
    m.add_class::<PyPromptAnalysis>()?;
    Ok(())
}
