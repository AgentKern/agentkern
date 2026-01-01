import { Module } from '@nestjs/common';
import { SynapseController } from '../controllers/synapse.controller';

@Module({
  controllers: [SynapseController],
  providers: [],
  exports: [],
})
export class SynapseModule {}
