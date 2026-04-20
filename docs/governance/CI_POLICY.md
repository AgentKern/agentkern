# CI Policy

This document defines which CI checks are blocking and which are advisory.

## Policy Goals

- Protect main branch quality and supply-chain integrity.
- Keep developer feedback fast and actionable.
- Allow non-deterministic external analysis tools to report without blocking merges.

## Blocking Checks

The following checks are merge-blocking and must pass:

- Rust formatting and linting
- Workspace tests and server tests
- OSS capability consistency checks
- Docker build verification
- Security audit and dependency audit
- License compliance checks
- Secret scanning

Rationale:

- These checks validate correctness, security posture, and enforceable project contracts.

## Advisory Checks

The following checks are advisory (non-blocking):

- Coverage generation (`cargo tarpaulin`)
- SonarCloud quality gate

Rationale:

- Coverage tooling can fail for environment-dependent reasons and should not block urgent fixes.
- SonarCloud depends on external service availability and token configuration.
- Both still produce useful quality signals and should be reviewed before release.

## Governance Rule

Any change that adds `continue-on-error: true` to a new step must include:

1. A justification in the workflow comment.
2. A corresponding update to this `CI_POLICY.md`.

This rule is automatically enforced by:

- `scripts/verify-ci-policy.sh`
- CI job: `OSS Capability Consistency`

Any proposal to make advisory checks blocking must include:

- evidence of reliability across recent runs
- impact assessment on contributor throughput
- rollback plan if false failures spike

## Release Readiness Rule

For tagged releases, advisory checks should be reviewed manually by release owners.
If advisory checks fail, release notes must include explicit risk acceptance or release should be delayed.
