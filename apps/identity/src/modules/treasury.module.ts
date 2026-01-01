import { Module } from '@nestjs/common';
import { TreasuryController } from '../controllers/treasury.controller';

@Module({
  controllers: [TreasuryController],
  providers: [],
  exports: [],
})
export class TreasuryModule {}
