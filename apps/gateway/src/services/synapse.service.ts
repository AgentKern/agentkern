/**
 * AgentKern Gateway - Synapse Service
 * 
 * Business logic for agent memory and intent tracking.
 * Per ARCHITECTURE: Uses CRDTs for eventual consistency.
 */

import { Injectable, NotFoundException } from '@nestjs/common';
import { randomUUID } from 'crypto';

interface IntentStep {
  step: number;
  action: string;
  result?: string;
  timestamp: Date;
}

interface IntentPath {
  id: string;
  originalIntent: string;
  currentStep: number;
  totalSteps: number;
  history: IntentStep[];
  driftDetected: boolean;
  driftScore: number;
}

interface AgentState {
  agentId: string;
  intentPath?: IntentPath;
  state: Record<string, unknown>;
  updatedAt: Date;
  version: number;
}

@Injectable()
export class SynapseService {
  // In-memory state store (replace with Redis/graph DB in production)
  private readonly states = new Map<string, AgentState>();

  async getState(agentId: string): Promise<AgentState | null> {
    return this.states.get(agentId) ?? null;
  }

  async setState(agentId: string, state: Record<string, unknown>): Promise<AgentState> {
    const existing = this.states.get(agentId);
    
    const agentState: AgentState = {
      agentId,
      intentPath: existing?.intentPath,
      state: { ...existing?.state, ...state }, // CRDT-like merge
      updatedAt: new Date(),
      version: (existing?.version ?? 0) + 1,
    };

    this.states.set(agentId, agentState);
    return agentState;
  }

  async startIntent(agentId: string, intent: string, expectedSteps: number): Promise<IntentPath> {
    const intentPath: IntentPath = {
      id: randomUUID(),
      originalIntent: intent,
      currentStep: 0,
      totalSteps: expectedSteps,
      history: [],
      driftDetected: false,
      driftScore: 0,
    };

    const existing = this.states.get(agentId);
    const agentState: AgentState = {
      agentId,
      intentPath,
      state: existing?.state ?? {},
      updatedAt: new Date(),
      version: (existing?.version ?? 0) + 1,
    };

    this.states.set(agentId, agentState);
    return intentPath;
  }

  async recordStep(agentId: string, action: string, result?: string): Promise<IntentPath> {
    const state = this.states.get(agentId);
    if (!state?.intentPath) {
      throw new NotFoundException(`No active intent path for agent ${agentId}`);
    }

    const step: IntentStep = {
      step: state.intentPath.currentStep + 1,
      action,
      result,
      timestamp: new Date(),
    };

    state.intentPath.history.push(step);
    state.intentPath.currentStep = step.step;
    state.updatedAt = new Date();
    state.version++;

    // Simple drift detection (replace with ML-based detection in production)
    state.intentPath.driftScore = this.calculateDrift(state.intentPath);
    state.intentPath.driftDetected = state.intentPath.driftScore > 50;

    return state.intentPath;
  }

  async checkDrift(agentId: string): Promise<{ drifted: boolean; score: number }> {
    const state = this.states.get(agentId);
    if (!state?.intentPath) {
      return { drifted: false, score: 0 };
    }

    return {
      drifted: state.intentPath.driftDetected,
      score: state.intentPath.driftScore,
    };
  }

  private calculateDrift(intentPath: IntentPath): number {
    // Simple drift calculation based on step progression
    // In production, this would use semantic similarity with the original intent
    if (intentPath.history.length === 0) return 0;
    
    const progressRatio = intentPath.currentStep / intentPath.totalSteps;
    
    // If agent is taking more steps than expected, increase drift score
    if (progressRatio > 1) {
      return Math.min(100, (progressRatio - 1) * 100);
    }
    
    return 0;
  }
}
