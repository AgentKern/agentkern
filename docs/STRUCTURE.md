# Repository Structure (2026 Rust Unified)

This document defines the canonical directory structure of the AgentKern repository. We use a Rust workspace monorepo to ensure atomic consistency across the Six Pillars.

## 📁 Directory Layout

*   **`apps/`**: Entry points for execution.
    *   `server/`: The AgentKern Unified Server (Rust). Single binary for Identity, Gate, and Coordination.
    *   `playground/`: React/Vite frontend for real-time mesh visualization.
*   **`packages/`**: Core logic domains.
    *   `pillars/`: Implementation of the 6 core pillars (Gate, Synapse, Arbiter, Nexus, Treasury, Identity).
    *   `foundation/`: Lower-level shared infrastructure (Runtime, Edge, Crypto, Observability).
*   **`sdks/`**: Polyglot interface libraries.
    *   `typescript/`: N-API bound high-performance SDK for Node.js environments.
    *   `python/`: Python bindings for AI agent frameworks.
*   **`ee/`**: Enterprise Edition (Proprietary). Licensed modules for multi-tenancy and global mesh sync.
*   **`docs/`**: Pragmatic documentation (this directory).
*   **`scripts/`**: CI/CD, migration, and automation utilities.

## 🛠️ Workspaces

- **Rust**: Managed via root `Cargo.toml`. Members defined in `[workspace]`.
- **Node.js**: Managed via `pnpm` workspaces (`pnpm-workspace.yaml`).

## 🏷️ Naming Conventions

1. **Rust Crates**: Lowercase-kebab (e.g., `agentkern-gate`).
2. **Environment Variables**: UPPER_SNAKE_CASE (e.g., `JWT_SECRET`).
3. **Pillars**: Capitalized nouns (Gate, Synapse) in documentation; lowercase in code namespaces.
