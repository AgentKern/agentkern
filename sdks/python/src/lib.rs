use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;

// Pillars
use agentkern_gate::engine::{GateEngine, VerificationRequestBuilder};
use agentkern_gate::prompt_guard::{PromptAction, PromptGuard, ThreatLevel};

// ============================================================================
// PROMPT GUARD
// ============================================================================

#[pyclass(name = "ThreatLevel")]
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

#[pyclass(name = "PromptAction")]
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
        let builder = VerificationRequestBuilder::new(agent_id, action);

        if let Some(_ctx) = context {
            // Basic context conversion (string keys/values for now)
            // In a real SDK we'd do full recursive dict -> serde_json conversion
        }

        let request = builder.build();

        // Block on async Rust
        let result = self.rt.block_on(self.inner.verify(request));

        Ok(result.allowed)
    }
}

// ============================================================================
// MODULE DEFINITION
// ============================================================================

#[pymodule]
fn agentkern(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPromptGuard>()?;
    m.add_class::<PyGateEngine>()?;
    m.add_class::<PyThreatLevel>()?;
    m.add_class::<PyPromptAction>()?;
    m.add_class::<PyPromptAnalysis>()?;
    Ok(())
}
