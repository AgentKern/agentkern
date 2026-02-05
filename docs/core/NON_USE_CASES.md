# When Not to Use This

AgentKern is a specialized toolset for high-reliability agentic systems. It introduces complexity (TEEs, CRDTs, 2P-Commit) that is unnecessary for many AI applications.

## Explicit Non-Use Cases

### 1. Simple Chat Interfaces
If your application provides a basic chat interface where the AI does not have autonomous access to external tools, databases, or financial assets, AgentKern is unnecessary overhead.

### 2. Low-Scale / Single-Agent Systems
AgentKern's coordination pillars (Arbiter, Synapse) are designed for multi-agent meshes. If you are running a single agent in a single region, standard local state management is preferred.

### 3. Non-Regulated / Low-Stakes Applications
If you do not require verifiable audit trails, hardware-level isolation, or cryptographic proof of authorship, the identity and gate pillars will add latency without providing relevant value.

### 4. High-Latency Tolerant Workflows
AgentKern is optimized for sub-millisecond safety enforcement. If your workflow involves long-running batch processes where a 2-second safety check is acceptable, a standard centralized Web2 security stack is easier to implement.

### 5. "No-Code" Environments
AgentKern is a developer kit requiring Rust or TypeScript expertise. It is not compatible with drag-and-drop agent builders or purely decorative AI widgets.
