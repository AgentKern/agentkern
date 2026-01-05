import { Module } from '@nestjs/common';
import { SynapseController } from '../controllers/synapse.controller';
import { SynapseService } from '../services/synapse.service';
import { GateModule } from './gate.module';

@Module({
  imports: [GateModule],
  controllers: [SynapseController],
  providers: [SynapseService],
  exports: [SynapseService],
})
export class SynapseModule {}

