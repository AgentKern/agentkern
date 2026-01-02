import { MigrationInterface, QueryRunner } from 'typeorm';

/**
 * Migration: Create gate_policies table
 *
 * Part of the Gate Policy Engine persistence feature.
 * Stores security policies for prompt filtering and access control.
 */
export class CreateGatePoliciesTable1735838500000 implements MigrationInterface {
  name = 'CreateGatePoliciesTable1735838500000';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`
      CREATE TABLE "gate_policies" (
        "id" uuid NOT NULL,
        "name" character varying NOT NULL,
        "description" text,
        "active" boolean NOT NULL DEFAULT true,
        "rules" jsonb NOT NULL DEFAULT '[]',
        "tags" text NOT NULL DEFAULT '',
        "createdBy" character varying,
        "createdAt" TIMESTAMP NOT NULL DEFAULT now(),
        "updatedAt" TIMESTAMP NOT NULL DEFAULT now(),
        "version" integer NOT NULL DEFAULT 0,
        CONSTRAINT "PK_gate_policies" PRIMARY KEY ("id")
      )
    `);

    // Create index for active policies query
    await queryRunner.query(`
      CREATE INDEX "IDX_gate_policies_active" ON "gate_policies" ("active")
    `);

    // Create index for policy name search
    await queryRunner.query(`
      CREATE INDEX "IDX_gate_policies_name" ON "gate_policies" ("name")
    `);

    // Create GIN index for JSONB rules search
    await queryRunner.query(`
      CREATE INDEX "IDX_gate_policies_rules" ON "gate_policies" USING GIN ("rules")
    `);
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`DROP INDEX "IDX_gate_policies_rules"`);
    await queryRunner.query(`DROP INDEX "IDX_gate_policies_name"`);
    await queryRunner.query(`DROP INDEX "IDX_gate_policies_active"`);
    await queryRunner.query(`DROP TABLE "gate_policies"`);
  }
}
