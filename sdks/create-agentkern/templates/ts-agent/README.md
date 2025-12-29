# {{PROJECT_NAME}}

An AI Agent built with [AgentKern](https://github.com/AgentKern/agentkern).

## Features

- 🔐 **Zero-Trust Security**: mTLS, capability-based permissions
- 📊 **Built-in Observability**: OpenTelemetry tracing
- 🛡️ **Policy Enforcement**: All actions gated through AgentKern Gate
- 💰 **Budget Control**: Token and cost limits enforced

## Getting Started

```bash
# Install dependencies
npm install

# Set environment variables
export AGENTKERN_GATE_URL=http://localhost:8080
export AGENT_ID=your-agent-id
export AGENT_SECRET=your-secret

# Run in development
npm run dev

# Build for production
npm run build
npm start
```

## Agent Capabilities

| Capability | Description |
|------------|-------------|
| `greet` | Greets the user |
| `process` | Processes user input with AI |

## Configuration

Edit the `agentCard` in `src/index.ts` to customize:
- Capabilities
- Network permissions
- Resource limits

## Learn More

- [AgentKern Documentation](https://github.com/AgentKern/agentkern/docs)
- [SDK Reference](https://github.com/AgentKern/agentkern/sdks/typescript)
