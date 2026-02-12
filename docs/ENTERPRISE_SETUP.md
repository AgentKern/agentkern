# Enterprise Edition Setup

AgentKern is an open-core platform. The core operating system is Apache 2.0 open source, while advanced enterprise features are available under a commercial license.

## Architecture

The Enterprise Edition (EE) is designed as an "overlay" that sits on top of the open source core.

- **OSS Repo**: `AgentKern/agentkern` (Public)
- **EE Repo**: `AgentKern/agentkern-ee` (Private)

## Setup Guide

If you have an Enterprise License, follow these steps to set up your environment:

1.  **Clone the OSS Repository**
    ```bash
    git clone https://github.com/AgentKern/agentkern.git
    cd agentkern
    ```

2.  **Sync the Enterprise Assets**
    Enterprise features are maintained in a separate private repository and synced into the local `ee/` overlay.
    ```bash
    ./scripts/pull-ee.sh
    ```
    For future updates, run the same command again.

    Optional overrides:
    ```bash
    EE_REPO_URL=git@github.com:AgentKern/agentkern-ee.git EE_BRANCH=main ./scripts/pull-ee.sh
    ```

    *Note: The `.gitignore` file is configured to ignore the `ee/` directory, so you won't accidentally commit enterprise code to the public repo.*

3.  **Activate Enterprise Features**
    Set your license key in the environment:
    ```bash
    export AGENTKERN_LICENSE_KEY="your-license-key"
    ```

4.  **Enable EE Workspace**
    Run the EE initializer to add enterprise crates to the OSS workspace `members`.
    ```bash
    ./ee/scripts/init-workspace.sh
    ```

5.  **Build**
    ```bash
    cargo build --workspace
    ```

## Daily Workflow (Clear Path)

Use this exact sequence when working with Enterprise code:

```bash
# 1) Sync latest private EE overlay
./scripts/pull-ee.sh

# 2) Ensure workspace includes EE crates
./ee/scripts/init-workspace.sh

# 3) Build and test
cargo build --workspace
cargo test --workspace
```

To switch back to OSS-only workspace members:

```bash
./ee/scripts/reset-workspace.sh
```

## Enterprise Modules

The `ee/` directory adds the following capabilities:

| Module | Description |
|--------|-------------|
| **Treasury** | 2-Phase Commit ledger, SWIFT/IBAN connectors, Budget enforcement |
| **Energy** | Carbon grid integration, WattTime/ElectricityMap connectors |
| **Audit** | Immutable audit logs, SIEM integration (Splunk, Datadog) |
| **Security** | HSM integration, KMS envelope encryption |
| **Compliance** | GDPR Article 15 export, HIPAA controls |

## Licensing

The content in `ee/` is strictly proprietary and confidential. It is NOT open source.
See `ee/LICENSE-ENTERPRISE.md` for terms.
