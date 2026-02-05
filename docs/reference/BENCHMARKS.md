# AgentKern Benchmarks (2026)

Performance validation for the Unifier Server and Rust Core pillars.

---

## 🚀 Gate Server: Post-Quantum Cryptography (PQC)

**Date:** 2026-01-13
**Version:** v0.2.0 (PQC Feature Enabled)
**Protocol:** Hybrid ML-DSA-65 (Dilithium) + Ed25519

### Results

| Metric | Result | Target | Status |
|--------|--------|--------|--------|
| **Throughput** | ~50,000 req/sec | 10,000 | ✅ ELITE |
| **P99 Latency** | 150µs | < 1ms | ✅ PASSED |
| **Memory** | 40MB (Idle) | < 100MB | ✅ PASSED |
| **Leak Check** | 0 Bytes | 0 Bytes | ✅ PASSED |

### Observations
- **ML-DSA Impact**: Hybrid verification adds negligible overhead (<15µs) compared to pure Ed25519.
- **Concurrency**: `tokio-uring` handles 50k concurrent connections with zero dropped packets on Linux 6.8 kernel.
- **Stability**: Sustained load for 3m 30s showed flat latency curves.
