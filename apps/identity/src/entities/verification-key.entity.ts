/**
 * AgentKernIdentity - Verification Key Entity
 *
 * Persistent storage for public keys used to verify Liability Proofs.
 * Keys are registered by principals and used to verify agent signatures.
 */

import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
  Index,
  Unique,
} from 'typeorm';

@Entity('verification_keys')
@Unique(['principalId', 'credentialId'])
@Index(['principalId'])
export class VerificationKeyEntity {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column()
  @Index()
  principalId: string;

  @Column()
  credentialId: string;

  /**
   * Public key in PEM (SPKI) or JWK format
   */
  @Column('text')
  publicKey: string;

  /**
   * Algorithm used for signing (e.g., ES256, EdDSA)
   */
  @Column({ default: 'ES256' })
  algorithm: string;

  /**
   * Key format: 'pem' or 'jwk'
   */
  @Column({ default: 'pem' })
  format: 'pem' | 'jwk';

  /**
   * Whether this key is currently active
   */
  @Column({ default: true })
  active: boolean;

  /**
   * Optional key expiration
   */
  @Column({ type: 'timestamp', nullable: true })
  expiresAt: Date | null;

  /**
   * Last time this key was used for verification
   */
  @Column({ type: 'timestamp', nullable: true })
  lastUsedAt: Date | null;

  /**
   * Number of times this key has been used
   */
  @Column({ type: 'int', default: 0 })
  usageCount: number;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;
}
