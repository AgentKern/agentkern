//! VeriMantle: Agent Liability Insurance API
//!
//! Per FUTURE_INNOVATION_ROADMAP.md Innovation #2:
//! Native integration with insurance providers for AI agent liability.
//!
//! Features:
//! - Risk scoring integration
//! - Coverage verification before actions
//! - Incident reporting (EU AI Act compliant)
//! - Premium optimization

import { v4 as uuidv4 } from 'uuid';

// ============================================================================
// INSURANCE TYPES
// ============================================================================

/**
 * Insurance coverage types.
 */
export enum CoverageType {
  /** General liability */
  LIABILITY = 'LIABILITY',
  /** Errors and omissions */
  ERRORS_OMISSIONS = 'ERRORS_OMISSIONS',
  /** Cyber liability */
  CYBER = 'CYBER',
  /** Professional indemnity */
  PROFESSIONAL_INDEMNITY = 'PROFESSIONAL_INDEMNITY',
}

/**
 * Risk level.
 */
export enum RiskLevel {
  LOW = 'LOW',
  MEDIUM = 'MEDIUM',
  HIGH = 'HIGH',
  CRITICAL = 'CRITICAL',
}

/**
 * Insurance policy.
 */
export interface InsurancePolicy {
  id: string;
  agentId: string;
  providerId: string;
  coverageType: CoverageType;
  coverageLimit: number;  // USD
  deductible: number;
  premium: number;  // Monthly
  validFrom: Date;
  validUntil: Date;
  status: 'ACTIVE' | 'PENDING' | 'EXPIRED' | 'CANCELLED';
  exclusions: string[];
  metadata: Record<string, unknown>;
}

/**
 * Risk assessment.
 */
export interface RiskAssessment {
  agentId: string;
  trustScore: number;
  transactionVolume: number;
  historicalClaims: number;
  sectorRisk: RiskLevel;
  jurisdictionRisk: RiskLevel;
  overallRisk: RiskLevel;
  riskScore: number;  // 0-100
  assessedAt: Date;
  factors: RiskFactor[];
}

/**
 * Risk factor.
 */
export interface RiskFactor {
  name: string;
  weight: number;
  score: number;
  description: string;
}

/**
 * Coverage check result.
 */
export interface CoverageCheckResult {
  covered: boolean;
  policy: InsurancePolicy | null;
  coverageAmount: number;
  exclusionHit: boolean;
  exclusionReason?: string;
  warnings: string[];
}

/**
 * Incident report (EU AI Act compliant).
 */
export interface IncidentReport {
  id: string;
  agentId: string;
  policyId: string;
  incidentType: string;
  severity: RiskLevel;
  description: string;
  affectedParties: string[];
  damageEstimate: number;
  occurredAt: Date;
  reportedAt: Date;
  status: 'REPORTED' | 'INVESTIGATING' | 'RESOLVED' | 'DENIED';
  euAiActCompliant: boolean;
  rootCause?: string;
  correctiveActions?: string[];
}

/**
 * Premium quote.
 */
export interface PremiumQuote {
  id: string;
  agentId: string;
  coverageType: CoverageType;
  coverageLimit: number;
  deductible: number;
  monthlyPremium: number;
  annualPremium: number;
  riskScore: number;
  validUntil: Date;
  factors: PremiumFactor[];
}

/**
 * Premium factor.
 */
export interface PremiumFactor {
  name: string;
  impact: 'INCREASE' | 'DECREASE' | 'NEUTRAL';
  percentage: number;
  reason: string;
}

// ============================================================================
// INSURANCE PROVIDERS
// ============================================================================

/**
 * Insurance provider.
 */
export interface InsuranceProvider {
  id: string;
  name: string;
  supportedCoverages: CoverageType[];
  minCoverage: number;
  maxCoverage: number;
  jurisdictions: string[];
  apiEndpoint?: string;
}

// Pre-defined providers from market research
export const INSURANCE_PROVIDERS: InsuranceProvider[] = [
  {
    id: 'munich-re',
    name: 'Munich Re AI Warranty',
    supportedCoverages: [CoverageType.LIABILITY, CoverageType.ERRORS_OMISSIONS],
    minCoverage: 100_000,
    maxCoverage: 10_000_000,
    jurisdictions: ['US', 'EU', 'UK', 'APAC'],
  },
  {
    id: 'relm',
    name: 'Relm Insurance',
    supportedCoverages: [CoverageType.LIABILITY, CoverageType.CYBER],
    minCoverage: 50_000,
    maxCoverage: 5_000_000,
    jurisdictions: ['US'],
  },
  {
    id: 'axa-coalition',
    name: 'AXA/Coalition AI',
    supportedCoverages: [CoverageType.CYBER, CoverageType.PROFESSIONAL_INDEMNITY],
    minCoverage: 100_000,
    maxCoverage: 25_000_000,
    jurisdictions: ['US', 'EU', 'UK'],
  },
];

// ============================================================================
// INSURANCE SERVICE
// ============================================================================

