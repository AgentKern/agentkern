# Treasury Reconciliation Runbook

**Version:** 1.0.0 | **Last Updated:** December 2025  
**Owner:** Platform Finance | **Severity:** P1 High

---

## Overview

Treasury reconciliation ensures financial integrity across agent transactions. The Treasury pillar uses 2-Phase Commit (2PC) for atomic transfers.

---

## Daily Reconciliation

### Automated Checks (runs at 00:00 UTC)

The system automatically reconciles:
- Agent balance totals vs transaction sum
- Carbon credits vs WattTime API billing
- Pending 2PC transactions > 5 minutes old

### Manual Reconciliation Trigger

```bash
# Run full reconciliation
cargo run --bin treasury-cli -- reconcile --scope full --date 2025-12-30

# Spot check specific agent
cargo run --bin treasury-cli -- reconcile --agent-id "agent_123" --last 7d
```

---

## 2PC Transaction Recovery

### Identify Stuck Transactions

```bash
# List pending 2PC transactions
cargo run --bin treasury-cli -- transactions list --state pending --age ">5m"
```

Expected output:
```
TXN_ID          STATE      AGE     PARTICIPANTS    AMOUNT
txn_abc123      PREPARED   12m     agent_1,agent_2 1000.00
txn_def456      PREPARED   8m      agent_3,agent_4 250.50
```

### Recovery Procedure

1. **Analyze transaction state:**
   ```bash
   cargo run --bin treasury-cli -- transactions inspect --txn-id "txn_abc123"
   ```

2. **Force resolution (if safe):**
   ```bash
   # Commit if all participants prepared
   cargo run --bin treasury-cli -- transactions force-commit --txn-id "txn_abc123" --approval "INC-12345"
   
   # Rollback if any participant failed
   cargo run --bin treasury-cli -- transactions force-rollback --txn-id "txn_abc123" --approval "INC-12345"
   ```

3. **Verify balance integrity:**
   ```bash
   cargo run --bin treasury-cli -- verify --agent-id "agent_1" --agent-id "agent_2"
   ```

---

## Discrepancy Investigation

### When Balances Don't Match

```bash
# Generate audit trail
cargo run --bin treasury-cli -- audit export \
  --agent-id "agent_123" \
  --format csv \
  --output /tmp/agent_123_audit.csv

# Compare with expected
cargo run --bin treasury-cli -- audit diff \
  --expected /tmp/expected.csv \
  --actual /tmp/agent_123_audit.csv
```

### Common Causes

| Symptom | Likely Cause | Resolution |
|---------|--------------|------------|
| Balance higher than transactions | Duplicate credit | Reverse transaction |
| Balance lower than transactions | Orphaned debit | Apply missing credit |
| Pending for hours | Coordinator failure | Force resolution |
| Carbon mismatch | WattTime sync lag | Wait for next sync cycle |

---

## Carbon Credit Reconciliation

```bash
# Sync with WattTime
cargo run --bin treasury-cli -- carbon sync --force

# Verify carbon ledger
cargo run --bin treasury-cli -- carbon verify --date 2025-12-30
```

---

## Reporting

Monthly finance reports:

```bash
# Generate monthly report
cargo run --bin treasury-cli -- report monthly \
  --month 2025-12 \
  --format pdf \
  --output /reports/treasury_2025_12.pdf
```

---

## Escalation

| Issue | Escalate To | SLA |
|-------|-------------|-----|
| Single agent discrepancy | On-call | 4 hours |
| Multi-agent discrepancy | Treasury lead | 2 hours |
| System-wide mismatch | CTO + CFO | Immediate |
| Regulatory inquiry | Legal + Compliance | Immediate |
