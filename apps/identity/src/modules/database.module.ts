/**
 * AgentKern Identity - Database Module
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
      useFactory: (configService: ConfigService) => ({
        type: 'postgres',
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
        synchronize: configService.get('DATABASE_SYNC', 'true') === 'true', // Disable in production
        logging: configService.get('DATABASE_LOGGING', 'false') === 'true',
        ssl: configService.get('DATABASE_SSL', 'false') === 'true'
          ? { rejectUnauthorized: false }
          : false,
      }),
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
