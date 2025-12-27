//! AgentKern: Agent Legal Entity Framework
//!
//! Per FUTURE_INNOVATION_ROADMAP.md Innovation #3:
//! Legal incorporation of AI agents as LLCs/DAOs.
//!
//! This is the "ultimate moat" - no competitor is building legal infrastructure.
//!
//! Features:
//! - Entity registration (LLC, DAO, Trust)
//! - Jurisdiction support (Wyoming, Cayman, Switzerland)
//! - Operating agreement templates
//! - Treasury wallet integration
//! - Liability shield management

import { v4 as uuidv4 } from 'uuid';

// ============================================================================
// LEGAL ENTITY TYPES
// ============================================================================

/**
 * Legal entity types.
 */
export enum EntityType {
  /** Limited Liability Company */
  LLC = 'LLC',
  /** Decentralized Autonomous Organization */
  DAO = 'DAO',
  /** Trust */
  TRUST = 'TRUST',
  /** Foundation */
  FOUNDATION = 'FOUNDATION',
}

/**
 * Supported jurisdictions.
 */
export enum Jurisdiction {
  /** Wyoming, USA - Already allows DAOs */
  WYOMING = 'WYOMING',
  /** Delaware, USA - Business-friendly */
  DELAWARE = 'DELAWARE',
  /** Cayman Islands - Flexible structures */
  CAYMAN = 'CAYMAN',
  /** Switzerland - Association model */
  SWITZERLAND = 'SWITZERLAND',
  /** Singapore - Innovation-friendly */
  SINGAPORE = 'SINGAPORE',
}

/**
 * Entity status.
 */
export enum EntityStatus {
  DRAFT = 'DRAFT',
  PENDING_REGISTRATION = 'PENDING_REGISTRATION',
  ACTIVE = 'ACTIVE',
  SUSPENDED = 'SUSPENDED',
  DISSOLVED = 'DISSOLVED',
}

/**
 * Legal entity for an AI agent.
 */
export interface AgentLegalEntity {
  id: string;
  agentId: string;
  entityType: EntityType;
  jurisdiction: Jurisdiction;
  legalName: string;
  registrationNumber?: string;
  registeredAgent: string;  // Human or service
  operatingAgreementHash: string;
  treasuryWalletId: string;
  insurancePolicyId?: string;
  status: EntityStatus;
  createdAt: Date;
  registeredAt?: Date;
  metadata: Record<string, unknown>;
}

/**
 * Operating agreement.
 */
export interface OperatingAgreement {
  id: string;
  entityId: string;
  version: string;
  contentHash: string;
  signatories: Signatory[];
  effectiveDate: Date;
  terms: OperatingTerms;
}

/**
 * Signatory to the agreement.
 */
export interface Signatory {
  id: string;
  name: string;
  role: 'MEMBER' | 'MANAGER' | 'HUMAN_CONTROLLER' | 'AI_AGENT';
  signedAt?: Date;
  signatureHash?: string;
}

/**
 * Operating terms.
 */
export interface OperatingTerms {
  profitDistribution: number[];  // Percentages
  votingThreshold: number;  // Percentage needed
  emergencyShutdown: string;  // Who can trigger
  liabilityLimit: number;  // USD
  annualFilingRequired: boolean;
  humanOversightRequired: boolean;
}

/**
 * Liability event.
 */
export interface LiabilityEvent {
  id: string;
  entityId: string;
  eventType: string;
  claimant?: string;
  amount: number;
  description: string;
  status: 'FILED' | 'INVESTIGATING' | 'SETTLED' | 'DISMISSED';
  occurredAt: Date;
  resolvedAt?: Date;
}

/**
 * Jurisdiction info.
 */
export interface JurisdictionInfo {
  jurisdiction: Jurisdiction;
  supportedTypes: EntityType[];
  registrationFee: number;
  annualFee: number;
  formationTime: string;  // e.g., "2-3 weeks"
  requirements: string[];
  benefits: string[];
}

