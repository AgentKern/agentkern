/**
 * AgentKern Gateway - Identity DTOs
 */

export class RegisterAgentDto {
  name!: string;
  capabilities?: string[];
}

export class SignActionDto {
  action!: string;
  payload!: Record<string, unknown>;
}

export class VerifyProofDto {
  proof!: {
    action: string;
    agentId: string;
    timestamp: string;
    signature: string;
    payloadHash: string;
  };
}
