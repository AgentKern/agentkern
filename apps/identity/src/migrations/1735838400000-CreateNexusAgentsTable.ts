import { MigrationInterface, QueryRunner } from 'typeorm';

/**
 * Migration: Create nexus_agents table
 *
 * Part of the Nexus Agent Registry persistence feature.
 * Stores agent cards for the mesh protocol gateway.
 */
export class CreateNexusAgentsTable1735838400000 implements MigrationInterface {
  name = 'CreateNexusAgentsTable1735838400000';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`
      CREATE TABLE "nexus_agents" (
        "id" uuid NOT NULL,
        "name" character varying NOT NULL,
        "description" text NOT NULL DEFAULT '',
        "url" character varying NOT NULL,
        "version" character varying NOT NULL DEFAULT '1.0.0',
        "capabilities" jsonb NOT NULL DEFAULT '[]',
        "skills" jsonb NOT NULL DEFAULT '[]',
        "protocols" text NOT NULL DEFAULT '',
        "registeredAt" TIMESTAMP NOT NULL DEFAULT now(),
        "updatedAt" TIMESTAMP NOT NULL DEFAULT now(),
        "active" boolean NOT NULL DEFAULT true,
        "discoveredFrom" character varying,
        CONSTRAINT "PK_nexus_agents" PRIMARY KEY ("id")
      )
    `);

    // Create index for active agents query
    await queryRunner.query(`
      CREATE INDEX "IDX_nexus_agents_active" ON "nexus_agents" ("active")
    `);

    // Create GIN index for JSONB skills search
    await queryRunner.query(`
      CREATE INDEX "IDX_nexus_agents_skills" ON "nexus_agents" USING GIN ("skills")
    `);

    // Create index for name search
    await queryRunner.query(`
      CREATE INDEX "IDX_nexus_agents_name" ON "nexus_agents" ("name")
    `);
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`DROP INDEX "IDX_nexus_agents_name"`);
    await queryRunner.query(`DROP INDEX "IDX_nexus_agents_skills"`);
    await queryRunner.query(`DROP INDEX "IDX_nexus_agents_active"`);
    await queryRunner.query(`DROP TABLE "nexus_agents"`);
  }
}