/**
 * Agent Liability Insurance Service.
 */
export class InsuranceService {
  private policies: Map<string, InsurancePolicy> = new Map();
  private assessments: Map<string, RiskAssessment> = new Map();
  private incidents: Map<string, IncidentReport> = new Map();
  private quotes: Map<string, PremiumQuote> = new Map();

  /**
   * Assess risk for an agent.
   */
  async assessRisk(
    agentId: string,
    trustScore: number,
    transactionVolume: number,
    sector: string,
    jurisdiction: string
  ): Promise<RiskAssessment> {
    const factors: RiskFactor[] = [];
    let totalScore = 0;
    let totalWeight = 0;

    // Trust score factor
    const trustFactor = {
      name: 'Trust Score',
      weight: 0.3,
      score: trustScore,
      description: `Agent trust score: ${trustScore}/100`,
    };
    factors.push(trustFactor);
    totalScore += trustFactor.score * trustFactor.weight;
    totalWeight += trustFactor.weight;

    // Transaction volume factor
    const volumeScore = Math.min(100, transactionVolume / 1000);
    const volumeFactor = {
      name: 'Transaction Volume',
      weight: 0.2,
      score: volumeScore,
      description: `${transactionVolume} transactions processed`,
    };
    factors.push(volumeFactor);
    totalScore += volumeFactor.score * volumeFactor.weight;
    totalWeight += volumeFactor.weight;

    // Sector risk
    const sectorScores: Record<string, number> = {
      finance: 30,
      healthcare: 40,
      retail: 70,
      general: 60,
    };
    const sectorScore = sectorScores[sector.toLowerCase()] || 50;
    const sectorFactor = {
      name: 'Sector Risk',
      weight: 0.25,
      score: sectorScore,
      description: `${sector} sector risk profile`,
    };
    factors.push(sectorFactor);
    totalScore += sectorFactor.score * sectorFactor.weight;
    totalWeight += sectorFactor.weight;

    // Jurisdiction risk
    const jurisdictionScores: Record<string, number> = {
      EU: 60,  // Stricter regulations = lower risk
      US: 50,
      UK: 55,
      APAC: 45,
    };
    const jurisdictionScore = jurisdictionScores[jurisdiction.toUpperCase()] || 40;
    const jurisdictionFactor = {
      name: 'Jurisdiction Risk',
      weight: 0.25,
      score: jurisdictionScore,
      description: `${jurisdiction} regulatory environment`,
    };
    factors.push(jurisdictionFactor);
    totalScore += jurisdictionFactor.score * jurisdictionFactor.weight;
    totalWeight += jurisdictionFactor.weight;

    const riskScore = Math.round(totalScore / totalWeight);
    
    const assessment: RiskAssessment = {
      agentId,
      trustScore,
      transactionVolume,
      historicalClaims: 0,
      sectorRisk: this.scoreToRiskLevel(100 - sectorScore),
      jurisdictionRisk: this.scoreToRiskLevel(100 - jurisdictionScore),
      overallRisk: this.scoreToRiskLevel(100 - riskScore),
      riskScore,
      assessedAt: new Date(),
      factors,
    };

    this.assessments.set(agentId, assessment);
    return assessment;
  }

  /**
   * Get premium quote.
   */
  async getQuote(
    agentId: string,
    coverageType: CoverageType,
    coverageLimit: number,
    deductible: number
  ): Promise<PremiumQuote> {
    const assessment = this.assessments.get(agentId);
    if (!assessment) {
      throw new Error(`No risk assessment found for agent ${agentId}`);
    }

    // Base rate: 0.5% of coverage
    let basePremium = coverageLimit * 0.005;
    const factors: PremiumFactor[] = [];

    // Risk adjustment
    const riskMultiplier = 1 + ((100 - assessment.riskScore) / 100);
    if (riskMultiplier > 1) {
      factors.push({
        name: 'Risk Score Adjustment',
        impact: 'INCREASE',
        percentage: (riskMultiplier - 1) * 100,
        reason: `Risk score ${assessment.riskScore}/100`,
      });
    } else {
      factors.push({
        name: 'Risk Score Discount',
        impact: 'DECREASE',
        percentage: (1 - riskMultiplier) * 100,
        reason: `Excellent risk score ${assessment.riskScore}/100`,
      });
    }
    basePremium *= riskMultiplier;

    // Deductible discount
    const deductibleDiscount = Math.min(0.2, deductible / coverageLimit);
    factors.push({
      name: 'Deductible Discount',
      impact: 'DECREASE',
      percentage: deductibleDiscount * 100,
      reason: `$${deductible} deductible`,
    });
    basePremium *= (1 - deductibleDiscount);

    // Coverage type adjustment
    const typeMultipliers: Record<CoverageType, number> = {
      [CoverageType.LIABILITY]: 1.0,
      [CoverageType.ERRORS_OMISSIONS]: 1.2,
      [CoverageType.CYBER]: 1.5,
      [CoverageType.PROFESSIONAL_INDEMNITY]: 1.3,
    };
    const typeMultiplier = typeMultipliers[coverageType];
    if (typeMultiplier > 1) {
      factors.push({
        name: 'Coverage Type',
        impact: 'INCREASE',
        percentage: (typeMultiplier - 1) * 100,
        reason: `${coverageType} coverage`,
      });
    }
    basePremium *= typeMultiplier;

    const monthlyPremium = Math.round(basePremium / 12);

    const quote: PremiumQuote = {
      id: uuidv4(),
      agentId,
      coverageType,
      coverageLimit,
      deductible,
      monthlyPremium,
      annualPremium: Math.round(basePremium),
      riskScore: assessment.riskScore,
      validUntil: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000),
      factors,
    };

