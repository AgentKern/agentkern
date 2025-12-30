/**
 * AgentKernIdentity - Database Module
 *
 * TypeORM configuration for PostgreSQL.
 * Provides connection and entity registration for the Identity pillar.
 */

import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ConfigModule, ConfigService } from '@nestjs/config';
import { TrustRecordEntity } from '../entities/trust-record.entity';
import { AuditEventEntity } from '../entities/audit-event.entity';
import { AgentRecordEntity } from '../entities/agent-record.entity';
import { WebAuthnCredentialEntity, WebAuthnChallengeEntity } from '../entities/webauthn-credential.entity';

// All entities registered with the Identity database
const ENTITIES = [
  TrustRecordEntity,
  AuditEventEntity,
  AgentRecordEntity,
  WebAuthnCredentialEntity,
  WebAuthnChallengeEntity,
];

@Module({
  imports: [
    TypeOrmModule.forRootAsync({
      imports: [ConfigModule],
      inject: [ConfigService],
      useFactory: (configService: ConfigService) => {
        const databaseUrl = configService.get('DATABASE_URL');

        if (databaseUrl) {
          const url = new URL(databaseUrl);
          return {
            type: 'postgres' as const,
            host: url.hostname,
            port: parseInt(url.port) || 5432,
            username: url.username,
            password: url.password,
            database: String(url.pathname.slice(1)),
            entities: ENTITIES,
            synchronize: configService.get('DATABASE_SYNC', 'true') === 'true',
            logging: configService.get('DATABASE_LOGGING', 'false') === 'true',
            ssl: configService.get('DATABASE_SSL', 'false') === 'true'
              ? { rejectUnauthorized: false }
              : false,
          };
        }

        return {
          type: 'postgres' as const,
          host: configService.get('DATABASE_HOST', 'localhost'),
          port: configService.get('DATABASE_PORT', 5432),
          username: configService.get('DATABASE_USER', 'agentkern-identity'),
          password: configService.get('DATABASE_PASSWORD', 'agentkern-identity'),
          database: configService.get('DATABASE_NAME', 'agentkern-identity'),
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
