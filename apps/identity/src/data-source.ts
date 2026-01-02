/**
 * AgentKernIdentity - TypeORM Data Source Configuration
 *
 * For CLI migrations use only. App uses database.module.ts config.
 * Per TypeORM best practices: synchronize: false in production.
 *
 * Usage:
 *   npm run migration:generate --name=InitialSchema
 *   npm run migration:run
 *   npm run migration:revert
 */

import { DataSource, DataSourceOptions } from 'typeorm';
import { config } from 'dotenv';

// Load environment variables
config();

// Entity imports
import { TrustRecordEntity } from './entities/trust-record.entity';
import {
  TrustEventEntity,
  TrustScoreEntity,
} from './entities/trust-event.entity';
import { AuditEventEntity } from './entities/audit-event.entity';
import { AgentRecordEntity } from './entities/agent-record.entity';
import {
  WebAuthnCredentialEntity,
  WebAuthnChallengeEntity,
} from './entities/webauthn-credential.entity';
import { VerificationKeyEntity } from './entities/verification-key.entity';
import { SystemConfigEntity } from './entities/system-config.entity';
import { NexusAgentEntity } from './entities/nexus-agent.entity';
import { GatePolicyEntity } from './entities/gate-policy.entity';

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

function getDataSourceOptions(): DataSourceOptions {
  const databaseUrl = process.env.DATABASE_URL;

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
      migrations: ['src/migrations/*.ts'],
      synchronize: false, // NEVER true in production - use migrations
      migrationsRun: false, // Run via CLI, not on startup
      logging: process.env.NODE_ENV !== 'production',
    };
  }

  return {
    type: 'postgres',
    host: process.env.DATABASE_HOST || 'localhost',
    port: parseInt(process.env.DATABASE_PORT || '5432', 10),
    username: process.env.DATABASE_USER || 'agentkern',
    password: process.env.DATABASE_PASSWORD || 'agentkern_secret',
    database: process.env.DATABASE_NAME || 'agentkern_identity',
    entities: ENTITIES,
    migrations: ['src/migrations/*.ts'],
    synchronize: false, // NEVER true in production - use migrations
    migrationsRun: false, // Run via CLI, not on startup
    logging: process.env.NODE_ENV !== 'production',
  };
}

export const AppDataSource = new DataSource(getDataSourceOptions());
