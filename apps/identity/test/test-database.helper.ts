/**
 * Test Database Configuration Helper
 * 
 * Provides SQLite in-memory database configuration for E2E tests.
 */

import { TypeOrmModule } from '@nestjs/typeorm';
import { TrustRecordEntity } from '../src/entities/trust-record.entity';
import { AuditEventEntity } from '../src/entities/audit-event.entity';
import { PolicyEntity } from '../src/entities/policy.entity';
import { AgentRecordEntity } from '../src/entities/agent-record.entity';
import { MeshPeerEntity, NodeIdentityEntity } from '../src/entities/mesh-node.entity';

/**
 * All entities used in the application
 */
export const ALL_ENTITIES = [
  TrustRecordEntity,
  AuditEventEntity,
  PolicyEntity,
  AgentRecordEntity,
  MeshPeerEntity,
  NodeIdentityEntity,
];

/**
 * Creates a TypeORM module configured for in-memory SQLite testing
 */
export const TestDatabaseModule = TypeOrmModule.forRoot({
  type: 'sqlite',
  database: ':memory:',
  entities: ALL_ENTITIES,
  synchronize: true,
  dropSchema: true,
  logging: false,
});

/**
 * Creates TypeORM feature module for all entities
 */
export const TestEntitiesModule = TypeOrmModule.forFeature(ALL_ENTITIES);
