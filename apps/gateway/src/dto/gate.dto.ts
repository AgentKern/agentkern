/**
 * AgentKern Gateway - Gate DTOs
 */

export class VerifyActionDto {
  agentId!: string;
  action!: string;
  context?: Record<string, unknown>;
}

export class RegisterPolicyDto {
  id!: string;
  name!: string;
  rules!: {
    id: string;
    condition: string;
    action: 'allow' | 'deny' | 'review' | 'audit';
    message?: string;
  }[];
  jurisdictions!: string[];
  priority!: number;
  enabled: boolean = true; // Default to enabled
}
