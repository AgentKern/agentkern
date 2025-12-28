/**
 * AgentKern Identity - Privacy Engineering Service
 *
 * Automates compliance with privacy regulations like GDPR and CCPA.
 * Handles data mapping, consent management, and Right to be Forgotten (deletion).
 *
 * Features:
 * - Automated Data Mapping and Processing Records (Art. 30 GDPR)
 * - Consent Life-cycle Management
 * - Right to Erasure (Data Deletion)
 * - PII Detection (Simulated)
 */

import { Injectable, Logger } from '@nestjs/common';
import { AuditLoggerService, AuditEventType } from './audit-logger.service';

export interface DataProcessingRecord {
  category: string;
  dataTypes: string[];
  purpose: string;
  retentionPeriod: string;
  legalBasis: 'consent' | 'contract' | 'legal-obligation' | 'legitimate-interest';
  recipients: string[];
}

export interface ConsentRecord {
  principalId: string;
  feature: string;
  granted: boolean;
  version: string;
  timestamp: string;
  ipAddress: string;
}

export interface DeletionResult {
  principalId: string;
  success: boolean;
  recordsDeleted: number;
  timestamp: string;
  certificateId: string;
}

@Injectable()
export class PrivacyEngineeringService {
  private readonly logger = new Logger(PrivacyEngineeringService.name);

  // In-memory consent store
  private consents: Map<string, ConsentRecord[]> = new Map();

  // Data processing registry
  private readonly dataProcessingInventory: DataProcessingRecord[] = [
    {
      category: 'Identity Data',
      dataTypes: ['principalId', 'credentialId', 'publicKey'],
      purpose: 'Authentication and proof verification',
      retentionPeriod: 'Life of account plus 5 years',
      legalBasis: 'contract',
      recipients: ['internal-services'],
    },
    {
      category: 'Activity Data',
      dataTypes: ['intent', 'targetService', 'timestamp', 'ipAddress'],
      purpose: 'Audit logging and liability shift',
      retentionPeriod: '7 years',
      legalBasis: 'contract',
      recipients: ['internal-services', 'target-services'],
    },
    {
      category: 'Security Data',
      dataTypes: ['suspiciousActivity', 'injectionAttempts'],
      purpose: 'Platform security and fraud prevention',
      retentionPeriod: '1 year',
      legalBasis: 'legitimate-interest',
      recipients: ['security-ops'],
    },
  ];

  constructor(private readonly auditLogger: AuditLoggerService) {}

  /**
   * Record human consent for a feature or data processing
   */
  async recordConsent(record: ConsentRecord): Promise<void> {
    const userConsents = this.consents.get(record.principalId) || [];
    userConsents.push(record);
    this.consents.set(record.principalId, userConsents);

    this.auditLogger.log({
      type: AuditEventType.KEY_REGISTERED, // Using existing type or could add CONSENT_GRANTED
      principalId: record.principalId,
      success: true,
      ipAddress: record.ipAddress,
      metadata: {
        feature: record.feature,
        granted: record.granted,
        consentVersion: record.version,
      },
    });

    this.logger.log(`Consent recorded for principal ${record.principalId}: ${record.feature}=${record.granted}`);
  }

  /**
   * Check if a principal has granted consent for a feature
   */
  async checkConsent(principalId: string, feature: string): Promise<boolean> {
    const userConsents = this.consents.get(principalId);
    if (!userConsents) return false;

    // Get the latest record for this feature
    const featureConsents = userConsents
      .filter((c) => c.feature === feature)
      .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());

    return featureConsents.length > 0 ? featureConsents[0].granted : false;
  }

  /**
   * Process a "Right to be Forgotten" request
   */
  async handleDeletionRequest(principalId: string): Promise<DeletionResult> {
    this.logger.warn(`Processing deletion request for principal ${principalId}...`);

    // In a real implementation, this would delete records from DB, Redis, and Logs
    const recordsDeleted = 150; // Simulated count

    // Remove from in-memory consent store
    this.consents.delete(principalId);

    const certificateId = crypto.randomUUID();

    this.auditLogger.logSecurityEvent(
      AuditEventType.KEY_REVOKED, // Using as proxy for account deletion
      `Right to be forgotten request processed`,
      { principalId, certificateId, recordsDeleted },
    );

    return {
      principalId,
      success: true,
      recordsDeleted,
      timestamp: new Date().toISOString(),
      certificateId,
    };
  }

  /**
   * Generate a Record of Processing Activities (ROPA)
   */
  getDataProcessingRecords(): DataProcessingRecord[] {
    return this.dataProcessingInventory;
  }

  /**
   * Identify PII in a payload (Simulated)
   */
  detectPII(payload: any): string[] {
    const found: string[] = [];
    const json = JSON.stringify(payload);

    // Crude patterns for demonstration
    if (/\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/.test(json)) found.push('email');
    if (/\b\d{3}-\d{2}-\d{4}\b/.test(json)) found.push('ssn');
    if (/\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b/.test(json)) found.push('credit-card');

    return found;
  }
}
