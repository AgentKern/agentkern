# Security Tooling Setup & Configuration

This document outlines the security tooling integrated into the AgentKern workspace for continuous security monitoring and protection.

## 1. Static Application Security Testing (SAST)

### Semgrep
- **Configuration**: `.github/workflows/security.yml`
- **Rule Packs**: OWASP Top 10, Secrets, JWT, SQL Injection, NextJS/Node Security.
- **Local Scan**:
  ```bash
  semgrep scan --config auto
  ```

### ESLint Security
- **Plugins**: `eslint-plugin-security`, `typescript-eslint`.
- **Configuration**: `apps/identity/eslint.config.mjs` (includes security rules).

## 2. Dependency Scanning

### Rust (Cargo Audit)
- **Tool**: `cargo-audit`
- **Usage**:
  ```bash
  cargo audit
  ```
- **CI Integration**: Runs on every push to verify crates.io advisories.

### JavaScript (npm audit)
- **Usage**:
  ```bash
  npm audit --audit-level=high
  ```

### Container Scanning (Trivy)
- **Configuration**: Integrated in `.github/workflows/security.yml`.
- **Coverage**: Scans the compiled Docker images for OS and library vulnerabilities.

## 3. Secret Scanning

### Gitleaks
- **Scope**: Entire repository history.
- **Usage**:
  ```bash
  gitleaks detect --source . -v
  ```
- **Pre-commit**: Blocks commits containing suspected secrets.

### Detect-Secrets
- **Usage**:
  ```bash
  detect-secrets scan
  ```

## 4. Secret Setup (Pre-commit Hooks)

To install the security barriers locally:
```bash
pip install pre-commit
pre-commit install
```

## 5. DAST & Fuzzing

### Security E2E
- **File**: `apps/identity/test/security-comprehensive.e2e-spec.ts`
- **Targets**: Authentication, Business Logic, AI Vectors.

### k6 Security Load Testing
- **File**: `tests/performance/security-load-test.js`
- **Targets**: Rate limiting (DoS mitigation), Auth stress.
