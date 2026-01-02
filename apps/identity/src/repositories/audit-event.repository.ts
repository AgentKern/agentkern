/**
 * AgentKernIdentity - Audit Event Repository
 *
 * PostgreSQL repository for Audit Events.
 */

import { Injectable, Logger } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import {
  Repository,
  Between,
  In,
  FindOperator,
  FindOptionsWhere,
} from 'typeorm';
import {
  AuditEventEntity,
  AuditEventTypeEnum,
} from '../entities/audit-event.entity';

@Injectable()
export class AuditEventRepository {
  private readonly logger = new Logger(AuditEventRepository.name);

  constructor(
    @InjectRepository(AuditEventEntity)
    private readonly repository: Repository<AuditEventEntity>,
  ) {}

  /**
   * Log an audit event
   */
  async log(data: Partial<AuditEventEntity>): Promise<AuditEventEntity> {
    const event = this.repository.create(data);
    return this.repository.save(event);
  }

  /**
   * Get recent events
   */
  async getRecentEvents(limit = 100): Promise<AuditEventEntity[]> {
    return this.repository.find({
      order: { timestamp: 'DESC' },
      take: limit,
    });
  }

  /**
   * Get events for a principal
   */
  async getByPrincipal(
    principalId: string,
    limit = 100,
  ): Promise<AuditEventEntity[]> {
    return this.repository.find({
      where: { principalId },
      order: { timestamp: 'DESC' },
      take: limit,
    });
  }

  /**
   * Get events for an agent
   */
  async getByAgent(agentId: string, limit = 100): Promise<AuditEventEntity[]> {
    return this.repository.find({
      where: { agentId },
      order: { timestamp: 'DESC' },
      take: limit,
    });
  }

  /**
   * Get events by date range
   */
  async getByDateRange(
    startDate: Date,
    endDate: Date,
    types?: AuditEventTypeEnum[],
  ): Promise<AuditEventEntity[]> {
    const where: FindOptionsWhere<AuditEventEntity> = {
      timestamp: Between(startDate, endDate),
    };

    if (types && types.length > 0) {
      where.type = In(types) as FindOperator<AuditEventTypeEnum>;
    }

    return this.repository.find({
      where,
      order: { timestamp: 'DESC' },
    });
  }

  /**
   * Get security events
   */
  async getSecurityEvents(
    since?: Date,
    limit = 100,
  ): Promise<AuditEventEntity[]> {
    const securityTypes = [
      AuditEventTypeEnum.RATE_LIMIT_EXCEEDED,
      AuditEventTypeEnum.INVALID_INPUT,
      AuditEventTypeEnum.SUSPICIOUS_ACTIVITY,
      AuditEventTypeEnum.PROOF_INVALID_SIGNATURE,
      AuditEventTypeEnum.SECURITY_ALERT,
    ];

    const where: FindOptionsWhere<AuditEventEntity> = {
      type: In(securityTypes) as FindOperator<AuditEventTypeEnum>,
    };

    if (since) {
      where.timestamp = Between(since, new Date());
    }

    return this.repository.find({
      where,
      order: { timestamp: 'DESC' },
      take: limit,
    });
  }

  /**
   * Get compliance report data
   */
  async getComplianceData(
    startDate: Date,
    endDate: Date,
    agentId?: string,
    principalId?: string,
  ): Promise<{
    totalVerifications: number;
    successfulVerifications: number;
    failedVerifications: number;
    revocations: number;
    securityAlerts: number;
  }> {
    const qb = this.repository
      .createQueryBuilder('ae')
      .where('ae.timestamp BETWEEN :startDate AND :endDate', {
        startDate,
        endDate,
      });

    if (agentId) {
      qb.andWhere('ae.agentId = :agentId', { agentId });
    }

    if (principalId) {
      qb.andWhere('ae.principalId = :principalId', { principalId });
    }

    interface ComplianceResult {
      totalVerifications: string;
      successfulVerifications: string;
      failedVerifications: string;
      revocations: string;
      securityAlerts: string;
    }

    const result = await qb
      .select([
        `SUM(CASE WHEN type IN ('${AuditEventTypeEnum.PROOF_VERIFICATION_SUCCESS}', '${AuditEventTypeEnum.PROOF_VERIFICATION_FAILURE}') THEN 1 ELSE 0 END) as totalVerifications`,
        `SUM(CASE WHEN type = '${AuditEventTypeEnum.PROOF_VERIFICATION_SUCCESS}' THEN 1 ELSE 0 END) as successfulVerifications`,
        `SUM(CASE WHEN type = '${AuditEventTypeEnum.PROOF_VERIFICATION_FAILURE}' THEN 1 ELSE 0 END) as failedVerifications`,
        `SUM(CASE WHEN type = '${AuditEventTypeEnum.KEY_REVOKED}' THEN 1 ELSE 0 END) as revocations`,
        `SUM(CASE WHEN type = '${AuditEventTypeEnum.SECURITY_ALERT}' THEN 1 ELSE 0 END) as securityAlerts`,
      ])
      .getRawOne<ComplianceResult>();

    return {
      totalVerifications: parseInt(result?.totalVerifications ?? '0', 10),
      successfulVerifications: parseInt(
        result?.successfulVerifications ?? '0',
        10,
      ),
      failedVerifications: parseInt(result?.failedVerifications ?? '0', 10),
      revocations: parseInt(result?.revocations ?? '0', 10),
      securityAlerts: parseInt(result?.securityAlerts ?? '0', 10),
    };
  }
}
