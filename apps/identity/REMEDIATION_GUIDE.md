# Vulnerability Remediation Guide: AgentProof

This document provides a technical guide for remediating the security findings identified in the 2025 Audit Report.

## 🔴 Critical Remediations

### 1. Hardcoded Secrets (K8s)
- **Problem**: Secrets like `POSTGRES_PASSWORD` were hardcoded in `k8s/deployment.yaml`.
- **Remediation**: Use **External Secrets Operator** or **HashiCorp Vault**.
- **Secure Alternative**:
```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: postgres-secret
spec:
  data:
    - secretKey: password
      remoteRef:
        key: path/to/vault-secret
```

### 2. Quantum-Vulnerabilty (Cryptography)
- **Problem**: `jose` defaults to ES256 and Ed25519, which are vulnerable to Shor's algorithm.
- **Remediation**: Migrate to **Hybrid Signatures** in `CryptoAgilityService`.
- **Implementation**:
```typescript
// Register a PQC provider (e.g., Dilithium)
cryptoService.registerProvider('pqc', new DilithiumProvider());
cryptoService.setHybridMode(true); // signs with ES256 AND Dilithium
```

## 🟡 High Priority Remediations

### 3. WebSocket Origin Misconfiguration
- **Problem**: `MeshGateway` uses `origin: '*'`.
- **Remediation**: Restrict to trust mesh authorized domains in `src/gateways/mesh.gateway.ts`.
```typescript
@WebSocketGateway({
  cors: {
    origin: process.env.AUTHORIZED_PEER_DOMAINS?.split(',') || 'trusted-neighbor.com',
  },
})
```

### 4. Lack of Body Size Limits
- **Problem**: `express-json` defaults allow large payloads, leading to DoS.
- **Remediation**: Configure limits in `main.ts`.
```typescript
app.use(express.json({ limit: '100kb' }));
```

## 🟢 Medium Priority Remediations

### 5. In-Memory State Risks
- **Problem**: `PolicyService` and `DnsResolutionService` use `Map()` for state.
- **Remediation**: Migrate to **PostgreSQL/Redis Persistence**.
```typescript
// Update TypeORM entity
@Entity()
export class TrustRecordEntity extends BaseEntity { ... }
```

## 🔍 Continuous Verification
- Run SAST: `semgrep scan --config=.semgrep.yml`
- Run DAST: `npm run test:e2e -- test/penetration-test.spec.ts`
- Run Load: `k6 run test/performance-test.k6.js`
