import { Module } from '@nestjs/common';
import { GateController } from '../controllers/gate.controller';
import { GateService } from '../services/gate.service';

@Module({
  controllers: [GateController],
  providers: [GateService],
  exports: [GateService],
})
export class GateModule {}
