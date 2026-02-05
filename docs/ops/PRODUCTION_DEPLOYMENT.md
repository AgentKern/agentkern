# Production Deployment & Security

Comprehensive guide for deploying and hardening the AgentKern Rust Unified Server in production.

## 🚀 Deployment Checklist

### 1. Pre-Deployment
- [ ] All tests passing (`cargo test`)
- [ ] Zero compilation errors
- [ ] Security audit clean (`cargo audit`)
- [ ] Git tag created (e.g., `v0.2.0`)

### 2. Environment Configuration
Required variables for the Unified Server:
```bash
DATABASE_URL=postgresql://user:pass@host:5432/agentkern
JWT_SECRET=<generate-secure-32-char-secret>
ENCRYPTION_KEY=<generate-aes-256-key>
RUST_ENV=production
RUST_LOG=info
```

---

## 🛡️ Hardening & Security

### 1. Secret Management
- **JWT_SECRET**: Refuse to start if < 32 chars in production.
- **Database**: Use IAM roles or Secret Managers (Vault/AWS/GCP) instead of `.env` files.
- **Key Generation**: `openssl rand -base64 32`

### 2. Infrastructure Security
- **Non-Root**: Use the provided `agentkern` user in Docker/K8s.
- **mTLS**: Implement mTLS (Istio/Linkerd) for intra-pillar communication.
- **Network**: Isolate DB/Redis within private subnets.

### 3. Pillar Safety
- **Gate**: Ensure ML models are verified via SHA-256 checksums.
- **Arbiter**: Maintain access to the **Kill Switch** for incident response.
- **Identity**: Enforce short TTLs (e.g., 1 hour) for agent passports.

---

## 📦 Deployment Options

### Docker Compose (Sample)
```yaml
services:
  server:
    image: agentkern/server:latest
    environment:
      - RUST_ENV=production
      - JWT_SECRET=${JWT_SECRET}
      - DATABASE_URL=${DATABASE_URL}
    ports:
      - "3000:3000"
    restart: always
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
```

### Kubernetes
```bash
kubectl create secret generic agentkern-secrets \
  --from-literal=database-url=$DATABASE_URL \
  --from-literal=jwt-secret=$JWT_SECRET

kubectl apply -f k8s/deployment.yaml
```

---

## 🚨 Incident Response & Monitoring

- **Kill Switch**: Use `Arbiter` to freeze transactions if a compromise is detected.
- **Logging**: Mask PII in `tracing` logs before sending to aggregators.
- **Monitoring**: Set up Prometheus/Grafana using the native OTel exports.
- **Alerts**: Trigger on error rates > 0.1% or latency > 50ms.

---

*Support: security@agentkern.io | devops@agentkern.io*
