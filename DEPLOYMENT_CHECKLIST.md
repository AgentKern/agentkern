# AgentKern Staging Deployment Checklist

## Pre-Deployment

### Build Verification
- [ ] `cargo build --release -p agentkern-gate`
- [ ] `cargo build --release -p agentkern-synapse`
- [ ] `cargo build --release -p agentkern-treasury`
- [ ] `npm run build` in `apps/identity`

### Test Suite
- [ ] `cargo test --workspace`
- [ ] `npm test` in `apps/identity`
- [ ] Integration tests: `cargo test --test context_guard_integration`

### Security
- [ ] Run `cargo audit` - no high/critical vulnerabilities
- [ ] Verify secrets not in codebase: `git secrets --scan`
- [ ] SBOM generated: `npm sbom --sbom-format cyclonedx`

---

## Environment Configuration

### Required Environment Variables
```bash
# Identity Service
DATABASE_URL=postgresql://...
REDIS_URL=redis://...

# WattTime (optional - enables dynamic carbon)
WATTTIME_USERNAME=your_username
WATTTIME_PASSWORD=your_password

# Observability
OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4318
```

### Feature Flags
```bash
# Gate features
--features "wasm,neural,pqc,otel"
```

---

## Deployment Steps

1. **Database Migrations**
   ```bash
   cd apps/identity && npm run typeorm migration:run
   ```

2. **Deploy Gate Server**
   ```bash
   docker build -t agentkern-gate ./packages/pillars/gate
   docker push registry.example.com/agentkern-gate:latest
   kubectl apply -f k8s/gate-deployment.yaml
   ```

3. **Deploy Identity Service**
   ```bash
   docker build -t agentkern-identity ./apps/identity
   kubectl apply -f k8s/identity-deployment.yaml
   ```

4. **Verify Health**
   ```bash
   curl https://gate.staging.example.com/health
   curl https://identity.staging.example.com/health
   ```

---

## Post-Deployment

### Smoke Tests
- [ ] `/health` returns 200 on all services
- [ ] Prometheus metrics endpoint accessible
- [ ] Grafana dashboards loading

### Monitoring
- [ ] Check error rate in first 15 minutes
- [ ] Verify OTEL traces appearing in Tempo
- [ ] Carbon metrics in `gate_context_scans_total`

---

## Rollback Procedure
```bash
kubectl rollout undo deployment/gate-server
kubectl rollout undo deployment/identity-service
```
