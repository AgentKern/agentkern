# AgentKern Licensing Strategy

## Open Core Model

AgentKern uses an **Open Core** licensing model:

| Tier | License | Features |
|------|---------|----------|
| **Community** | Apache 2.0 | Core pillars, basic protocols |
| **Enterprise** | Commercial | Advanced features, support |

---

## Open Source (Apache 2.0)

### Packages

| Package | Description |
|---------|-------------|
| `packages/gate/` | Policy enforcement, neural verification |
| `packages/synapse/` | Memory, embeddings, CRDT |
| `packages/arbiter/` | Locks, coordination, killswitch |
| `packages/nexus/` | Protocol translation, routing |
| `packages/sdk/` | TypeScript SDK |
| `apps/identity/` | OAuth, JWT, agent credentials |
| `apps/gateway/` | REST/gRPC API |

### Features Included

- ✅ A2A Protocol support
- ✅ MCP Protocol support  
- ✅ AgentKern native protocol
- ✅ Agent Cards & discovery
- ✅ In-memory agent registry
- ✅ Skill-based task routing
- ✅ Rule-based explainability
- ✅ Basic SHAP explanations
- ✅ Circuit breaker
- ✅ Antifragile recovery
- ✅ Carbon scheduling
- ✅ Kill switch
- ✅ Raft consensus locks
- ✅ HIPAA, PCI, Shariah (Islamic Finance) compliance
- ✅ Quantum-safe crypto (hybrid)
- ✅ TEE attestation (simulated)

#### Phase 2 Features (NEW)

- ✅ **Legacy Bridge SDK** - Connector framework
- ✅ **SQL Connector** - Generic SQL/JDBC bridge (FREE)
- ✅ **Protocol Parsers** - SWIFT MT, SAP IDOC, COBOL
- ✅ **Memory Passport** - Portable agent state
- ✅ **GDPR Export** - Article 20 compliance
- ✅ **Escalation Triggers** - Trust threshold monitoring
- ✅ **Webhook Notifications** - Generic webhook support
- ✅ **Approval Workflow** - Human-in-the-loop

---

## Enterprise License (Commercial)

### Packages (`ee/` directory)

| Package | Description |
|---------|-------------|
| `ee/treasury/` | Cross-agent payments, insurance |
| `ee/multitenancy/` | Tenant isolation, quotas |
| `ee/billing/` | Stripe metering, usage billing |
| `ee/sovereign-mesh/` | Cross-datacenter replication |
| `ee/audit-export/` | Compliance export (SOC2, ISO) |
| `ee/cockpit/` | Admin dashboard |
| `ee/cloud/` | Managed cloud deployment |

### Enterprise Features

- 🔒 Distributed agent registry (PostgreSQL, Redis)
- 🔒 ML-based task routing optimization
- 🔒 Kubernetes service discovery
- 🔒 Insurance policy integration (Munich Re API)
- 🔒 Legal entity framework (Wyoming DAO)
- 🔒 LIME advanced explanations
- 🔒 GPU-accelerated SHAP
- 🔒 Cross-fleet failure correlation
- 🔒 Predictive failure detection
- 🔒 Multi-tenant isolation
- 🔒 Stripe billing integration
- 🔒 Audit export (PDF, CSV)
- 🔒 24/7 support SLA

#### Phase 2 Enterprise Features (NEW)

- 🔒 **SAP Connector** - RFC, BAPI, OData, Event Mesh
- 🔒 **SWIFT Connector** - MX (ISO 20022), GPI, Sanctions
- 🔒 **Mainframe Connector** - CICS, IMS, MQ
- 🔒 **Oracle Connector** - OCI, E-Business Suite
- 🔒 **Cross-cloud Migration** - AWS, GCP, Azure adapters
- 🔒 **Memory Encryption** - KMS integration, key rotation
- 🔒 **Memory Sharding** - Distributed memory storage
- 🔒 **Slack/Teams/PagerDuty** - Native integrations
- 🔒 **Multi-approver Workflows** - Complex approval chains
- 🔒 **Real-time Grid API** - Carbon Intersect integration

---

## License Enforcement

Enterprise features are gated via:

```rust
// Check for valid license
if std::env::var("AGENTKERN_LICENSE_KEY").is_err() {
    return Err(TreasuryError::LicenseRequired)
}
```

---

## Pricing (Proposed)

| Tier | Price | Target |
|------|-------|--------|
| Community | Free | Startups, OSS projects |
| Pro | $999/mo | Growing companies |
| Enterprise | Custom | Large enterprises |

---

## Contributing

Community contributions to `packages/` are welcome under Apache 2.0.
Enterprise features in `ee/` require a Contributor License Agreement (CLA).
