/**
 * AgentKern Gateway - Synapse DTOs
 */

export class SetStateDto {
  state!: Record<string, unknown>;
}

export class StartIntentDto {
  intent!: string;
  expectedSteps!: number;
}

export class RecordStepDto {
  action!: string;
  result?: string;
}
