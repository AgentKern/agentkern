/**
 * AgentKern SDK
 * 
 * The Operating System for the Agentic Economy.
 * 
 * @packageDocumentation
 * 
 * @example
 * ```typescript
 * import { AgentKern } from '@agentkern/sdk';
 * 
 * // Initialize the client
 * const client = new AgentKern({
 *   apiKey: process.env.AGENTKERN_API_KEY,
 *   region: 'eu',
 * });
 * 
 * // Register an agent identity
 * const agent = await client.identity.register('my-sales-agent', ['read', 'write']);
 * 
 * // Verify action is safe before executing
 * const verification = await client.gate.verify(agent.id, 'send_email', {
 *   to: 'customer@example.com',
 *   subject: 'Your order confirmation',
 * });
 * 
 * if (verification.allowed) {
 *   // Execute the action
 *   await sendEmail(...);
 *   
 *   // Record the step in the intent path
 *   await client.synapse.recordStep(agent.id, 'send_email', 'success');
 * }
 * ```
 * 
 * ## The Four Pillars
 * 
 * AgentKern provides four core modules:
 * 
 * 1. **Identity** - Agent authentication & liability proofs
 * 2. **Gate** - Pre-execution verification & guardrails
 * 3. **Synapse** - Cross-agent state & memory with intent tracking
 * 4. **Arbiter** - Conflict resolution & coordination
 * 
 * Plus **Sovereign** for data residency compliance (GDPR, PIPL, etc.)
 */

export { AgentKern } from './client';
export * from './types';
export * from './ports';
