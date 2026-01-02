/**
 * AgentKernIdentity - Database Module
 *
 * TypeORM configuration for PostgreSQL.
 * Provides connection and entity registration for the Identity pillar.
 */

import { Module } from '@nestjs/common';
import { TypeOrmModule, TypeOrmModuleOptions } from '@nestjs/typeorm';
import { ConfigModule, ConfigService } from '@nestjs/config';
import { TrustRecordEntity } from '../entities/trust-record.entity';
import {
  TrustEventEntity,
  TrustScoreEntity,
} from '../entities/trust-event.entity';
import { AuditEventEntity } from '../entities/audit-event.entity';
import { AgentRecordEntity } from '../entities/agent-record.entity';
import {
  WebAuthnCredentialEntity,
  WebAuthnChallengeEntity,
} from '../entities/webauthn-credential.entity';
import { VerificationKeyEntity } from '../entities/verification-key.entity';
import { SystemConfigEntity } from '../entities/system-config.entity';
import { NexusAgentEntity } from '../entities/nexus-agent.entity';
import { GatePolicyEntity } from '../entities/gate-policy.entity';

// All entities registered with the Identity database
const ENTITIES = [
  TrustRecordEntity,
  TrustEventEntity,
  TrustScoreEntity,
  AuditEventEntity,
  AgentRecordEntity,
  WebAuthnCredentialEntity,
  WebAuthnChallengeEntity,
  VerificationKeyEntity,
  SystemConfigEntity,
  NexusAgentEntity,
  GatePolicyEntity,
];

@Module({
  imports: [
    TypeOrmModule.forRootAsync({
      imports: [ConfigModule],
      inject: [ConfigService],
      useFactory: (configService: ConfigService): TypeOrmModuleOptions => {
        const databaseUrl = configService.get<string>('DATABASE_URL');

        if (databaseUrl) {
          const url = new URL(databaseUrl);
          return {
            type: 'postgres',
            host: url.hostname || 'localhost',
            port: parseInt(url.port, 10) || 5432,
            username: url.username || 'agentkern',
            password: decodeURIComponent(url.password) || 'agentkern_secret',
            database: url.pathname.slice(1) || 'agentkern_identity',
            entities: ENTITIES,
            synchronize: configService.get('DATABASE_SYNC', 'false') === 'true', // Default false for production safety
            dropSchema: configService.get('DATABASE_DROP_SCHEMA', 'false') === 'true', // For tests only
            logging: configService.get('DATABASE_LOGGING', 'false') === 'true',
            ssl:
              configService.get('DATABASE_SSL', 'false') === 'true'
                ? { rejectUnauthorized: false }
                : false,
          };
        }

        return {
          type: 'postgres',
          host: configService.get<string>('DATABASE_HOST', 'localhost'),
          port: configService.get<number>('DATABASE_PORT', 5432),
          username: configService.get<string>('DATABASE_USER', 'agentkern'),
          password: configService.get<string>(
            'DATABASE_PASSWORD',
            'agentkern_secret',
          ),
          database: configService.get<string>(
            'DATABASE_NAME',
            'agentkern_identity',
          ),
          entities: ENTITIES,
          synchronize: configService.get('DATABASE_SYNC', 'false') === 'true', // Default false for production safety
          dropSchema: configService.get('DATABASE_DROP_SCHEMA', 'false') === 'true', // For tests only
          logging: configService.get('DATABASE_LOGGING', 'false') === 'true',
          ssl:
            configService.get('DATABASE_SSL', 'false') === 'true'
              ? { rejectUnauthorized: false }
              : false,
        };
      },
    }),
    TypeOrmModule.forFeature(ENTITIES),
  ],
  exports: [TypeOrmModule],
})
export class DatabaseModule {}
