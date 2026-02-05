# Kill Switch Runbook

**Version:** 1.0.0 | **Last Updated:** December 2025  
**Owner:** Platform Operations | **Severity:** P0 Critical

---

## Overview

The Kill Switch is the emergency termination mechanism in the Arbiter pillar. It provides immediate agent termination for security incidents, runaway behavior, or compliance requirements.

---

## Activation Scenarios

| Scenario | Trigger | Expected Response Time |
|----------|---------|------------------------|
| Security breach | Manual or automated | < 100ms |
| Budget exhaustion | Automated | < 500ms |
| Malicious behavior detected | Gate escalation | < 200ms |
| Regulatory request | Manual | < 5min |

---

## Emergency Procedures

### 1. Global Kill Switch Activation

```bash
# Via CLI (requires admin credentials)
cargo run --bin arbiter-cli -- killswitch activate --scope global --reason "security incident"

# Via API
curl -X POST https://api.agentkern.io/v1/arbiter/killswitch \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{"scope": "global", "reason": "security incident", "operator": "oncall@agentkern.io"}'
```

**Expected outcome:** All agents terminate within 100ms.

### 2. Agent-Specific Termination

```bash
# Terminate specific agent
cargo run --bin arbiter-cli -- killswitch terminate --agent-id "agent_123" --reason "suspicious activity"
```

### 3. Swarm Termination

```bash
# Terminate all agents in a swarm
cargo run --bin arbiter-cli -- killswitch terminate-swarm --swarm-id "swarm_abc" --reason "budget exceeded"
```

---

## Verification Steps

After activation, verify:

1. **Check termination status:**
   ```bash
   cargo run --bin arbiter-cli -- killswitch status
   ```

2. **Review audit log:**
   ```bash
   cargo run --bin arbiter-cli -- audit query --event-type "agent.terminated" --last 1h
   ```

3. **Verify no active agents:**
   ```bash
   curl https://api.agentkern.io/v1/agents/active | jq '.count'
   # Expected: 0 (for global) or reduced count (for targeted)
   ```

---

## Rollback Procedure

After incident resolution:

1. **Identify root cause** — Do not restore until cause is understood
2. **Clear kill switch:**
   ```bash
   cargo run --bin arbiter-cli -- killswitch deactivate --approval-ticket "INC-12345"
   ```
3. **Restore agents gradually** — Use staged rollout
4. **Monitor closely** — Watch for recurrence

---

## Escalation Contacts

| Role | Contact | When to Escalate |
|------|---------|-----------------|
| On-call Engineer | PagerDuty | First responder |
| Security Team | security@agentkern.io | All security incidents |
| CTO | Direct escalation | Duration > 30min |

---

## Post-Incident

1. Create incident report within 24 hours
2. Update this runbook if gaps identified
3. Schedule blameless post-mortem within 72 hours
