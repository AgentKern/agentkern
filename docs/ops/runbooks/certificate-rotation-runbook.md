# Certificate Rotation Runbook

**Version:** 1.0.0 | **Last Updated:** December 2025  
**Owner:** Platform Security | **Severity:** P1 High

---

## Overview

This runbook covers mTLS certificate management for the AgentKern infrastructure. All inter-service communication uses mutual TLS for authentication.

---

## Certificate Types

| Type | Location | Rotation Frequency | Alert Threshold |
|------|----------|-------------------|-----------------|
| Root CA | HSM/Vault | 5 years | 90 days before expiry |
| Intermediate CA | Vault | 1 year | 30 days before expiry |
| Service certs | Kubernetes secrets | 90 days | 14 days before expiry |
| Agent certs | per-agent | 30 days | 7 days before expiry |

---

## Scheduled Rotation

### Service Certificate Rotation

```bash
# 1. Generate new certificate
./scripts/cert-rotate.sh generate --service identity --env production

# 2. Validate new certificate
./scripts/cert-rotate.sh validate --cert /tmp/identity-new.crt

# 3. Deploy with rolling update
kubectl set secret generic identity-tls \
  --from-file=tls.crt=/tmp/identity-new.crt \
  --from-file=tls.key=/tmp/identity-new.key \
  --dry-run=client -o yaml | kubectl apply -f -

# 4. Trigger rolling restart
kubectl rollout restart deployment/identity

# 5. Verify health
kubectl rollout status deployment/identity --timeout=300s
```

### Agent Certificate Rotation

Agent certificates are rotated automatically via the Identity pillar. Manual rotation:

```bash
# Revoke and reissue agent certificate
cargo run --bin identity-cli -- cert rotate \
  --agent-id "agent_123" \
  --reason "scheduled rotation"
```

---

## Emergency Revocation

For compromised certificates:

```bash
# 1. Revoke immediately
cargo run --bin identity-cli -- cert revoke \
  --serial "ABC123" \
  --reason "key compromise" \
  --effective-immediately

# 2. Propagate CRL
curl -X POST https://api.agentkern.io/v1/identity/crl/publish \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# 3. Verify propagation (all nodes should have updated CRL)
for node in $(kubectl get pods -l app=identity -o name); do
  kubectl exec $node -- cat /etc/pki/crl/current.crl | openssl crl -text | grep "Serial"
done
```

---

## Verification Commands

```bash
# Check certificate expiry
openssl x509 -in /etc/pki/service.crt -noout -enddate

# Verify certificate chain
openssl verify -CAfile /etc/pki/ca-chain.crt /etc/pki/service.crt

# Test mTLS connection
curl --cert client.crt --key client.key --cacert ca.crt https://identity.internal:8443/health
```

---

## Monitoring

Metrics to watch:
- `certificate_expiry_days` — Days until expiry
- `certificate_rotation_success` — Rotation success rate
- `crl_propagation_latency_ms` — CRL distribution time

Alerts:
- **Warning**: Certificate expires in < 14 days
- **Critical**: Certificate expires in < 7 days
- **Emergency**: Certificate expired

---

## Escalation

| Severity | Action |
|----------|--------|
| Warning | Schedule rotation in next maintenance window |
| Critical | Immediate rotation required, page on-call |
| Emergency | All hands, potential service disruption |
