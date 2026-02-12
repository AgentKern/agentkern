# OSS/EE Migration Matrix (`agentkern_ee_backup` → current repo)

This matrix is based on direct file inspection of `agentkern_ee_backup` and the live `ee/` + OSS tree.

## Decision Legend

- **OSS**: should live in the open-source workspace.
- **EE**: should live in enterprise modules under `ee/`.
- **Drop**: should not be migrated as-is.

## File-Level Mapping

| Backup Source | Decision | Target Location | Fact-Based Notes |
|---|---|---|---|
| `agentkern_ee_backup/identity/src/services/audit.rs` | OSS | `packages/pillars/identity/src/services/audit.rs` + `packages/pillars/identity/src/api/server.rs` | Identity migrations already define `audit_events`; API had audit wiring commented out. |
| `agentkern_ee_backup/parsers/Cargo.toml` | OSS | `packages/foundation/parsers/Cargo.toml` | Parser crate was previously part of OSS workspace and was deleted in working tree. |
| `agentkern_ee_backup/parsers/src/lib.rs` | OSS | `packages/foundation/parsers/src/lib.rs` | Needed to expose Copybook/IDOC/SWIFT MT parser surfaces. |
| `agentkern_ee_backup/parsers/src/copybook.rs` | OSS | `packages/foundation/parsers/src/copybook.rs` | No equivalent parser implementation currently exists in OSS or EE. |
| `agentkern_ee_backup/parsers/src/idoc.rs` | OSS | `packages/foundation/parsers/src/idoc.rs` | No equivalent parser implementation currently exists in OSS or EE. |
| `agentkern_ee_backup/parsers/src/swift_mt.rs` | OSS | `packages/foundation/parsers/src/swift_mt.rs` | Complements EE SWIFT MX parser by covering MT payloads. |
| `agentkern_ee_backup/parsers/fuzz/Cargo.toml` | OSS | `packages/foundation/parsers/fuzz/Cargo.toml` | Keep as parser hardening harness (not part of normal workspace build). |
| `agentkern_ee_backup/parsers/fuzz/fuzz_targets/*` | OSS | `packages/foundation/parsers/fuzz/fuzz_targets/*` | Keep as security testing assets. |
| `agentkern_ee_backup/arbiter/src/entity/shariah.rs` | Drop | N/A | Functional overlap with `packages/foundation/governance/src/industry/finance/shariah.rs`. |
| `agentkern_ee_backup/arbiter/src/entity/compliance.rs` | EE | `ee/compliance` | Broad policy/compliance taxonomy not currently represented in OSS arbiter entities. |
| `agentkern_ee_backup/gate/src/connectors/sap.rs` | EE | `ee/connectors/src/sap/*` | EE already has SAP connector module; migrate only missing capabilities, not duplicate stack. |
| `agentkern_ee_backup/gate/src/connectors/swift.rs` | EE | `ee/connectors/src/swift/*` | EE already has SWIFT connector module; migrate only missing capabilities. |
| `agentkern_ee_backup/synapse/src/encryption.rs` | Drop | N/A | File itself warns implementation is non-production; EE already has stronger envelope encryption in `ee/sovereign-memory/src/encryption.rs`. |
| `agentkern_ee_backup/synapse/src/secure_passport.rs` | EE | `ee/sovereign-memory` (+ selective OSS sync if needed) | Overlaps with current OSS Synapse passport model; treat as enterprise extension candidate, not direct copy. |
| `agentkern_ee_backup/treasury/src/watttime.rs` | EE | `ee/energy` or `ee/treasury` | Live WattTime client belongs with enterprise grid/carbon data features. |
| `agentkern_ee_backup/carbon.rs` | EE | `ee/energy` (+ optional treasury hooks) | Large carbon ledger implementation aligns with enterprise sustainability stack. |
| `agentkern_ee_backup/energy/tests/grid_integration.rs` | Drop | N/A | Targets old async API shape and legacy crate naming; needs rewrite against current `ee/energy`. |
| `agentkern_ee_backup/screening.rs` | EE | `ee/compliance` (or governance extension) | Domain screening logic belongs in enterprise compliance surfaces. |
| `agentkern_ee_backup/takaful.rs` | EE | `ee/treasury` | Shariah-specific pooled risk feature is domain/enterprise, not current OSS core path. |
| `agentkern_ee_backup/tee.rs` | EE | `ee/microvm` (or dedicated EE tee crate) | Hardware enclave logic aligns with enterprise confidential-compute surfaces. |
| `agentkern_ee_backup/LICENSE` | Drop | N/A | Backup license artifact is not needed in active monorepo tree. |

## Non-Source Artifacts to Exclude

Do not migrate compiled artifacts from backup trees (for example `target/`, fuzz build outputs, `.fingerprint/`, incremental objects). Keep only source and manifest files.

## Implementation Status (February 12, 2026)

- **Completed in OSS**
  - `packages/foundation/parsers` restored (crate, parser modules, and fuzz targets) and re-added to workspace members.
- **Completed in EE**
  - `ee/compliance` crate added with cultural checks, screening, liability models, and license gating.
  - `ee/connectors` now enforces compliance in SWIFT, SAP, and Mainframe transaction entry points.
  - `ee/energy/src/watttime.rs` added as the dynamic carbon-intensity client with HTTP mode + deterministic fallback mode.
  - `ee/treasury/src/takaful.rs` added and wired into `Treasury` APIs (pool creation, contribution, claim submit/approve/reject/pay) with Shariah validation.
  - `ee/energy/src/carbon.rs` added with regional intensity modeling, per-action footprinting, budget enforcement, metrics export, and solar-curve scheduling support.
  - `ee/microvm/src/tee.rs` added with TEE platform detection, attestation/verifier APIs, enclave wrapper, and sealed secret handling for simulated mode.
  - `ee/sovereign-memory/src/secure_passport.rs` added with zero-trust field-level encryption, access grants, and TEE-sealed field support using sovereign memory encryption primitives.
- **Still pending from this matrix**
  - None (all mapped items implemented).

## Final Verification Sweep (February 12, 2026)

- **Command run**
  - `cargo test -p agentkern-compliance-ee -p agentkern-connectors-ee -p agentkern-energy-ee -p agentkern-microvm-ee -p agentkern-treasury-ee -p agentkern-sovereign-memory-ee --offline`
- **Result**
  - All targeted migrated crates passed (`0` failures).
  - Environment-gated integration tests remained intentionally ignored (AWS KMS credentials and `AGENTKERN_LOCAL_KEK` requirements).

## PR-Ready Checklist

- [x] All mapped OSS/EE migration items implemented.
- [x] Shared EE compliance surface integrated across connectors and treasury flows.
- [x] Energy stack includes WattTime + Carbon ledger capabilities.
- [x] MicroVM stack includes TEE attestation/sealing interfaces.
- [x] Sovereign memory includes secure passport with grant + TEE-sealed field support.
- [x] Consolidated EE test sweep completed successfully.
