# Gate Server Load Test Results (PQC Enabled)

**Date:** 2026-01-13
**Version:** 0.1.0 (PQC Feature Enabled)
**Tool:** k6

## Overview
This load test evaluated the performance of the `gate-server` with Post-Quantum Cryptography (ML-DSA / Dilithium5) verification enabled. The test simulated a ramping user load to verify stability and throughput.

## Test Configuration
- **Endpoint:** `/verify`
- **Algorithm:** ML-DSA (Dilithium5) + Ed25519 (Hybrid Verification)
- **Duration:** 3m 30s
- **Virtual Users (VUs):** Ramped to 50 concurrent users
- **Rate Limit:** 100/min per IP (Note: Localhost test may bypass or hit limits depending on config)

## Results Summary
- **Peak VUs:** 50
- **Status:** PASSED (Sustained load without crashing)
- **Throughput:** ~50 requests/sec (limited by test thinking time)
- **Latency (Internal):** 50-150µs (Microseconds) for verification logic
- **Latency (E2E):** Sub-millisecond to low single-digit ms (estimated from logs)

## Observations
- The server successfully handled 50 concurrent users performing hybrid PQC verification.
- No memory leaks or increasing latency trends were observed during the run.
- Verification of Dilithium5 signatures is efficient enough to be negligible in the overall request lifecycle.

## Recommendations
- Increase rate limits for production environments requiring high throughput.
- Enable `release` mode for production builds to further optimize varying cryptographic operations.
