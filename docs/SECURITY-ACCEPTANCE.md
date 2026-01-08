# Security Acceptance & Risk Register

This document formally records accepted security risks and their justifications per AgentKern's security review process.

## Audit Information

| Field | Value |
|-------|-------|
| **Last Audit Date** | 2026-01-08 |
| **Auditor** | Internal Security Review |
| **Scope** | Full codebase (Identity, Gate, Synapse, Arbiter, Nexus, Treasury, EE) |
| **Overall Rating** | **STRONG** ✅ |

---

## Vulnerability Summary

| Severity | Count | Status |
|----------|-------|--------|
| 🔴 Critical | 0 | N/A |
| 🟠 High | 0 | N/A |
| 🟡 Medium | 3 | ✅ All Fixed |
| 🟢 Low | 2 | ✅ L1 Fixed, L2 Accepted |

---

## Fixed Vulnerabilities

### M1: Ignored Security Advisories (RUSTSEC-2026-0001)
- **Risk:** rkyv crate had potential OOB/UB issues
- **Fix:** Updated `rust_decimal` with `default-features = false` across all crates
- **Commit:** `5019056`

### M2: CORS Wildcard Fallback
- **Risk:** CORS fell back to `*` if `CORS_ORIGINS` not set
- **Fix:** Required `CORS_ORIGINS` in production with fail-fast error
- **Commit:** `5019056`

### M3: OptionalAuthGuard Missing Verification Flag
- **Risk:** Downstream handlers couldn't distinguish verified vs unverified claims
- **Fix:** Added `verified: boolean` to `LiabilityProofPayload`
- **Commit:** `5019056`

### L1: Swagger Documentation in Production
- **Risk:** API documentation exposed in all environments
- **Fix:** Gated Swagger behind `NODE_ENV !== 'production'`
- **Commit:** `b601f91`

---

## Accepted Risks

### L2: Console Logging in Tests
- **Location:** Various `*.spec.ts` files
- **Risk:** Test output may contain sensitive data in CI logs
- **Severity:** Low
- **Justification:** 
  - Test environments use mock/fake data only
  - CI logs are not publicly accessible
  - Impact is limited to development visibility
- **Mitigation:**
  - Mock data has no production resemblance
  - CI retention policy limits log lifetime
- **Acceptance Date:** 2026-01-08
- **Review Date:** 2026-07-08

---

## Security Controls Verified

| Control | Status | Evidence |
|---------|--------|----------|
| Ed25519 JWT Signatures | ✅ | `liability-proof.guard.ts` |
| CSRF Double-Submit Cookie | ✅ | `csrf.middleware.ts` |
| Rate Limiting (Throttler) | ✅ | `app.module.ts` |
| Security Headers (Helmet) | ✅ | `main.ts` |
| Input Validation (class-validator) | ✅ | `*.dto.ts` |
| Parameterized Queries (SQLx) | ✅ | All repositories |
| Dependency Audit | ✅ | `cargo audit` = 0 vulnerabilities |

---

## Security Test Coverage

| Test Suite | Tests | Status |
|------------|-------|--------|
| Auth Bypass | 7 | ✅ Pass |
| CSRF | 7 | ✅ Pass |
| Injection | 12 | ✅ Pass |
| Rate Limiting | 3 | ✅ Pass |
| **Total** | **29** | ✅ All Pass |

---

## Next Review

- **Scheduled:** 2026-07-08
- **Trigger Events:** Major release, dependency update, incident