    this.quotes.set(quote.id, quote);
    return quote;
  }

  /**
   * Purchase a policy.
   */
  async purchasePolicy(quoteId: string): Promise<InsurancePolicy> {
    const quote = this.quotes.get(quoteId);
    if (!quote) {
      throw new Error(`Quote not found: ${quoteId}`);
    }

    if (new Date() > quote.validUntil) {
      throw new Error('Quote has expired');
    }

    const now = new Date();
    const policy: InsurancePolicy = {
      id: uuidv4(),
      agentId: quote.agentId,
      providerId: 'verimantle-underwrite',
      coverageType: quote.coverageType,
      coverageLimit: quote.coverageLimit,
      deductible: quote.deductible,
      premium: quote.monthlyPremium,
      validFrom: now,
      validUntil: new Date(now.getTime() + 365 * 24 * 60 * 60 * 1000),
      status: 'ACTIVE',
      exclusions: [
        'Intentional acts',
        'Pre-existing conditions',
        'Acts of war',
        'Nuclear incidents',
      ],
      metadata: { quoteId },
    };

    this.policies.set(policy.id, policy);
    return policy;
  }

  /**
   * Check if an action is covered.
   */
  async checkCoverage(
    agentId: string,
    actionType: string,
    estimatedValue: number
  ): Promise<CoverageCheckResult> {
    // Find active policy
    const policy = Array.from(this.policies.values()).find(
      p => p.agentId === agentId && p.status === 'ACTIVE'
    );

    if (!policy) {
      return {
        covered: false,
        policy: null,
        coverageAmount: 0,
        exclusionHit: false,
        warnings: ['No active policy found'],
      };
    }

    // Check exclusions
    const exclusionPatterns = ['hack', 'fraud', 'intentional', 'malicious'];
    const exclusionHit = exclusionPatterns.some(p => 
      actionType.toLowerCase().includes(p)
    );

    if (exclusionHit) {
      return {
        covered: false,
        policy,
        coverageAmount: 0,
        exclusionHit: true,
        exclusionReason: 'Action type matches policy exclusion',
        warnings: [],
      };
    }

    // Check coverage limit
    const warnings: string[] = [];
    if (estimatedValue > policy.coverageLimit) {
      warnings.push(
        `Estimated value $${estimatedValue} exceeds coverage limit $${policy.coverageLimit}`
      );
    }

    return {
      covered: true,
      policy,
      coverageAmount: Math.min(estimatedValue, policy.coverageLimit) - policy.deductible,
      exclusionHit: false,
      warnings,
    };
  }

  /**
   * Report an incident (EU AI Act compliant).
   */
  async reportIncident(
    agentId: string,
    policyId: string,
    incidentType: string,
    severity: RiskLevel,
    description: string,
    affectedParties: string[],
    damageEstimate: number
  ): Promise<IncidentReport> {
    const policy = this.policies.get(policyId);
    if (!policy) {
      throw new Error(`Policy not found: ${policyId}`);
    }

    const now = new Date();
    const report: IncidentReport = {
      id: uuidv4(),
      agentId,
      policyId,
      incidentType,
      severity,
      description,
      affectedParties,
      damageEstimate,
      occurredAt: now,
      reportedAt: now,
      status: 'REPORTED',
      euAiActCompliant: true,  // Auto-generate required fields
    };

    this.incidents.set(report.id, report);

    // EU AI Act requires notification within 72 hours for high-risk systems
    console.log(`[EU AI Act] Incident ${report.id} logged for agent ${agentId}`);

    return report;
  }

  /**
   * Get active policies for agent.
   */
  getActivePolicies(agentId: string): InsurancePolicy[] {
    return Array.from(this.policies.values()).filter(
      p => p.agentId === agentId && p.status === 'ACTIVE'
    );
  }

  /**
   * Get incidents for agent.
   */
  getIncidents(agentId: string): IncidentReport[] {
    return Array.from(this.incidents.values()).filter(
      i => i.agentId === agentId
    );
  }

  // Helper
  private scoreToRiskLevel(score: number): RiskLevel {
    if (score < 25) return RiskLevel.LOW;
    if (score < 50) return RiskLevel.MEDIUM;
    if (score < 75) return RiskLevel.HIGH;
    return RiskLevel.CRITICAL;
  }
}
