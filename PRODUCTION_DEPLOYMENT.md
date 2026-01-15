# Production Deployment Checklist

Comprehensive checklist for deploying AgentKern v0.2.0 (Rust Unified Server) to production.

## Pre-Deployment

### 1. Code Quality ✅
- [x] All tests passing (Cargo workspace)
- [x] Zero compilation errors
- [x] Security scans clean (`cargo audit`)
- [x] CHANGELOG updated
- [x] Git tag created (v0.2.0)

### 2. Security Verification
- [ ] Run security scan workflow
- [ ] Verify no HIGH/CRITICAL vulnerabilities
- [ ] Check secret management (no hardcoded secrets)
- [ ] Validate TLS certificates
- [ ] Review CORS origins configuration

### 3. Environment Configuration

#### Unified Server Configuration
```bash
# Required
DATABASE_URL=postgresql://user:pass@host:5432/agentkern
JWT_SECRET=<generate-secure-secret>
ENCRYPTION_KEY=<generate-aes-256-key>

# Optional
RUST_LOG=info,agentkern_server=debug
PORT=3000

# AWS KMS (if using)
AWS_REGION=us-east-1
AWS_KMS_KEY_ID=<kms-key-id>
```

#### Enterprise Edition
```bash
# License
AGENTKERN_LICENSE_KEY=<enterprise-key>

# Identity Provider
AGENTKERN_IDENTITY_API_KEY=<entra-api-key>
AGENTKERN_IDENTITY_TENANT_ID=<tenant-id>
```

---

## Deployment Steps

### Option A: Docker Deployment

#### 1. Build Images

```bash
# Unified Server
docker build -t agentkern/server:0.2.0 -f apps/server/Dockerfile .

# Tag as latest
docker tag agentkern/server:0.2.0 agentkern/server:latest
```

#### 2. Push to Registry

```bash
docker push agentkern/server:0.2.0
docker push agentkern/server:latest
```

#### 3. Deploy with Docker Compose

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: agentkern
      POSTGRES_USER: agentkern_prod
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: always

  server:
    image: agentkern/server:0.2.0
    environment:
      DATABASE_URL: postgresql://agentkern_prod:${DB_PASSWORD}@postgres:5432/agentkern
      JWT_SECRET: ${JWT_SECRET}
      RUST_LOG: info
    ports:
      - "3000:3000"
    depends_on:
      - postgres
    restart: always
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  postgres_data:
```

#### 4. Run

```bash
docker-compose -f docker-compose.prod.yml up -d
```

---

### Option B: Kubernetes Deployment

#### 1. Create Secrets

```bash
kubectl create secret generic agentkern-secrets \
  --from-literal=database-url=$DATABASE_URL \
  --from-literal=jwt-secret=$JWT_SECRET \
  --from-literal=encryption-key=$ENCRYPTION_KEY
```

#### 2. Apply Manifests

```bash
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/postgres.yaml
kubectl apply -f k8s/server-deployment.yaml
kubectl apply -f k8s/server-service.yaml
kubectl apply -f k8s/ingress.yaml
```

---

### Option C: Cloud Platform

#### AWS (ECS/Fargate)
```bash
# Create task definition
aws ecs register-task-definition --cli-input-json file://ecs-task-def.json

# Create service
aws ecs create-service \
  --cluster agentkern-prod \
  --service-name server \
  --task-definition agentkern-server:1 \
  --desired-count 2 \
  --load-balancers ...
```

#### Google Cloud Run
```bash
gcloud run deploy server \
  --image gcr.io/agentkern/server:0.2.0 \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --set-env-vars DATABASE_URL=$DATABASE_URL
```

---

## Post-Deployment

### 1. Database Migrations

The server automatically handles migrations on startup if configured, or you can run:

```bash
# Run migrations manually via binary if needed
# (Assuming access to the binary in the container)
docker exec -it agentkern-server /usr/local/bin/agentkern-server migrate
```

### 2. Smoke Tests

```bash
# Health check
curl https://api.agentkern.io/health

# Create test agent
curl -X POST https://api.agentkern.io/api/v1/agents \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{"name": "smoke-test"}'
```

### 3. Monitoring Setup

- [ ] Configure application monitoring (DataDog, New Relic)
- [ ] Set up error tracking (Sentry)
- [ ] Enable log aggregation (CloudWatch, Stackdriver)
- [ ] Create alerts for error rates, latency
- [ ] Set up uptime monitoring (Pingdom, UptimeRobot)

### 4. Security Hardening

- [ ] Enable WAF rules
- [ ] Configure rate limiting
- [ ] Set up DDoS protection
- [ ] Enable audit logging
- [ ] Review IAM permissions

---

## Rollback Plan

### Quick Rollback

```bash
# Docker
docker-compose down
docker-compose -f docker-compose.prod.yml up -d agentkern/server:0.1.0

# Kubernetes
kubectl set image deployment/server server=agentkern/server:0.1.0

# Cloud Run
gcloud run deploy server --image gcr.io/agentkern/server:0.1.0
```

### Database Rollback

```bash
# Revert last migration (manual)
# requires sqlx-cli or manual SQL execution
```

---

## Success Criteria

- [ ] All health checks passing
- [ ] Response time < 5ms (p95) (Rust speed!)
- [ ] Error rate < 0.1%
- [ ] Zero security vulnerabilities
- [ ] Logs flowing to aggregation
- [ ] Metrics visible in dashboard
- [ ] Alerts configured and tested
- [ ] Documentation updated
- [ ] Team notified

---

## Support Contacts

- **On-Call**: See PagerDuty rotation
- **Security Issues**: security@agentkern.io
- **Infrastructure**: devops@agentkern.io
