/**
 * AgentKern Identity - Trust Record Repository
 * 
 * PostgreSQL repository for Trust Records.
 */

import { Injectable, Logger } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { TrustRecordEntity } from '../entities/trust-record.entity';

@Injectable()
export class TrustRecordRepository {
  private readonly logger = new Logger(TrustRecordRepository.name);

  constructor(
    @InjectRepository(TrustRecordEntity)
    private readonly repository: Repository<TrustRecordEntity>,
  ) {}

  /**
   * Find trust record by agent and principal
   */
  async findByAgentAndPrincipal(
    agentId: string,
    principalId: string,
  ): Promise<TrustRecordEntity | null> {
    return this.repository.findOne({
      where: { agentId, principalId },
    });
  }

  /**
   * Find all records for a principal
   */
  async findByPrincipal(principalId: string): Promise<TrustRecordEntity[]> {
    return this.repository.find({
      where: { principalId },
      order: { trustScore: 'DESC' },
    });
  }

  /**
   * Create or update a trust record
   */
  async upsert(data: Partial<TrustRecordEntity>): Promise<TrustRecordEntity> {
    const existing = await this.findByAgentAndPrincipal(
      data.agentId!,
      data.principalId!,
    );

    if (existing) {
      // Update existing
      Object.assign(existing, data);
      return this.repository.save(existing);
    }

    // Create new
    const record = this.repository.create({
      ...data,
      trustScore: data.trustScore ?? 500,
      trusted: data.trusted ?? true,
      revoked: data.revoked ?? false,
      verificationCount: data.verificationCount ?? 0,
      failureCount: data.failureCount ?? 0,
    });

    return this.repository.save(record);
  }

  /**
   * Increment verification count
   */
  async recordVerificationSuccess(
    agentId: string,
    principalId: string,
  ): Promise<TrustRecordEntity | null> {
    const record = await this.findByAgentAndPrincipal(agentId, principalId);
    if (!record) return null;

    record.verificationCount++;
    record.lastVerifiedAt = new Date();
    record.trustScore = this.recalculateScore(record);
    record.trusted = record.trustScore >= 500;

    return this.repository.save(record);
  }

  /**
   * Increment failure count
   */
  async recordVerificationFailure(
    agentId: string,
    principalId: string,
  ): Promise<TrustRecordEntity | null> {
    const record = await this.findByAgentAndPrincipal(agentId, principalId);
    if (!record) return null;

    record.failureCount++;
    record.trustScore = this.recalculateScore(record);
    record.trusted = record.trustScore >= 500;

    return this.repository.save(record);
  }

  /**
   * Revoke trust
   */
  async revoke(
    agentId: string,
    principalId: string,
  ): Promise<TrustRecordEntity | null> {
    const record = await this.findByAgentAndPrincipal(agentId, principalId);
    if (!record) return null;

    record.revoked = true;
    record.trusted = false;
    record.trustScore = Math.max(0, record.trustScore - 200);

    return this.repository.save(record);
  }

  /**
   * Reinstate trust
   */
  async reinstate(
    agentId: string,
    principalId: string,
  ): Promise<TrustRecordEntity | null> {
    const record = await this.findByAgentAndPrincipal(agentId, principalId);
    if (!record) return null;

    record.revoked = false;
    record.trusted = record.trustScore >= 500;

    return this.repository.save(record);
  }

  /**
   * Get statistics
   */
  async getStats(): Promise<{
    totalRecords: number;
    trustedCount: number;
    revokedCount: number;
    avgScore: number;
  }> {
    const result = await this.repository
      .createQueryBuilder('tr')
      .select('COUNT(*)', 'totalRecords')
      .addSelect('SUM(CASE WHEN trusted = true THEN 1 ELSE 0 END)', 'trustedCount')
      .addSelect('SUM(CASE WHEN revoked = true THEN 1 ELSE 0 END)', 'revokedCount')
      .addSelect('AVG(trustScore)', 'avgScore')
      .getRawOne();

    return {
      totalRecords: parseInt(result.totalRecords, 10) || 0,
      trustedCount: parseInt(result.trustedCount, 10) || 0,
      revokedCount: parseInt(result.revokedCount, 10) || 0,
      avgScore: parseFloat(result.avgScore) || 500,
    };
  }

  private recalculateScore(record: TrustRecordEntity): number {
    const baseScore = 500;
    const successBonus = record.verificationCount * 2;
    const failurePenalty = record.failureCount * 10;
    
    const score = baseScore + successBonus - failurePenalty;
    return Math.max(0, Math.min(1000, score));
  }
}