// Jurisdiction details
export const JURISDICTION_INFO: Record<Jurisdiction, JurisdictionInfo> = {
  [Jurisdiction.WYOMING]: {
    jurisdiction: Jurisdiction.WYOMING,
    supportedTypes: [EntityType.LLC, EntityType.DAO],
    registrationFee: 100,
    annualFee: 52,
    formationTime: '1-2 days',
    requirements: [
      'Registered agent in Wyoming',
      'Articles of Organization',
      'AI disclosure (for DAOs)',
    ],
    benefits: [
      'First state to legally recognize DAOs',
      'No state income tax',
      'Strong asset protection',
      'AI-specific legislation',
    ],
  },
  [Jurisdiction.DELAWARE]: {
    jurisdiction: Jurisdiction.DELAWARE,
    supportedTypes: [EntityType.LLC, EntityType.TRUST],
    registrationFee: 90,
    annualFee: 300,
    formationTime: '24 hours (expedited)',
    requirements: [
      'Registered agent in Delaware',
      'Certificate of Formation',
    ],
    benefits: [
      'Established business court',
      'Flexible LLC laws',
      'Privacy protections',
    ],
  },
  [Jurisdiction.CAYMAN]: {
    jurisdiction: Jurisdiction.CAYMAN,
    supportedTypes: [EntityType.LLC, EntityType.FOUNDATION],
    registrationFee: 5000,
    annualFee: 3000,
    formationTime: '2-3 weeks',
    requirements: [
      'Local registered office',
      'Memorandum of Association',
      'Anti-money laundering documentation',
    ],
    benefits: [
      'No direct taxation',
      'Flexible foundation structures',
      'International recognition',
    ],
  },
  [Jurisdiction.SWITZERLAND]: {
    jurisdiction: Jurisdiction.SWITZERLAND,
    supportedTypes: [EntityType.FOUNDATION, EntityType.DAO],
    registrationFee: 2000,
    annualFee: 1000,
    formationTime: '4-6 weeks',
    requirements: [
      'Swiss foundation deed',
      'Purpose statement',
      'Regulatory approval for crypto-related',
    ],
    benefits: [
      'Strong privacy laws',
      'Crypto-friendly environment',
      'Association model for DAOs',
    ],
  },
  [Jurisdiction.SINGAPORE]: {
    jurisdiction: Jurisdiction.SINGAPORE,
    supportedTypes: [EntityType.LLC],
    registrationFee: 350,
    annualFee: 200,
    formationTime: '1-2 days',
    requirements: [
      'Local director required',
      'Registered office in Singapore',
      'Constitution document',
    ],
    benefits: [
      'Low corporate tax',
      'Innovation-friendly regulations',
      'Strong IP protection',
    ],
  },
};

// ============================================================================
// LEGAL ENTITY SERVICE
// ============================================================================

/**
 * Agent Legal Entity Service.
 */
export class LegalEntityService {
  private entities: Map<string, AgentLegalEntity> = new Map();
  private agreements: Map<string, OperatingAgreement> = new Map();
  private liabilityEvents: Map<string, LiabilityEvent[]> = new Map();

  /**
   * Create a draft legal entity for an agent.
   */
  async createEntity(
    agentId: string,
    entityType: EntityType,
    jurisdiction: Jurisdiction,
    legalName: string,
    registeredAgent: string,
    treasuryWalletId: string
  ): Promise<AgentLegalEntity> {
    // Validate jurisdiction supports entity type
    const info = JURISDICTION_INFO[jurisdiction];
    if (!info.supportedTypes.includes(entityType)) {
      throw new Error(
        `${jurisdiction} does not support ${entityType}. Supported: ${info.supportedTypes.join(', ')}`
      );
    }

    const entity: AgentLegalEntity = {
      id: uuidv4(),
      agentId,
      entityType,
      jurisdiction,
      legalName,
      registeredAgent,
      operatingAgreementHash: '',
      treasuryWalletId,
      status: EntityStatus.DRAFT,
      createdAt: new Date(),
      metadata: {},
    };

    this.entities.set(entity.id, entity);
    this.liabilityEvents.set(entity.id, []);

    return entity;
  }

  /**
   * Create operating agreement.
   */
  async createOperatingAgreement(
    entityId: string,
    signatories: Signatory[],
    terms: OperatingTerms
  ): Promise<OperatingAgreement> {
    const entity = this.entities.get(entityId);
    if (!entity) {
      throw new Error(`Entity not found: ${entityId}`);
    }

    // Require at least one human controller for safety
    const hasHumanController = signatories.some(
      s => s.role === 'HUMAN_CONTROLLER' || s.role === 'MANAGER'
    );
    if (!hasHumanController && terms.humanOversightRequired) {
      throw new Error('Operating agreement requires at least one human controller');
    }

    const agreement: OperatingAgreement = {
      id: uuidv4(),
      entityId,
      version: '1.0',
      contentHash: this.hashAgreement(terms),
      signatories,
      effectiveDate: new Date(),
      terms,
    };

    this.agreements.set(agreement.id, agreement);

    // Update entity
    entity.operatingAgreementHash = agreement.contentHash;

    return agreement;
  }

