//! VeriMantle-Identity: Decentralized Identity (DID) Registry
//!
//! Per FUTURE_INNOVATION_ROADMAP.md Innovation #5:
//! W3C DID-compliant decentralized registry for agent identities.
//!
//! Features:
//! - did:verimantle method specification
//! - W3C DID Document format
//! - On-chain reputation storage (optional)
//! - Key recovery mechanism
//! - Cross-platform portability

import { v4 as uuidv4 } from 'uuid';

// ============================================================================
// DID TYPES
// ============================================================================

/**
 * DID Document per W3C spec.
 */
export interface DIDDocument {
  '@context': string[];
  id: string;  // did:verimantle:xxx
  controller?: string;
  verificationMethod: VerificationMethod[];
  authentication: string[];
  assertionMethod?: string[];
  keyAgreement?: string[];
  service?: ServiceEndpoint[];
  created: string;
  updated: string;
}

/**
 * Verification method (key).
 */
export interface VerificationMethod {
  id: string;
  type: string;
  controller: string;
  publicKeyJwk?: JsonWebKey;
  publicKeyMultibase?: string;
}

/**
 * Service endpoint.
 */
export interface ServiceEndpoint {
  id: string;
  type: string;
  serviceEndpoint: string;
  description?: string;
}

/**
 * DID Resolution result.
 */
export interface DIDResolutionResult {
  didDocument: DIDDocument | null;
  didResolutionMetadata: {
    contentType?: string;
    error?: string;
  };
  didDocumentMetadata: {
    created?: string;
    updated?: string;
    deactivated?: boolean;
  };
}

/**
 * Recovery key for agent identity.
 */
export interface RecoveryKey {
  id: string;
  threshold: number;  // Required shares to recover
  totalShares: number;
  holders: string[];  // DIDs of key holders
}

/**
 * On-chain reputation record.
 */
export interface ReputationRecord {
  did: string;
  trustScore: number;
  transactionCount: number;
  positiveInteractions: number;
  negativeInteractions: number;
  endorsements: string[];  // DIDs that endorsed this agent
  lastUpdated: string;
}

// ============================================================================
// DID REGISTRY
// ============================================================================

/**
 * Decentralized Identity Registry for VeriMantle agents.
 */
export class DIDRegistry {
  private documents: Map<string, DIDDocument> = new Map();
  private reputation: Map<string, ReputationRecord> = new Map();
  private recoveryKeys: Map<string, RecoveryKey> = new Map();

  /**
   * Create a new DID for an agent.
   */
  async createDID(
    agentName: string,
    publicKey: JsonWebKey,
    options: {
      controller?: string;
      services?: ServiceEndpoint[];
    } = {}
  ): Promise<DIDDocument> {
    const id = `did:verimantle:${uuidv4()}`;
    const now = new Date().toISOString();

    const verificationMethodId = `${id}#key-1`;

    const doc: DIDDocument = {
      '@context': [
        'https://www.w3.org/ns/did/v1',
        'https://verimantle.io/ns/did/v1',
      ],
      id,
      controller: options.controller,
      verificationMethod: [
        {
          id: verificationMethodId,
          type: 'JsonWebKey2020',
          controller: id,
          publicKeyJwk: publicKey,
        },
      ],
      authentication: [verificationMethodId],
      assertionMethod: [verificationMethodId],
      service: options.services || [
        {
          id: `${id}#verimantle`,
          type: 'VeriMantleAgent',
          serviceEndpoint: 'https://api.verimantle.io/agents',
          description: agentName,
        },
      ],
      created: now,
      updated: now,
    };

    this.documents.set(id, doc);

    // Initialize reputation
    this.reputation.set(id, {
      did: id,
      trustScore: 50,  // Start neutral
      transactionCount: 0,
      positiveInteractions: 0,
      negativeInteractions: 0,
      endorsements: [],
      lastUpdated: now,
    });

    return doc;
  }

  /**
   * Resolve a DID to its document.
   */
  async resolve(did: string): Promise<DIDResolutionResult> {
    // Validate DID format
    if (!did.startsWith('did:verimantle:')) {
      return {
        didDocument: null,
        didResolutionMetadata: {
          error: 'invalidDid',
        },
        didDocumentMetadata: {},
      };
    }

    const doc = this.documents.get(did);
    
    if (!doc) {
      return {
        didDocument: null,
        didResolutionMetadata: {
          error: 'notFound',
        },
        didDocumentMetadata: {},
      };
    }

    return {
      didDocument: doc,
      didResolutionMetadata: {
        contentType: 'application/did+json',
      },
      didDocumentMetadata: {
        created: doc.created,
        updated: doc.updated,
      },
    };
  }

