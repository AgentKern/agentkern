# Chaos Engineering Runbook

## Overview
This runbook documents procedures for controlled chaos testing in AgentKern staging environments.

## Prerequisites
- [ ] Staging environment isolated from production
- [ ] Monitoring dashboards accessible
- [ ] Database backups verified
- [ ] On-call team notified

## Chaos Profiles

| Profile | Failure Rate | Delay (ms) | Use Case |
|---------|-------------|------------|----------|
| Light | 5% | 50 | CI/CD pipeline validation |
| Moderate | 15% | 200 | Sprint chaos drill (weekly) |
| Heavy | 30% | 500 | Pre-release stress testing |

## Drill Procedure

### 1. Pre-Drill Checklist
```bash
# Verify staging is healthy
curl -sf http://staging:8080/health

# Ensure monitoring is up
# Open Grafana/Jaeger dashboards

# Notify team
# Post in #engineering Slack channel
```

### 2. Start Chaos
```bash
cd /path/to/agentkern
./scripts/chaos_drill.sh start moderate

# Or with validation tests
RUN_TESTS=true ./scripts/chaos_drill.sh start moderate
```

### 3. Monitor (15-30 minutes)
Watch for:
- Response time p95/p99 degradation
- Error rate increase (expected)
- Circuit breaker activations
- Database connection pool usage

### 4. Validation Checks
```bash
# Health endpoints should still respond
curl http://staging:8080/health
curl http://staging:8080/api/v1/arbiter/health

# Persistent locks should survive
curl -X POST http://staging:8080/api/v1/arbiter/locks \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "chaos-test", "resource": "test:resource"}'
```

### 5. Stop Chaos
```bash
./scripts/chaos_drill.sh stop

# Restart server to apply
systemctl restart agentkern-server
# Or: kubectl rollout restart deployment/agentkern-server
```

### 6. Post-Drill Review
Document:
- [ ] Error rates during drill
- [ ] Recovery time after stopping
- [ ] Any unexpected failures
- [ ] Circuit breaker behavior
- [ ] Latency distribution

## Emergency Procedures

### Kill Switch
If chaos causes unexpected issues:
```bash
# Immediate stop
./scripts/chaos_drill.sh stop

# Force restart
systemctl restart agentkern-server

# Or in Kubernetes
kubectl rollout restart deployment/agentkern-server
```

### Database Recovery
If distributed locks become inconsistent:
```sql
-- Clear test locks
DELETE FROM arbiter_locks WHERE resource LIKE 'test:%';
DELETE FROM arbiter_queue WHERE resource LIKE 'test:%';

-- Verify clean state
SELECT COUNT(*) FROM arbiter_locks;
```

## Metrics to Capture

| Metric | Normal | During Chaos | Recovery |
|--------|--------|--------------|----------|
| Request Latency (p50) | <10ms | <300ms | <10ms |
| Request Latency (p99) | <50ms | <1000ms | <50ms |
| Error Rate | <0.1% | 15-30% | <0.1% |
| Circuit Breaker State | CLOSED | OPEN/HALF | CLOSED |

## Drill Schedule
- **Weekly**: Light chaos during off-peak hours
- **Sprint Demo**: Moderate chaos before demo
- **Release Candidate**: Heavy chaos for 30 min
