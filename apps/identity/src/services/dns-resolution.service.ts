/**
 * AgentKern Identity - DNS Resolution Service
 * 
 * Provides global, cacheable trust resolution for AI agents.
 * Implements the Intent DNS protocol.
 */

import { Injectable, Logger } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import {
  TrustRecord,
  TrustQuery,
  TrustResolution,
  createTrustRecord,
  createTrustResolution,
  calculateTrustScore,
  isTrusted,
  calculateTTL,
} from '../domain/trust-record.entity';
import { AuditLoggerService, AuditEventType } from './audit-logger.service';
import { TrustRecordEntity } from '../entities/trust-record.entity';

interface CacheEntry {
  resolution: TrustResolution;
  expiresAt: number;
}

@Injectable()
export class DnsResolutionService {
  private readonly logger = new Logger(DnsResolutionService.name);
  
  // Resolution cache
  private cache: Map<string, CacheEntry> = new Map();

  constructor(
    private readonly auditLogger: AuditLoggerService,
    @InjectRepository(TrustRecordEntity)
    private readonly trustRepository: Repository<TrustRecordEntity>,
  ) {}

  /**
   * Resolve trust for an agent-principal pair
   */
  async resolve(query: TrustQuery): Promise<TrustResolution> {
    const cacheKey = this.getCacheKey(query);
    
    // Check cache first
    const cached = this.getFromCache(cacheKey);
    if (cached) {
      this.logger.debug(`Cache hit for ${cacheKey}`);
      return cached;
    }

    // Look up or create trust record
    let record = await this.trustRepository.findOne({
      where: { agentId: query.agentId, principalId: query.principalId },
    });
    
    if (!record) {
      // Create new record for unknown agent-principal pair
      const newRecord = createTrustRecord(query.agentId, query.principalId);
      record = await this.trustRepository.save(this.trustRepository.create(newRecord));
      
      this.logger.log(`Created new trust record for ${query.agentId}:${query.principalId}`);
    }

    // Create resolution
    const resolution = createTrustResolution(record);
    
    // Cache the resolution
    if (resolution.ttl > 0) {
      this.setCache(cacheKey, resolution);
    }

    return resolution;
  }

  /**
   * Batch resolve multiple queries
   */
  async resolveBatch(queries: TrustQuery[]): Promise<TrustResolution[]> {
    return Promise.all(queries.map(query => this.resolve(query)));
  }

  /**
   * Register a new agent-principal trust relationship
   */
  async registerTrust(
    agentId: string,
    principalId: string,
    metadata?: { agentName?: string; agentVersion?: string; principalDevice?: string },
  ): Promise<TrustRecord> {
    const cacheKey = this.getCacheKey({ agentId, principalId });
    
    const record = createTrustRecord(agentId, principalId, metadata);
    const saved = await this.trustRepository.save(this.trustRepository.create(record));
    
    // Invalidate cache
    this.cache.delete(cacheKey);
    
    this.logger.log(`Registered trust for ${agentId}:${principalId}`);
    
    return saved;
  }

  /**
   * Record a successful verification (increases trust score)
   */
  async recordVerificationSuccess(agentId: string, principalId: string): Promise<TrustRecord | null> {
    const cacheKey = this.getCacheKey({ agentId, principalId });
    const record = await this.trustRepository.findOne({ where: { agentId, principalId } });
    
    if (!record) return null;

    record.verificationCount++;
    record.lastVerifiedAt = new Date();
    record.trustScore = this.recalculateScore(record);
    record.trusted = isTrusted(record.trustScore);
    
    await this.trustRepository.save(record);
    
    // Invalidate cache
    this.cache.delete(cacheKey);
    
    return record;
  }

  /**
   * Record a failed verification (decreases trust score)
   */
  async recordVerificationFailure(agentId: string, principalId: string): Promise<TrustRecord | null> {
    const cacheKey = this.getCacheKey({ agentId, principalId });
    const record = await this.trustRepository.findOne({ where: { agentId, principalId } });
    
    if (!record) return null;

    record.failureCount++;
    record.trustScore = this.recalculateScore(record);
    record.trusted = isTrusted(record.trustScore);
    
    await this.trustRepository.save(record);
    
    // Immediate cache invalidation on failure
    this.cache.delete(cacheKey);
    
    this.auditLogger.logSecurityEvent(
      AuditEventType.PROOF_VERIFICATION_FAILURE,
      `Trust degraded for ${agentId}:${principalId}`,
      { newScore: record.trustScore },
    );
    
    return record;
  }

  /**
   * Revoke trust for an agent-principal pair
   */
  async revokeTrust(agentId: string, principalId: string, reason: string): Promise<TrustRecord | null> {
    const cacheKey = this.getCacheKey({ agentId, principalId });
    const record = await this.trustRepository.findOne({ where: { agentId, principalId } });
    
    if (!record) return null;

    record.revoked = true;
    record.trusted = false;
    record.trustScore = Math.max(0, record.trustScore - 200);
    
    await this.trustRepository.save(record);
    
    // Immediate cache invalidation
    this.cache.delete(cacheKey);
    
    this.auditLogger.log({
      type: AuditEventType.KEY_REVOKED,
      agentId,
      principalId,
      success: true,
      metadata: { reason },
    });
    
    this.logger.warn(`Trust revoked for ${agentId}:${principalId}: ${reason}`);
    
    return record;
  }

  /**
   * Reinstate previously revoked trust
   */
  async reinstateTrust(agentId: string, principalId: string): Promise<TrustRecord | null> {
    const cacheKey = this.getCacheKey({ agentId, principalId });
    const record = await this.trustRepository.findOne({ where: { agentId, principalId } });
    
    if (!record) return null;

    record.revoked = false;
    record.trusted = isTrusted(record.trustScore);
    
    await this.trustRepository.save(record);
    
    // Invalidate cache
    this.cache.delete(cacheKey);
    
    this.logger.log(`Trust reinstated for ${agentId}:${principalId}`);
    
    return record;
  }

  /**
   * Get all trust records for a principal
   */
  async getTrustRecordsForPrincipal(principalId: string): Promise<TrustRecord[]> {
    return this.trustRepository.find({ where: { principalId } });
  }

  /**
   * Get a specific trust record
   */
  async getTrustRecord(agentId: string, principalId: string): Promise<TrustRecord | null> {
    return this.trustRepository.findOne({ where: { agentId, principalId } });
  }

  /**
   * Recalculate trust score based on record history
   */
  private recalculateScore(record: TrustRecord): number {
    const now = new Date();
    const lastVerified = new Date(record.lastVerifiedAt);
    const registered = new Date(record.registeredAt);
    
    const daysSinceLastVerification = Math.floor(
      (now.getTime() - lastVerified.getTime()) / (1000 * 60 * 60 * 24)
    );
    
    const daysActive = Math.floor(
      (now.getTime() - registered.getTime()) / (1000 * 60 * 60 * 24)
    );
    
    return calculateTrustScore(
      record.verificationCount,
      record.failureCount,
      daysSinceLastVerification,
      daysActive,
    );
  }

  private getCacheKey(query: TrustQuery): string {
    return `${query.agentId}:${query.principalId}`;
  }

  private getFromCache(key: string): TrustResolution | null {
    const entry = this.cache.get(key);
    
    if (!entry) return null;
    
    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      return null;
    }
    
    return entry.resolution;
  }

  private setCache(key: string, resolution: TrustResolution): void {
    this.cache.set(key, {
      resolution,
      expiresAt: Date.now() + (resolution.ttl * 1000),
    });
  }
}