  /**
   * Sign operating agreement.
   */
  async signAgreement(
    agreementId: string,
    signatoryId: string,
    signatureHash: string
  ): Promise<void> {
    const agreement = this.agreements.get(agreementId);
    if (!agreement) {
      throw new Error(`Agreement not found: ${agreementId}`);
    }

    const signatory = agreement.signatories.find(s => s.id === signatoryId);
    if (!signatory) {
      throw new Error(`Signatory not found: ${signatoryId}`);
    }

    signatory.signedAt = new Date();
    signatory.signatureHash = signatureHash;
  }

  /**
   * Submit for registration.
   */
  async submitForRegistration(entityId: string): Promise<void> {
    const entity = this.entities.get(entityId);
    if (!entity) {
      throw new Error(`Entity not found: ${entityId}`);
    }

    if (!entity.operatingAgreementHash) {
      throw new Error('Operating agreement required before registration');
    }

    // Check all signatories have signed
    const agreement = Array.from(this.agreements.values()).find(
      a => a.contentHash === entity.operatingAgreementHash
    );
    if (agreement) {
      const unsigned = agreement.signatories.filter(s => !s.signedAt);
      if (unsigned.length > 0) {
        throw new Error(`Unsigned signatories: ${unsigned.map(s => s.name).join(', ')}`);
      }
    }

    entity.status = EntityStatus.PENDING_REGISTRATION;
    
    // In production, this would submit to the jurisdiction's registration system
    console.log(`[LegalEntity] Submitted ${entity.legalName} for registration in ${entity.jurisdiction}`);
  }

  /**
   * Complete registration (called by admin or webhook).
   */
  async completeRegistration(
    entityId: string,
    registrationNumber: string
  ): Promise<void> {
    const entity = this.entities.get(entityId);
    if (!entity) {
      throw new Error(`Entity not found: ${entityId}`);
    }

    entity.registrationNumber = registrationNumber;
    entity.registeredAt = new Date();
    entity.status = EntityStatus.ACTIVE;
  }

  /**
   * File a liability claim.
   */
  async fileLiabilityClaim(
    entityId: string,
    eventType: string,
    claimant: string,
    amount: number,
    description: string
  ): Promise<LiabilityEvent> {
    const entity = this.entities.get(entityId);
    if (!entity) {
      throw new Error(`Entity not found: ${entityId}`);
    }

    const event: LiabilityEvent = {
      id: uuidv4(),
      entityId,
      eventType,
      claimant,
      amount,
      description,
      status: 'FILED',
      occurredAt: new Date(),
    };

    const events = this.liabilityEvents.get(entityId) || [];
    events.push(event);
    this.liabilityEvents.set(entityId, events);

    return event;
  }

  /**
   * Get entity by agent ID.
   */
  getByAgentId(agentId: string): AgentLegalEntity | undefined {
    return Array.from(this.entities.values()).find(e => e.agentId === agentId);
  }

  /**
   * Get entity by ID.
   */
  getEntity(entityId: string): AgentLegalEntity | undefined {
    return this.entities.get(entityId);
  }

  /**
   * Get operating agreement.
   */
  getAgreement(entityId: string): OperatingAgreement | undefined {
    const entity = this.entities.get(entityId);
    if (!entity) return undefined;
    
    return Array.from(this.agreements.values()).find(
      a => a.contentHash === entity.operatingAgreementHash
    );
  }

  /**
   * Get liability events.
   */
  getLiabilityEvents(entityId: string): LiabilityEvent[] {
    return this.liabilityEvents.get(entityId) || [];
  }

  /**
   * Get jurisdiction requirements.
   */
  getJurisdictionInfo(jurisdiction: Jurisdiction): JurisdictionInfo {
    return JURISDICTION_INFO[jurisdiction];
  }

  /**
   * Check if entity can perform action.
   */
  async canPerformAction(
    entityId: string,
    actionValue: number
  ): Promise<{ allowed: boolean; reason?: string }> {
    const entity = this.entities.get(entityId);
    if (!entity) {
      return { allowed: false, reason: 'Entity not found' };
    }

    if (entity.status !== EntityStatus.ACTIVE) {
      return { allowed: false, reason: `Entity status: ${entity.status}` };
    }

    const agreement = this.getAgreement(entityId);
    if (agreement && actionValue > agreement.terms.liabilityLimit) {
      return {
        allowed: false,
        reason: `Action value ${actionValue} exceeds liability limit ${agreement.terms.liabilityLimit}`,
      };
    }

    return { allowed: true };
  }

  // Hash the agreement terms for integrity
  private hashAgreement(terms: OperatingTerms): string {
    const content = JSON.stringify(terms);
    // Simple hash for demo; use crypto.subtle in production
    let hash = 0;
    for (let i = 0; i < content.length; i++) {
      const chr = content.charCodeAt(i);
      hash = ((hash << 5) - hash) + chr;
      hash |= 0;
    }
    return `0x${Math.abs(hash).toString(16)}`;
  }
}
