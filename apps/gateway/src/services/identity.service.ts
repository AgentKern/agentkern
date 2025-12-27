/**
 * AgentKern Gateway - Identity Service
 * 
 * Business logic for agent identity operations.
 * Per MANDATE: Every agent request is signed.
 */

import { Injectable, NotFoundException } from '@nestjs/common';
import { randomUUID, createHash, createSign, generateKeyPairSync } from 'crypto';

interface AgentRecord {
  id: string;
  name: string;
  publicKey: string;
  privateKey: string;
  capabilities: string[];
  createdAt: Date;
  trustScore: number;
}

@Injectable()
export class IdentityService {
  // In-memory store (replace with database in production)
  private readonly agents = new Map<string, AgentRecord>();

  async register(name: string, capabilities: string[] = []): Promise<Omit<AgentRecord, 'privateKey'>> {
    const id = randomUUID();
    
    // Generate key pair for signing
    const { publicKey, privateKey } = generateKeyPairSync('ed25519', {
      publicKeyEncoding: { type: 'spki', format: 'pem' },
      privateKeyEncoding: { type: 'pkcs8', format: 'pem' },
    });

    const agent: AgentRecord = {
      id,
      name,
      publicKey,
      privateKey,
      capabilities,
      createdAt: new Date(),
      trustScore: 100, // Start with perfect score
    };

    this.agents.set(id, agent);

    // Return public info only
    const { privateKey: _, ...publicAgent } = agent;
    return publicAgent;
  }

  async getIdentity(id: string): Promise<Omit<AgentRecord, 'privateKey'> | null> {
    const agent = this.agents.get(id);
    if (!agent) return null;
    
    const { privateKey: _, ...publicAgent } = agent;
    return publicAgent;
  }

  async signAction(agentId: string, action: string, payload: unknown) {
    const agent = this.agents.get(agentId);
    if (!agent) {
      throw new NotFoundException(`Agent ${agentId} not found`);
    }

    const timestamp = new Date();
    const payloadHash = createHash('sha256')
      .update(JSON.stringify(payload))
      .digest('hex');

    const dataToSign = `${agentId}:${action}:${timestamp.toISOString()}:${payloadHash}`;
    
    const sign = createSign('ed25519');
    sign.update(dataToSign);
    const signature = sign.sign(agent.privateKey, 'base64');

    return {
      action,
      agentId,
      timestamp,
      signature,
      payloadHash,
    };
  }

  async verifyProof(proof: {
    action: string;
    agentId: string;
    timestamp: string;
    signature: string;
    payloadHash: string;
  }): Promise<{ valid: boolean; reason?: string }> {
    const agent = this.agents.get(proof.agentId);
    if (!agent) {
      return { valid: false, reason: 'Agent not found' };
    }

    try {
      const { createVerify } = await import('crypto');
      const dataToVerify = `${proof.agentId}:${proof.action}:${proof.timestamp}:${proof.payloadHash}`;
      
      const verify = createVerify('ed25519');
      verify.update(dataToVerify);
      const isValid = verify.verify(agent.publicKey, proof.signature, 'base64');

      return { valid: isValid };
    } catch (error) {
      return { valid: false, reason: 'Signature verification failed' };
    }
  }

  async getTrustScore(agentId: string): Promise<number> {
    const agent = this.agents.get(agentId);
    if (!agent) {
      throw new NotFoundException(`Agent ${agentId} not found`);
    }
    return agent.trustScore;
  }
}
