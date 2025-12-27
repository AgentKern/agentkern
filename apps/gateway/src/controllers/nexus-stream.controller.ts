/**
 * AgentKern Gateway - SSE Streaming Controller
 * 
 * Server-Sent Events for real-time task updates.
 * Per A2A spec: SSE is the recommended transport for streaming.
 */

import { 
  Controller, 
  Get, 
  Post,
  Param, 
  Query,
  Res,
  Body,
  HttpStatus,
} from '@nestjs/common';
import { Response } from 'express';
import { NexusService } from '../services/nexus.service';

interface TaskEvent {
  type: 'status' | 'progress' | 'result' | 'error' | 'heartbeat';
  taskId: string;
  data: any;
  timestamp: string;
}

@Controller('nexus/stream')
export class NexusStreamController {
  private activeStreams = new Map<string, Set<Response>>();
  
  constructor(private readonly nexusService: NexusService) {}

  /**
   * Subscribe to task updates via SSE.
   * GET /nexus/stream/tasks/:taskId
   */
  @Get('tasks/:taskId')
  async streamTask(
    @Param('taskId') taskId: string,
    @Res() res: Response,
  ) {
    // Set SSE headers
    res.setHeader('Content-Type', 'text/event-stream');
    res.setHeader('Cache-Control', 'no-cache');
    res.setHeader('Connection', 'keep-alive');
    res.setHeader('X-Accel-Buffering', 'no');
    res.flushHeaders();

    // Register this stream
    if (!this.activeStreams.has(taskId)) {
      this.activeStreams.set(taskId, new Set());
    }
    this.activeStreams.get(taskId)!.add(res);

    // Send initial connection event
    this.sendEvent(res, {
      type: 'status',
      taskId,
      data: { status: 'connected' },
      timestamp: new Date().toISOString(),
    });

    // Heartbeat to keep connection alive
    const heartbeat = setInterval(() => {
      this.sendEvent(res, {
        type: 'heartbeat',
        taskId,
        data: {},
        timestamp: new Date().toISOString(),
      });
    }, 30000);

    // Cleanup on close
    res.on('close', () => {
      clearInterval(heartbeat);
      this.activeStreams.get(taskId)?.delete(res);
      if (this.activeStreams.get(taskId)?.size === 0) {
        this.activeStreams.delete(taskId);
      }
    });
  }

  /**
   * Subscribe to all agent events.
   * GET /nexus/stream/agents
   */
  @Get('agents')
  async streamAgents(@Res() res: Response) {
    res.setHeader('Content-Type', 'text/event-stream');
    res.setHeader('Cache-Control', 'no-cache');
    res.setHeader('Connection', 'keep-alive');
    res.flushHeaders();

    // Send current agents
    const agents = await this.nexusService.listAgents();
    this.sendEvent(res, {
      type: 'status',
      taskId: 'agents',
      data: { agents, count: agents.length },
      timestamp: new Date().toISOString(),
    });

    // Heartbeat
    const heartbeat = setInterval(() => {
      this.sendEvent(res, {
        type: 'heartbeat',
        taskId: 'agents',
        data: {},
        timestamp: new Date().toISOString(),
      });
    }, 30000);

    res.on('close', () => {
      clearInterval(heartbeat);
    });
  }

  /**
   * Publish task update (internal use).
   */
  async publishTaskUpdate(taskId: string, event: Partial<TaskEvent>) {
    const streams = this.activeStreams.get(taskId);
    if (!streams) return;

    const fullEvent: TaskEvent = {
      type: event.type || 'status',
      taskId,
      data: event.data || {},
      timestamp: new Date().toISOString(),
    };

    streams.forEach(res => {
      this.sendEvent(res, fullEvent);
    });
  }

  /**
   * Send SSE event.
   */
  private sendEvent(res: Response, event: TaskEvent) {
    res.write(`event: ${event.type}\n`);
    res.write(`data: ${JSON.stringify(event)}\n\n`);
  }

  /**
   * Get active stream count.
   * GET /nexus/stream/stats
   */
  @Get('stats')
  async getStreamStats() {
    const stats: Record<string, number> = {};
    this.activeStreams.forEach((streams, taskId) => {
      stats[taskId] = streams.size;
    });
    
    return {
      activeStreams: this.activeStreams.size,
      totalConnections: Array.from(this.activeStreams.values())
        .reduce((sum, set) => sum + set.size, 0),
      byTask: stats,
    };
  }
}
