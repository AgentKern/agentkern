# AgentKern Execution Mandate

**Version**: 1.0  
**Date**: 2026-01-03  
**Status**: Active

> **Core Principles**: This document defines the non-negotiable requirements for all AgentKern code, architecture, and operations.

---

## Section 1: Autonomous Agent Spending Controls

**Requirement**: All autonomous agents must have spending limits and budget controls.

**Implementation**:
- Treasury pillar enforces per-agent budgets
- Real-time spending tracking and alerts
- Automatic termination on budget exhaustion
- Carbon footprint tracking and limits

**Rationale**: Uncontrolled spending by autonomous agents can lead to financial losses and resource abuse.

---

## Section 2: Production-Ready Code Standards

**Requirement**: Zero tolerance for unsafe code, mocks, TODOs, or placeholders in production code.

**Standards**:
- ✅ Type-safe error handling (no `any` types)
- ✅ Proper `Result` handling in Rust (no `unwrap()` in production)
- ✅ Comprehensive error messages
- ✅ No silent failures
- ✅ Fail-fast in production environments

**Rationale**: Production systems require reliability, safety, and maintainability.

---

## Section 3: Clean Architecture

**Requirement**: All code must follow Clean Architecture principles.

**Standards**:
- Proper abstraction layers
- Dependency inversion
- Type safety throughout
- Clear separation of concerns
- Testable components

**Rationale**: Maintainable, scalable, and testable codebase.

---

## Section 4: Hardware Root of Trust

**Requirement**: All agent actions must be traceable to hardware-backed keys.

**Implementation**:
- Intel TDX or AMD SEV-SNP for encrypted memory
- Hardware-backed attestation
- Cryptographic signatures for all actions
- TEE integration where available

**Rationale**: Logic alone is insufficient for trust in agentic systems.

---

## Section 5: Symbolic Policy Enforcement

**Requirement**: Security decisions must be deterministic and rule-based.

**Implementation**:
- Neuro-symbolic architecture (Rust + small neural models)
- Fail-closed policy engine
- Symbolic rule evaluation (<1ms)
- Neural intent analysis (<20ms)

**Rationale**: Stochastic models cannot be trusted for security-critical decisions.

---

## Section 6: Autonomous Agent Security

**Requirement**: Hardware-level kill switch for rogue agent termination.

**Implementation**:
- Emergency termination mechanism
- Policy violation detection
- Anomaly detection and response
- Manual override capabilities

**Rationale**: Autonomous agents must be controllable and terminable when they behave unexpectedly.

---

## Section 7: Carbon-Aware & Sustainable Computing

**Requirement**: All operations must track and limit carbon footprint.

**Implementation**:
- Carbon ledger per agent
- Carbon offset purchasing
- Energy-efficient algorithms
- Sustainable infrastructure choices

**Rationale**: Environmental responsibility is a core requirement, not optional.

---

## Section 8: Antifragile by Default

**Requirement**: Every failure must make the system stronger.

**Implementation**:
- Comprehensive error logging
- Failure analysis and learning
- Automatic recovery mechanisms
- Chaos engineering practices

**Rationale**: Systems that learn from failures are more resilient.

---

## Section 9: Data Sovereignty

**Requirement**: PII never leaves its legal jurisdiction without explicit consent.

**Implementation**:
- `DataRegion` enums (EU, US, CN, SA)
- Protocol-level enforcement
- Consent tracking and verification
- Jurisdiction-aware routing

**Rationale**: Legal compliance and user privacy are non-negotiable.

---

## Section 10: Sub-Millisecond Safety

**Requirement**: Safety checks must not add significant latency.

**Target**:
- Safety checks: <10ms
- Policy evaluation: <1ms
- Neural inference: <20ms

**Rationale**: Safety is irrelevant if it makes the system unusable.

---

## Compliance

All code, architecture decisions, and operational procedures must comply with this mandate. Violations are considered critical issues and must be addressed immediately.

**Related Documents**:
- `MANIFESTO.md` - High-level philosophy
- `STRATEGY.md` - Strategic direction
- `EPISTEMIC_HEALTH.md` - Current architectural status

---

**Last Updated**: 2026-01-03
