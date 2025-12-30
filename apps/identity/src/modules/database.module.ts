/**
 * AgentKernIdentity - Database Module
 * 
 * TypeORM configuration for PostgreSQL.
 */

import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ConfigModule, ConfigService } from '@nestjs/config';
import { TrustRecordEntity } from '../entities/trust-record.entity';
import { AuditEventEntity } from '../entities/audit-event.entity';
import { PolicyEntity } from '../entities/policy.entity';
import { AgentRecordEntity } from '../entities/agent-record.entity';
import { MeshPeerEntity, NodeIdentityEntity } from '../entities/mesh-node.entity';

@Module({
  imports: [
    TypeOrmModule.forRootAsync({
      imports: [ConfigModule],
      inject: [ConfigService],
      useFactory: (configService: ConfigService) => {
        // Support DATABASE_URL (for CI/Docker) or individual vars
        const databaseUrl = configService.get('DATABASE_URL');
        
        if (databaseUrl) {
          // Parse DATABASE_URL: postgresql://user:password@host:port/database
          const url = new URL(databaseUrl);
          return {
            type: 'postgres' as const,
            host: url.hostname,
            port: parseInt(url.port) || 5432,
            username: url.username,
            password: url.password,
            database: url.pathname.slice(1), // Remove leading /
            entities: [
              TrustRecordEntity,
              AuditEventEntity,
              PolicyEntity,
              AgentRecordEntity,
              MeshPeerEntity,
              NodeIdentityEntity,
            ],
            synchronize: configService.get('DATABASE_SYNC', 'true') === 'true',
            logging: configService.get('DATABASE_LOGGING', 'false') === 'true',
            ssl: configService.get('DATABASE_SSL', 'false') === 'true'
              ? { rejectUnauthorized: false }
              : false,
          };
        }
        
        // Fallback to individual environment variables
        return {
          type: 'postgres' as const,
          host: configService.get('DATABASE_HOST', 'localhost'),
          port: configService.get('DATABASE_PORT', 5432),
          username: configService.get('DATABASE_USER', 'agentkern-identity'),
          password: configService.get('DATABASE_PASSWORD', 'agentkern-identity'),
          database: configService.get('DATABASE_NAME', 'agentkern-identity'),
          entities: [
            TrustRecordEntity,
            AuditEventEntity,
            PolicyEntity,
            AgentRecordEntity,
            MeshPeerEntity,
            NodeIdentityEntity,
          ],
          synchronize: configService.get('DATABASE_SYNC', 'true') === 'true',
          logging: configService.get('DATABASE_LOGGING', 'false') === 'true',
          ssl: configService.get('DATABASE_SSL', 'false') === 'true'
            ? { rejectUnauthorized: false }
            : false,
        };
      },
    }),
    TypeOrmModule.forFeature([
      TrustRecordEntity,
      AuditEventEntity,
      PolicyEntity,
      AgentRecordEntity,
      MeshPeerEntity,
      NodeIdentityEntity,
    ]),
  ],
  exports: [TypeOrmModule],
})
export class DatabaseModule {}
