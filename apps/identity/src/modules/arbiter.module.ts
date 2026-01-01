import { Module } from '@nestjs/common';
import { ArbiterController } from '../controllers/arbiter.controller';

@Module({
  controllers: [ArbiterController],
  providers: [],
  exports: [],
})
export class ArbiterModule {}
