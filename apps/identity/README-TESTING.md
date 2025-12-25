# AgentProof: Testing Strategy & Manual

This guide outlines how to run the various test suites implemented for AgentProof.

## 🧪 1. Unit & Integration Tests (Jest)
Focus on business logic coverage (>80%).
```bash
# Run all tests
npm test

# Run with coverage report
npm test -- --coverage

# Run specific service test
npm test src/services/crypto-agility.service.spec.ts
```

## 🛡️ 2. Security Penetration Tests (Supertest)
Automated checks for OWASP Top 10 and AI-specific vulnerabilities (Prompt Injection).
```bash
# Requires a running database/redis (or uses test-env)
npm run test:e2e -- test/penetration-test.spec.ts
```
**Coverage**: SQLi, XSS, Broken Auth, IDOR, Prompt Injection, Jailbreaking attempts.

## 📊 3. Performance & Load Testing (k6)
Validates system stability under concurrent load.
```bash
# Install k6 (if not present)
# Visit https://k6.io/docs/getting-started/installation/

# Run the load test
k6 run test/performance-test.k6.js
```
**Scenarios**: 50-100 concurrent users with ramping stages.

## 🔍 4. Static Analysis (SAST - Semgrep)
Scans source code for security anti-patterns.
```bash
# Install Semgrep
# pip install semgrep

# Run scan with local rules
semgrep scan --config=.semgrep.yml
```

## 🏗️ 5. CI/CD Integration
All tests are integrated into `.github/workflows/security.yml`. 
Failure in any critical security scan or penetration test will block the PR merge.

---
**Standard Coverage Target**: 85% Statements, 100% Security Endpoints.
