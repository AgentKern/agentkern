import { Module } from '@nestjs/common';
import { SynapseController } from '../controllers/synapse.controller';
import { SynapseService } from '../services/synapse.service';

@Module({
  controllers: [SynapseController],
  providers: [SynapseService],
  exports: [SynapseService],
})
export class SynapseModule {}
