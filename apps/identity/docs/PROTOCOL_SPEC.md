# AgentProof Protocol Specification v1.0

## Overview

AgentProof provides **Liability Proofs** – cryptographic attestations that prove:
1. A specific human authorized a specific action
2. The authorization was made via a hardware-bound credential (Passkey)
3. The authorizer accepts liability for the agent's action

---

## Liability Proof Format

A Liability Proof is a compact, self-verifying token embedded in HTTP headers.

### Header Format
```
X-AgentProof: v1.<payload>.<signature>
```

### Payload Structure (Base64URL-encoded JSON)

```json
{
  "version": "1.0",
  "proof_id": "uuid-v4",
  "issued_at": "2025-12-24T10:00:00Z",
  "expires_at": "2025-12-24T10:05:00Z",
  
  "principal": {
    "id": "user-uuid",
    "credential_id": "passkey-credential-id",
    "device_attestation": "platform-attestation-hash"
  },
  
  "agent": {
    "id": "agent-uuid",
    "name": "cursor-ai-agent",
    "version": "1.0.0"
  },
  
  "intent": {
    "action": "transfer",
    "target": {
      "service": "api.bank.com",
      "endpoint": "/v1/transfers",
      "method": "POST"
    },
    "parameters": {
      "amount": 1000,
      "currency": "USD",
      "to_account": "****1234"
    }
  },
  
  "constraints": {
    "max_amount": 5000,
    "allowed_recipients": ["****1234", "****5678"],
    "geo_fence": ["US", "CA"],
    "valid_hours": {"start": 9, "end": 17},
    "require_confirmation_above": 1000
  },
  
  "liability": {
    "accepted_by": "principal",
    "terms_version": "1.0",
    "dispute_window_hours": 72
  }
}
```

### Signature

The signature is created using ES256 (ECDSA with P-256 curve) via WebAuthn:

1. Serialize payload to canonical JSON
2. Hash with SHA-256
3. Sign with user's Passkey private key
4. Encode signature as Base64URL

---

## Verification Flow

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Agent     │────▶│  Target Service  │────▶│  Verify Proof   │
│  (Request)  │     │  (Receives X-    │     │  1. Check expiry│
│             │     │   AgentProof)    │     │  2. Verify sig  │
└─────────────┘     └──────────────────┘     │  3. Check intent│
                                              │  4. Log audit   │
                                              └─────────────────┘
```

### Verification Steps

1. **Parse Header**: Extract version, payload, signature
2. **Decode Payload**: Base64URL decode, parse JSON
3. **Check Expiry**: `expires_at` must be in future
4. **Retrieve Public Key**: From AgentProof Trust Registry or cached
5. **Verify Signature**: ES256 verification using public key
6. **Validate Intent**: Check action matches request being made
7. **Check Constraints**: Ensure request falls within authorized bounds
8. **Log Audit**: Record verification result for compliance

---

## Liability Acceptance

The `liability` block is critical for legal clarity:

```json
"liability": {
  "accepted_by": "principal",    // Who accepts liability
  "terms_version": "1.0",        // Which terms they agreed to
  "dispute_window_hours": 72     // Time to dispute unauthorized action
}
```

### Liability Rules

| Scenario | Liable Party |
|----------|--------------|
| Valid proof, agent acts within constraints | Principal (user) |
| Valid proof, agent exceeds constraints | Agent operator |
| Invalid/forged proof | Agent operator |
| Expired proof | Agent operator |
| Revoked credential | Agent operator |

---

## Trust Resolution (Intent DNS)

For real-time trust lookups without full verification:

```
agentproof://agent-123.principal-456.verify
```

Returns DNS TXT record:
```
"v=agentproof1 trusted=true score=850 expires=2025-12-25"
```

---

## HTTP Integration

### Request with AgentProof
```http
POST /v1/transfers HTTP/1.1
Host: api.bank.com
Content-Type: application/json
X-AgentProof: v1.eyJ2ZXJzaW9uIjoiMS4wIi....<signature>

{"amount": 1000, "to_account": "****1234"}
```

### Response Codes

| Code | Meaning |
|------|---------|
| 200 | Request processed, proof valid |
| 401 | Missing or invalid X-AgentProof header |
| 403 | Proof valid but action not authorized by constraints |
| 410 | Proof expired |

---

## Security Considerations

1. **Passkey Binding**: Proofs can only be created with hardware-bound keys
2. **Short Expiry**: Default 5 minutes to limit replay window
3. **Single Use**: Proof IDs should be tracked to prevent replay
4. **Constraint Enforcement**: Target services MUST check constraints
5. **Audit Trail**: All verifications must be logged

---

## Versioning

Protocol versions follow semver. The `version` field in payload ensures forward compatibility.

| Version | Status |
|---------|--------|
| 1.0 | Current specification |

---

*AgentProof Protocol Specification v1.0*
*December 2025*
