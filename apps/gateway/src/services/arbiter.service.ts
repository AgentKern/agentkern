/**
 * AgentKern Gateway - Arbiter Service
 * 
 * Business logic for coordination and traffic control.
 * Per ARCHITECTURE: Implements Atomic Business Locks.
 */

import { Injectable, ConflictException } from '@nestjs/common';

interface BusinessLock {
  resource: string;
  lockedBy: string;
  acquiredAt: Date;
  expiresAt: Date;
  priority: number;
}

interface CoordinationRequest {
  agentId: string;
  resource: string;
  operation: 'read' | 'write' | 'exclusive';
  expectedDurationMs: number;
  priority: number;
}

interface CoordinationResult {
  granted: boolean;
  lock?: BusinessLock;
  queuePosition?: number;
  estimatedWaitMs?: number;
}

interface QueueEntry {
  agentId: string;
  resource: string;
  priority: number;
  requestedAt: Date;
}

@Injectable()
export class ArbiterService {
  // In-memory lock store (replace with Redis/distributed lock in production)
  private readonly locks = new Map<string, BusinessLock>();
  private readonly queue: QueueEntry[] = [];

  async requestCoordination(request: CoordinationRequest): Promise<CoordinationResult> {
    const existingLock = this.locks.get(request.resource);

    // Check if resource is locked
    if (existingLock && existingLock.expiresAt > new Date()) {
      // Resource is locked by someone else
      if (existingLock.lockedBy !== request.agentId) {
        // Add to queue
        this.addToQueue(request);
        const position = this.getQueuePosition(request.agentId, request.resource);
        
        return {
          granted: false,
          queuePosition: position,
          estimatedWaitMs: position * 5000, // Rough estimate
        };
      }
      // Already holds the lock
      return { granted: true, lock: existingLock };
    }

    // Grant the lock
    const lock = await this.acquireLock(request.agentId, request.resource, request.priority);
    if (lock) {
      return { granted: true, lock };
    }

    return { granted: false };
  }

  async acquireLock(agentId: string, resource: string, priority = 0): Promise<BusinessLock | null> {
    const existingLock = this.locks.get(resource);

    // Check if already locked by someone else
    if (existingLock && existingLock.expiresAt > new Date() && existingLock.lockedBy !== agentId) {
      // Check priority - higher priority can preempt
      if (priority > existingLock.priority) {
        // Preempt the existing lock
        console.log(`Agent ${agentId} preempted lock on ${resource} from ${existingLock.lockedBy}`);
      } else {
        return null;
      }
    }

    const lock: BusinessLock = {
      resource,
      lockedBy: agentId,
      acquiredAt: new Date(),
      expiresAt: new Date(Date.now() + 30000), // 30 second default TTL
      priority,
    };

    this.locks.set(resource, lock);
    this.removeFromQueue(agentId, resource);
    
    return lock;
  }

  async releaseLock(agentId: string, resource: string): Promise<{ released: boolean }> {
    const lock = this.locks.get(resource);

    if (!lock) {
      return { released: false };
    }

    if (lock.lockedBy !== agentId) {
      throw new ConflictException(`Lock on ${resource} is owned by ${lock.lockedBy}, not ${agentId}`);
    }

    this.locks.delete(resource);
    
    // Process queue - grant lock to next in line
    const nextInQueue = this.queue.find(q => q.resource === resource);
    if (nextInQueue) {
      await this.acquireLock(nextInQueue.agentId, nextInQueue.resource, nextInQueue.priority);
    }

    return { released: true };
  }

  async getLockStatus(resource: string): Promise<BusinessLock | null> {
    const lock = this.locks.get(resource);
    if (lock && lock.expiresAt > new Date()) {
      return lock;
    }
    return null;
  }

  async getQueuePosition(agentId: string, resource: string): Promise<number> {
    const index = this.queue.findIndex(
      q => q.agentId === agentId && q.resource === resource
    );
    return index === -1 ? 0 : index + 1;
  }

  private addToQueue(request: CoordinationRequest): void {
    const existing = this.queue.find(
      q => q.agentId === request.agentId && q.resource === request.resource
    );
    
    if (!existing) {
      this.queue.push({
        agentId: request.agentId,
        resource: request.resource,
        priority: request.priority,
        requestedAt: new Date(),
      });
      
      // Sort by priority (higher first)
      this.queue.sort((a, b) => b.priority - a.priority);
    }
  }

  private removeFromQueue(agentId: string, resource: string): void {
    const index = this.queue.findIndex(
      q => q.agentId === agentId && q.resource === resource
    );
    if (index !== -1) {
      this.queue.splice(index, 1);
    }
  }
}
