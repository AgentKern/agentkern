# Creating Policies

Define guardrails for your agents using VeriMantle's YAML-based Policy DSL.

---

## Policy Structure

```yaml
id: policy-unique-id
name: Human Readable Name
description: What this policy does
priority: 100
enabled: true
jurisdictions: [us, eu, global]

rules:
  - id: rule-1
    condition: "expression"
    action: allow | deny | review | audit
    message: "Optional message"
    risk_score: 50
```

---

## Quick Examples

### Spending Limits

```yaml
id: spending-limits
name: Spending Limits Policy
description: Prevent excessive spending by agents
priority: 100
enabled: true

rules:
  - id: block-large-transactions
    condition: "action == 'transfer_funds' && context.amount > 10000"
    action: deny
    message: "Transactions over $10,000 are not allowed"
    risk_score: 100

  - id: require-approval-medium
    condition: "action == 'transfer_funds' && context.amount > 1000"
    action: review
    message: "Transactions over $1,000 require approval"
    risk_score: 60

  - id: audit-all-transfers
    condition: "action == 'transfer_funds'"
    action: audit
```

### Data Access Controls

```yaml
id: data-access
name: Data Access Policy
description: Control access to sensitive data
priority: 90

rules:
  - id: block-pii-export
    condition: "action == 'export_data' && context.dataType == 'pii'"
    action: deny
    message: "PII export is prohibited"

  - id: limit-bulk-reads
    condition: "action == 'read_data' && context.recordCount > 1000"
    action: review
    message: "Bulk data reads require approval"
```

---

## Condition Expressions

### Available Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `action` | Action being verified | `"transfer_funds"` |
| `agent_id` | Agent requesting the action | `"agent-123"` |
| `context.*` | Context key-value pairs | `context.amount` |

### Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equals | `action == 'delete'` |
| `!=` | Not equals | `action != 'read'` |
| `>` | Greater than | `context.amount > 1000` |
| `<` | Less than | `context.priority < 5` |
| `>=` | Greater or equal | `context.count >= 10` |
| `<=` | Less or equal | `context.retries <= 3` |
| `&&` | Logical AND | `a == 1 && b == 2` |
| `\|\|` | Logical OR | `a == 1 \|\| b == 2` |

### Examples

```yaml
# Simple equality
condition: "action == 'delete_database'"

# Numeric comparison
condition: "context.amount > 10000"

# Combined conditions
condition: "action == 'transfer' && context.amount > 5000 && context.recipient != 'internal'"

# String matching
condition: "context.destination == 'external'"
```

---

## Policy Actions

### `allow`
Explicitly allows the action. Stops further rule evaluation.

### `deny`
Blocks the action. The agent receives the `message` in the response.

### `review`
Flags the action for human review. The action proceeds but is logged.

### `audit`
Logs the action for compliance. Does not affect execution.

---

## Jurisdictions

Policies can be scoped to specific regions:

```yaml
jurisdictions: [eu, us]  # Only applies in EU and US
```

| Code | Region | Regulation |
|------|--------|------------|
| `us` | United States | General |
| `eu` | European Union | GDPR |
| `cn` | China | PIPL |
| `sa` | Saudi Arabia | Vision 2030 |
| `in` | India | DPDP |
| `br` | Brazil | LGPD |
| `global` | Everywhere | - |

---

## Priority

Higher priority policies are evaluated first:

```yaml
priority: 100  # Evaluated before priority: 50
```

Use priority to ensure critical security policies run before general policies.

---

## Registering Policies

### Via SDK

```typescript
await client.gate.registerPolicy({
  id: 'my-policy',
  name: 'My Policy',
  priority: 100,
  enabled: true,
  rules: [
    {
      id: 'block-dangerous',
      condition: "action == 'dangerous_action'",
      action: 'deny',
      message: 'This action is blocked',
    },
  ],
});
```

### Via API

```bash
curl -X POST https://api.verimantle.io/v1/gate/policies \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d @policy.json
```

---

## Best Practices

1. **Start permissive, tighten gradually** — Begin with `audit` rules, then promote to `deny`

2. **Use meaningful IDs** — `spending-limit-10k` is better than `rule-1`

3. **Layer your policies** — High-priority security, medium business logic, low audit

4. **Test in staging** — Use `enabled: false` to deploy without activation

5. **Document with messages** — Every `deny` should explain why