  /**
   * Update a DID document.
   */
  async update(
    did: string,
    updates: Partial<DIDDocument>
  ): Promise<DIDDocument> {
    const existing = this.documents.get(did);
    if (!existing) {
      throw new Error(`DID not found: ${did}`);
    }

    const updated: DIDDocument = {
      ...existing,
      ...updates,
      id: existing.id,  // ID cannot change
      '@context': existing['@context'],
      updated: new Date().toISOString(),
    };

    this.documents.set(did, updated);
    return updated;
  }

  /**
   * Deactivate a DID.
   */
  async deactivate(did: string): Promise<void> {
    const doc = this.documents.get(did);
    if (doc) {
      // Remove verification methods
      doc.verificationMethod = [];
      doc.authentication = [];
      doc.updated = new Date().toISOString();
    }
  }

  /**
   * Add a verification method (key rotation).
   */
  async addKey(
    did: string,
    key: JsonWebKey,
    purposes: ('authentication' | 'assertionMethod' | 'keyAgreement')[]
  ): Promise<DIDDocument> {
    const doc = this.documents.get(did);
    if (!doc) {
      throw new Error(`DID not found: ${did}`);
    }

    const keyId = `${did}#key-${doc.verificationMethod.length + 1}`;
    
    doc.verificationMethod.push({
      id: keyId,
      type: 'JsonWebKey2020',
      controller: did,
      publicKeyJwk: key,
    });

    for (const purpose of purposes) {
      if (purpose === 'authentication') {
        doc.authentication.push(keyId);
      } else if (purpose === 'assertionMethod') {
        doc.assertionMethod = doc.assertionMethod || [];
        doc.assertionMethod.push(keyId);
      } else if (purpose === 'keyAgreement') {
        doc.keyAgreement = doc.keyAgreement || [];
        doc.keyAgreement.push(keyId);
      }
    }

    doc.updated = new Date().toISOString();
    return doc;
  }

  /**
   * Setup recovery mechanism.
   */
  async setupRecovery(
    did: string,
    threshold: number,
    holders: string[]
  ): Promise<RecoveryKey> {
    const recovery: RecoveryKey = {
      id: `${did}#recovery`,
      threshold,
      totalShares: holders.length,
      holders,
    };

    this.recoveryKeys.set(did, recovery);
    return recovery;
  }

  /**
   * Get reputation for a DID.
   */
  async getReputation(did: string): Promise<ReputationRecord | null> {
    return this.reputation.get(did) || null;
  }

  /**
   * Record an interaction (positive or negative).
   */
  async recordInteraction(
    did: string,
    positive: boolean
  ): Promise<ReputationRecord> {
    let rep = this.reputation.get(did);
    if (!rep) {
      throw new Error(`DID not found: ${did}`);
    }

    rep.transactionCount++;
    if (positive) {
      rep.positiveInteractions++;
    } else {
      rep.negativeInteractions++;
    }

    // Recalculate trust score
    const total = rep.positiveInteractions + rep.negativeInteractions;
    if (total > 0) {
      rep.trustScore = Math.round((rep.positiveInteractions / total) * 100);
    }

    rep.lastUpdated = new Date().toISOString();
    return rep;
  }

  /**
   * Endorse an agent.
   */
  async endorse(endorserDid: string, targetDid: string): Promise<void> {
    const rep = this.reputation.get(targetDid);
    if (!rep) {
      throw new Error(`DID not found: ${targetDid}`);
    }

    if (!rep.endorsements.includes(endorserDid)) {
      rep.endorsements.push(endorserDid);
      rep.trustScore = Math.min(100, rep.trustScore + 5);
      rep.lastUpdated = new Date().toISOString();
    }
  }

  /**
   * List all registered DIDs.
   */
  listAll(): string[] {
    return Array.from(this.documents.keys());
  }

  /**
   * Find DIDs by trust score threshold.
   */
  findByTrustScore(minScore: number): string[] {
    const results: string[] = [];
    for (const [did, rep] of this.reputation) {
      if (rep.trustScore >= minScore) {
        results.push(did);
      }
    }
    return results;
  }
}
