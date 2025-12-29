/**
 * {{PROJECT_NAME}} - AgentKern AI Agent
 * 
 * This agent is scaffolded with Zero-Trust defaults:
 * - mTLS authentication (via AgentKern Gate)
 * - OpenTelemetry tracing
 * - Capability-based permissions
 */

import { AgentKern, AgentCard, Capability } from '@agentkern/sdk';

// Agent capabilities (what this agent can do)
const capabilities: Capability[] = [
  {
    name: 'greet',
    description: 'Greets the user',
    inputSchema: {
      type: 'object',
      properties: {
        name: { type: 'string', description: 'Name to greet' },
      },
      required: ['name'],
    },
  },
  {
    name: 'process',
    description: 'Processes user input with AI',
    inputSchema: {
      type: 'object',
      properties: {
        prompt: { type: 'string', description: 'User prompt' },
      },
      required: ['prompt'],
    },
  },
];

// Define the Agent Card (identity + capabilities)
const agentCard: AgentCard = {
  name: '{{PROJECT_NAME}}',
  description: 'AI Agent built with AgentKern',
  version: '0.1.0',
  capabilities,
  // Zero-Trust: Required permissions
  permissions: {
    network: ['openai.com', 'anthropic.com'],
    resources: {
      maxTokens: 100_000,
      maxCost: 10.0, // $10 max spend
    },
  },
};

// Initialize AgentKern client
const client = new AgentKern({
  // Gate URL (for policy enforcement)
  gateUrl: process.env.AGENTKERN_GATE_URL || 'http://localhost:8080',
  // Agent credentials
  agentId: process.env.AGENT_ID,
  agentSecret: process.env.AGENT_SECRET,
});

// Register agent and start handling requests
async function main() {
  console.log(`🚀 Starting {{PROJECT_NAME}}...`);

  // Register with AgentKern mesh
  await client.register(agentCard);
  console.log('✅ Agent registered with AgentKern');

  // Start listening for tasks
  await client.listen(async (task) => {
    console.log(`📥 Received task: ${task.capability}`);

    switch (task.capability) {
      case 'greet':
        return { message: `Hello, ${task.input.name}!` };

      case 'process':
        // Example: Call an LLM (proxied through Gate for safety)
        const response = await client.llm.complete({
          model: 'gpt-4o-mini',
          prompt: task.input.prompt,
        });
        return { result: response };

      default:
        throw new Error(`Unknown capability: ${task.capability}`);
    }
  });
}

main().catch(console.error);
