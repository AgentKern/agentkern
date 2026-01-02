/**
 * AgentKernIdentity - Gate Module
 *
 * Security and policy enforcement module.
 * Now with TypeORM persistence for policies.
 */

import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { GateController } from '../controllers/gate.controller';
import { GateService } from '../services/gate.service';
import { GatePolicyEntity } from '../entities/gate-policy.entity';
import { GatePolicyRepository } from '../repositories/gate-policy.repository';

@Module({
  imports: [TypeOrmModule.forFeature([GatePolicyEntity])],
  controllers: [GateController],
  providers: [GateService, GatePolicyRepository],
  exports: [GateService, GatePolicyRepository],
})
export class GateModule {}
