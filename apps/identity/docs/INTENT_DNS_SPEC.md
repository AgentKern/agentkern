# Intent DNS Specification v1.0

## Overview

Intent DNS provides a global, cacheable trust resolution system for AI agents. 
Similar to DNS for domain names, Intent DNS resolves agent identities to trust information.

---

## Resolution Format

### Query Format
```
agentkern-identity://[agent-id].[principal-id].verify
```

### Example
```
agentkern-identity://cursor-agent.user-123.verify
```

### Response Format (TXT Record Style)
```
v=agentkern-identity1 trusted=true score=850 expires=2025-12-25T00:00:00Z revoked=false
```

---

## HTTP API (Alternative to DNS)

For environments where custom DNS protocols aren't practical, we provide an HTTP API.

### Resolve Trust
```
GET /api/v1/dns/resolve?agent={agentId}&principal={principalId}
```

Response:
```json
{
  "version": "1.0",
  "agentId": "cursor-agent",
  "principalId": "user-123",
  "trusted": true,
  "trustScore": 850,
  "expiresAt": "2025-12-25T00:00:00Z",
  "revoked": false,
  "cachedAt": "2025-12-24T12:00:00Z",
  "ttl": 300
}
```

### Batch Resolve
```
POST /api/v1/dns/resolve/batch
```

Request:
```json
{
  "queries": [
    {"agentId": "cursor-agent", "principalId": "user-123"},
    {"agentId": "copilot-agent", "principalId": "user-456"}
  ]
}
```

---

## Trust Record Structure

```typescript
interface TrustRecord {
  agentId: string;
  principalId: string;
  trustScore: number;        // 0-1000
  trusted: boolean;          // Score >= threshold (default 500)
  revoked: boolean;          // Manually revoked
  registeredAt: string;      // ISO timestamp
  lastVerifiedAt: string;    // Last successful verification
  expiresAt: string;         // Trust expiry
  verificationCount: number; // Total verifications
  failureCount: number;      // Failed verifications
  metadata?: {
    agentName?: string;
    agentVersion?: string;
    principalDevice?: string;
  };
}
```

---

## Caching Strategy

### TTL Rules
| Trust Score | TTL |
|-------------|-----|
| 800-1000 | 1 hour |
| 500-799 | 15 minutes |
| 0-499 | 5 minutes |
| Revoked | No cache |

### Cache Invalidation
- Trust revocation: immediate invalidation
- Score drop >100 points: immediate invalidation
- Principal key rotation: immediate invalidation

---

## Trust Score Calculation

```
score = base_score
  + (successful_verifications * 2)
  - (failed_verifications * 10)
  - (days_since_last_verification * 1)
  + (age_bonus)
  - (revocation_count * 50)
```

Where:
- `base_score` = 500 (new agents)
- `age_bonus` = min(days_active, 100)

---

## Security Considerations

1. **Query Rate Limiting**: 100 queries/minute per IP
2. **Response Signing**: Responses signed with server key
3. **Cache Poisoning Prevention**: HMAC validation
4. **Replay Protection**: Timestamp + nonce in queries
