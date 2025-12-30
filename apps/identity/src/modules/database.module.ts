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
import { TrustEventEntity, TrustScoreEntity } from '../entities/trust-event.entity';
import { AuditEventEntity } from '../entities/audit-event.entity';
import { AgentRecordEntity } from '../entities/agent-record.entity';
import { WebAuthnCredentialEntity, WebAuthnChallengeEntity } from '../entities/webauthn-credential.entity';
import { VerificationKeyEntity } from '../entities/verification-key.entity';
import { SystemConfigEntity } from '../entities/system-config.entity';

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
            username: url.username || 'agentkern-identity',
            password: decodeURIComponent(url.password) || 'agentkern-identity',
            database: url.pathname.slice(1) || 'agentkern-identity',
            entities: ENTITIES,
            synchronize: configService.get('DATABASE_SYNC', 'true') === 'true',
            logging: configService.get('DATABASE_LOGGING', 'false') === 'true',
            ssl: configService.get('DATABASE_SSL', 'false') === 'true'
              ? { rejectUnauthorized: false }
              : false,
          };
        }

        return {
          type: 'postgres',
          host: configService.get<string>('DATABASE_HOST', 'localhost'),
          port: configService.get<number>('DATABASE_PORT', 5432),
          username: configService.get<string>('DATABASE_USER', 'agentkern-identity'),
          password: configService.get<string>('DATABASE_PASSWORD', 'agentkern-identity'),
          database: configService.get<string>('DATABASE_NAME', 'agentkern-identity'),
          entities: ENTITIES,
          synchronize: configService.get('DATABASE_SYNC', 'true') === 'true',
          logging: configService.get('DATABASE_LOGGING', 'false') === 'true',
          ssl: configService.get('DATABASE_SSL', 'false') === 'true'
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
