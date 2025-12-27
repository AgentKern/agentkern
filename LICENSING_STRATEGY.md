# AgentKern Licensing Strategy: The "Ghost Kernel" Model

**"Give away the Engine. Sell the Air Traffic Control."**

To optimize for **Massive Adoption** (FAANG-level standard) while guaranteeing **Unicorn Valuation**, we reject the "closed source" model. We also reject the controversial "BSL" (Business Source License) which scares away enterprise legal teams.

We adopt the **Temporal / Deno Open Core Model**.

---

## 1. The Open Zone (The Trojan Horse)
**License**: **MIT / Apache 2.0 (Permissive)**.
These components are 100% free, forever. This kills competitors by making the "Standard" free.
*   **`@agentkern/sdk`**: The developer interface.
*   **`@agentkern/identity`**: The passport protocol.
*   **`agentkern-core` (The Binary)**: The single-node Rust binary. Developers can run a "Local Cell" on their laptop for free.

**Why?**
*   **Developer Addiction**: If `npm install agentkern` works instantly, devs will use it.
*   **No Vendor Lock-in Fear**: Enterprises adopt it because "we can self-host if we have to."

## 2. The Commercial Zone (The Moat)
**License**: **Proprietary / Commercial SaaS**.
These features are *impossible* to self-host at scale without a dedicated team.
*   **AgentKern Cloud (The Multi-Cell Mesh)**:
    *   Coordinate 100+ Nodes.
    *   Global State Sync (Geo-Replication).
    *   Managed "Autonomic Mitosis" (Auto-scaling).
*   **AgentKern-Cockpit (The UI)**:
    *   "Mission Control" Dashboard.
    *   Audit Logs & Compliance Reporting (SOC2).
    *   Team Management / SSO.
*   **AgentKern-Trust (Reputation)**:
    *   Shared Global Reputation Score (The "Credit Bureau" of Agents).

---

## 3. The "Ghost Kernel" Strategy (Why this wins)
We give away the **"Ghost"** (The Standard) to haunt the ecosystem.
We sell the **"Mantle"** (The Safety Layer) to enterprises.

> *"You can run AgentKern on your laptop for free. But if you want to run 10,000 agents safely, you need AgentKern Cloud."*

**Comparison:**
| Feature | Local (Open Source) | Cloud (Enterprise) |
| :--- | :--- | :--- |
| **Agents** | Unlimited | Unlimited |
| **Nodes** | Single Node (Laptop) | **Multi-Node Cluster** |
| **State** | Filesystem (SQLite) | **Global Graph (Synapse)** |
| **Security** | Basic Guardrails | **SOC2 / HIPAA / Takaful** |
| **Price** | Free | **Usage-Based (x402)** |

This model built **MongoDB, HashiCorp, and Temporal**. It is the defined path to a $50B valuation.
