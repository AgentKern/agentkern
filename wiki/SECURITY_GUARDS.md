# Security Guards Reference

This document describes the security guards available in the AgentKern Identity service.

## LiabilityProofGuard

Validates `X-AgentKernIdentity` header for protected endpoints.

### Usage

```typescript
import { LiabilityProofGuard, Public } from '../guards/liability-proof.guard';

@Controller('agents')
@UseGuards(LiabilityProofGuard)
export class AgentsController {
  // All routes require liability proof

  @Get('public-info')
  @Public()  // Skip authentication
  getPublicInfo() {
    return { version: '1.0' };
  }

  @Post('create')
  // Requires valid X-AgentKernIdentity header
  createAgent(@Req() req) {
    const proof = req.liabilityProof;
    // proof.issuer, proof.subject, proof.action available
  }
}
```

### Validation Steps

1. **Header exists**: `X-AgentKernIdentity` must be present
2. **JWT format**: Must have 3 parts (header.claims.signature)
3. **Not expired**: `exp` claim must be in the future
4. **Required claims**: `iss` (issuer) and `sub` (subject) must exist
5. **Signature valid**: EdDSA signature verified against public key

### Error Responses

| Code | Message | Cause |
|------|---------|-------|
| 401 | Missing liability proof header | No `X-AgentKernIdentity` |
| 401 | Invalid liability proof | JWT parsing/validation failed |

---

## OptionalAuthGuard

Same as LiabilityProofGuard but doesn't throw on missing token.

```typescript
@UseGuards(OptionalAuthGuard)
@Get('profile')
getProfile(@Req() req) {
  if (req.liabilityProof) {
    // Authenticated request
    return { agent: req.liabilityProof.subject };
  } else {
    // Anonymous request
    return { agent: null };
  }
}
```

---

## EnterpriseLicenseGuard

Validates enterprise license for commercial features.

```typescript
@UseGuards(EnterpriseLicenseGuard)
@Get('enterprise/analytics')
getAnalytics() {
  // Only available with valid enterprise license
}
```

---

## Combining Guards

Guards can be combined with `AND` logic:

```typescript
@UseGuards(EnterpriseLicenseGuard, LiabilityProofGuard)
@Post('enterprise/transfer')
transfer() {
  // Requires BOTH license AND liability proof
}
```

## Creating Liability Proofs

Use the SDK to create valid proofs:

```typescript
import { Agent } from '@agentkern/sdk';

const agent = Agent.generate('my-agent');
const proof = agent.createProof('transfer:funds');

// Use in requests
fetch('/api/v1/agents/create', {
  headers: {
    'X-AgentKernIdentity': proof.jwt
  }
});
```
