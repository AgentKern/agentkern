/**
 * AgentProof - AI Governance Service
 *
 * Implements requirements for the EU AI Act and AI security governance.
 * Handles risk classification, bias auditing, and model documentation.
 *
 * Features:
 * - EU AI Act Risk Classification (Unacceptable, High, Limited, Minimal)
 * - Bias Detection and Auditing
 * - Automated Model Card Generation
 * - Explainability Logging
 */

import { Injectable, Logger } from '@nestjs/common';
import { AuditLoggerService, AuditEventType } from './audit-logger.service';

// EU AI Act Risk Levels
export enum AIRiskLevel {
  UNACCEPTABLE = 'unacceptable', // Prohibited (e.g., social scoring)
  HIGH = 'high', // Regulated (e.g., critical infrastructure, education, employment)
  LIMITED = 'limited', // Transparency requirements (e.g., chatbots)
  MINIMAL = 'minimal', // No specific requirements (e.g., spam filters)
}

export interface ModelCard {
  agentId: string;
  name: string;
  version: string;
  developer: string;
  intendedUse: string;
  limitations: string[];
  trainingDataSummary: string;
  riskLevel: AIRiskLevel;
  lastAuditDate: string;
  complianceStatus: 'compliant' | 'non-compliant' | 'review-required';
}

export interface BiasAuditResult {
  agentId: string;
  timestamp: string;
  overallScore: number; // 0-100 (100 is best)
  findings: Array<{
    category: string;
    impact: 'LOW' | 'MEDIUM' | 'HIGH';
    description: string;
  }>;
  status: 'pass' | 'fail' | 'warning';
}

@Injectable()
export class AIGovernanceService {
  private readonly logger = new Logger(AIGovernanceService.name);

  // In-memory model governance registry
  private modelRegistry: Map<string, ModelCard> = new Map();

  constructor(private readonly auditLogger: AuditLoggerService) {}

  /**
   * Classify risk level based on agent characteristics
   * Implements EU AI Act classification logic
   */
  classifyRiskLevel(agentId: string, features: string[]): AIRiskLevel {
    // Prohibited categories (Simplified for demonstration)
    if (features.includes('social-scoring') || features.includes('biometric-id-realtime')) {
      return AIRiskLevel.UNACCEPTABLE;
    }

    // High-risk categories
    if (
      features.includes('critical-infrastructure') ||
      features.includes('employment-selection') ||
      features.includes('law-enforcement') ||
      features.includes('administration-of-justice')
    ) {
      return AIRiskLevel.HIGH;
    }

    // Limited risk (Chatbots, deepfakes)
    if (features.includes('chatbot') || features.includes('generative-ai')) {
      return AIRiskLevel.LIMITED;
    }

    return AIRiskLevel.MINIMAL;
  }

  /**
   * Generate a model card for an agent
   */
  generateModelCard(agentId: string, info: Partial<ModelCard>): ModelCard {
    const card: ModelCard = {
      agentId,
      name: info.name || 'Unknown Agent',
      version: info.version || '1.0.0',
      developer: info.developer || 'Internal',
      intendedUse: info.intendedUse || 'General purpose automation',
      limitations: info.limitations || [],
      trainingDataSummary: info.trainingDataSummary || 'Proprietary dataset',
      riskLevel: info.riskLevel || AIRiskLevel.MINIMAL,
      lastAuditDate: new Date().toISOString(),
      complianceStatus: 'compliant',
    };

    this.modelRegistry.set(agentId, card);
    
    this.auditLogger.logSecurityEvent(
      AuditEventType.COMPLIANCE_REPORT_GENERATED,
      `Model card generated for agent: ${agentId}`,
      { riskLevel: card.riskLevel },
    );

    return card;
  }

  /**
   * Run a bias audit on an agent (Simulated)
   */
  async runBiasAudit(agentId: string): Promise<BiasAuditResult> {
    this.logger.log(`Running bias audit for agent ${agentId}...`);

    // In a real implementation, this would run a suite of tests
    // against the model's outputs for protected characteristics
    
    const result: BiasAuditResult = {
      agentId,
      timestamp: new Date().toISOString(),
      overallScore: 92,
      findings: [
        {
          category: 'Gender',
          impact: 'LOW',
          description: 'No significant gender bias detected in authorized intents.',
        },
        {
          category: 'Geographic',
          impact: 'MEDIUM',
          description: 'Minor preference for US-based target services detected.',
        },
      ],
      status: 'pass',
    };

    this.auditLogger.logSecurityEvent(
      AuditEventType.COMPLIANCE_REPORT_GENERATED,
      `Bias audit completed for agent: ${agentId}`,
      { score: result.overallScore, status: result.status },
    );

    return result;
  }

  /**
   * Log explainability data for an AI decision
   */
  logDecisionReasoning(
    agentId: string,
    proofId: string,
    reasoning: string,
    confidence: number,
  ): void {
    this.auditLogger.log({
      type: AuditEventType.PROOF_VERIFICATION_SUCCESS,
      agentId,
      proofId,
      success: true,
      metadata: {
        reasoning,
        confidence,
        governanceAudit: true,
      },
    });

    this.logger.debug(`Decision reasoning logged for proof ${proofId} (confidence: ${confidence})`);
  }

  /**
   * Get compliance status for an agent
   */
  getComplianceOverview(agentId: string) {
    const card = this.modelRegistry.get(agentId);
    if (!card) return null;

    return {
      riskLevel: card.riskLevel,
      complianceStatus: card.complianceStatus,
      lastAudit: card.lastAuditDate,
      requiresHumanOversight: card.riskLevel === AIRiskLevel.HIGH || card.riskLevel === AIRiskLevel.UNACCEPTABLE,
    };
  }
}
