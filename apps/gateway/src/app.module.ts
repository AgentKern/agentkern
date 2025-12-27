/**
 * AgentKern Gateway - App Module
 * 
 * Root module that imports all Six Pillars modules.
 * Per MANDATE: Hexagonal Architecture - Core logic isolated from I/O.
 */

import { Module } from '@nestjs/common';
import { IdentityModule } from './modules/identity.module';
import { GateModule } from './modules/gate.module';
import { SynapseModule } from './modules/synapse.module';
import { ArbiterModule } from './modules/arbiter.module';
import { NexusModule } from './modules/nexus.module';
import { HealthController } from './controllers/health.controller';

@Module({
  imports: [
    // The Six Pillars
    IdentityModule,
    GateModule,
    SynapseModule,
    ArbiterModule,
    NexusModule,       // Protocol Translation (NEW)
    // TreasuryModule, // Enterprise only (ee/treasury)
  ],
  controllers: [HealthController],
})
export class AppModule {}

