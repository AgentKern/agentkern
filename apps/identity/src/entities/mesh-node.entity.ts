import { Entity, Column, PrimaryColumn, CreateDateColumn, UpdateDateColumn } from 'typeorm';
import { MeshNodeType } from '../domain/mesh.entity';

@Entity('mesh_peers')
export class MeshPeerEntity {
  @PrimaryColumn()
  id: string;

  @Column()
  publicKey: string;

  @Column({
    type: 'enum',
    enum: MeshNodeType,
    default: MeshNodeType.FULL,
  })
  type: MeshNodeType;

  @Column('simple-array')
  endpoints: string[];

  @Column('simple-array')
  capabilities: string[];

  @Column()
  trustScore: number;

  @CreateDateColumn()
  connectedAt: Date;

  @UpdateDateColumn()
  lastSeen: Date;
}

@Entity('node_identity')
export class NodeIdentityEntity {
  @PrimaryColumn()
  id: string; // Internal ID, e.g., 'local'

  @Column()
  nodeId: string;

  @Column()
  publicKey: string;

  @Column()
  privateKey: string;

  @Column({
    type: 'enum',
    enum: MeshNodeType,
    default: MeshNodeType.FULL,
  })
  type: MeshNodeType;
}
