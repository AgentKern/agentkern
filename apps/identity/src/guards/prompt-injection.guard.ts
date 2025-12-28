/**
 * AgentKern Identity - Prompt Injection Guard
 *
 * Multi-layer defense against prompt injection attacks from AI agents.
 * Scans all incoming requests for known injection patterns and suspicious content.
 *
 * Security Features:
 * - Pattern-based detection of known injection techniques
 * - Entropy analysis for obfuscated payloads
 * - Nested JSON/base64 payload inspection
 * - Audit logging of all blocked attempts
 */

import {
  Injectable,
  CanActivate,
  ExecutionContext,
  ForbiddenException,
  Logger,
} from '@nestjs/common';
import { Request } from 'express';
import { AuditLoggerService, AuditEventType } from '../services/audit-logger.service';

export interface InjectionDetectionResult {
  detected: boolean;
  pattern?: string;
  location?: string;
  severity?: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
  confidence?: number;
}

@Injectable()
export class PromptInjectionGuard implements CanActivate {
  private readonly logger = new Logger(PromptInjectionGuard.name);

  // Known prompt injection patterns - regularly update this list
  private readonly INJECTION_PATTERNS: Array<{
    pattern: RegExp;
    name: string;
    severity: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
  }> = [
    // Direct instruction override attempts
    {
      pattern: /ignore\s+(all\s+)?(previous|prior|above)\s+(instructions?|prompts?|rules?)/i,
      name: 'IGNORE_PREVIOUS',
      severity: 'CRITICAL',
    },
    {
      pattern: /disregard\s+(all\s+)?(previous|prior|above)/i,
      name: 'DISREGARD_PREVIOUS',
      severity: 'CRITICAL',
    },
    {
      pattern: /forget\s+(everything|all|what)\s+(you|i)\s+(said|told|know)/i,
      name: 'FORGET_INSTRUCTIONS',
      severity: 'CRITICAL',
    },

    // System prompt extraction
    {
      pattern: /what\s+(is|are)\s+your\s+(system\s+)?prompt/i,
      name: 'SYSTEM_PROMPT_EXTRACTION',
      severity: 'HIGH',
    },
    {
      pattern: /show\s+me\s+(your\s+)?(system\s+)?instructions/i,
      name: 'INSTRUCTION_EXTRACTION',
      severity: 'HIGH',
    },
    {
      pattern: /reveal\s+(your\s+)?(hidden|secret|internal)/i,
      name: 'SECRET_EXTRACTION',
      severity: 'HIGH',
    },

    // Role manipulation
    {
      pattern: /you\s+are\s+now\s+(a|an|the)\s+/i,
      name: 'ROLE_OVERRIDE',
      severity: 'CRITICAL',
    },
    {
      pattern: /pretend\s+(you\s+are|to\s+be)\s+/i,
      name: 'PRETEND_ROLE',
      severity: 'HIGH',
    },
    {
      pattern: /act\s+as\s+(if\s+you\s+are\s+)?(a|an|the)\s+/i,
      name: 'ACT_AS_ROLE',
      severity: 'MEDIUM',
    },
    {
      pattern: /roleplay\s+as\s+/i,
      name: 'ROLEPLAY',
      severity: 'MEDIUM',
    },

    // Jailbreak attempts
    {
      pattern: /\bDAN\b.*\bdo\s+anything\s+now\b/i,
      name: 'DAN_JAILBREAK',
      severity: 'CRITICAL',
    },
    {
      pattern: /developer\s+mode\s+(enabled|activated|on)/i,
      name: 'DEVELOPER_MODE',
      severity: 'CRITICAL',
    },
    {
      pattern: /maintenance\s+mode/i,
      name: 'MAINTENANCE_MODE',
      severity: 'HIGH',
    },

    // Boundary escape attempts
    {
      pattern: /\]\s*\}\s*\)\s*;\s*\/\//,
      name: 'CODE_INJECTION',
      severity: 'CRITICAL',
    },
    {
      pattern: /<\/?script/i,
      name: 'SCRIPT_INJECTION',
      severity: 'CRITICAL',
    },
    {
      pattern: /{{.*}}/,
      name: 'TEMPLATE_INJECTION',
      severity: 'HIGH',
    },

    // Social engineering
    {
      pattern: /this\s+is\s+(an?\s+)?(urgent|emergency|critical)\s+(test|situation)/i,
      name: 'URGENCY_MANIPULATION',
      severity: 'MEDIUM',
    },
    {
      pattern: /admin(istrator)?\s+(override|access|mode)/i,
      name: 'ADMIN_OVERRIDE',
      severity: 'CRITICAL',
    },

    // Data exfiltration
    {
      pattern: /send\s+(this\s+)?to\s+(my\s+)?email/i,
      name: 'DATA_EXFIL_EMAIL',
      severity: 'HIGH',
    },
    {
      pattern: /upload\s+to\s+(http|ftp|s3)/i,
      name: 'DATA_EXFIL_UPLOAD',
      severity: 'CRITICAL',
    },

    // Instruction smuggling
    {
      pattern: /\[INST\]|\[\/INST\]/i,
      name: 'INSTRUCTION_TAGS',
      severity: 'HIGH',
    },
    {
      pattern: /<\|im_start\|>|<\|im_end\|>/i,
      name: 'CHAT_TEMPLATE_TAGS',
      severity: 'HIGH',
    },
    {
      pattern: /###\s*(Human|Assistant|System):/i,
      name: 'ROLE_MARKERS',
      severity: 'MEDIUM',
    },
  ];

  // Suspicious character sequences that might indicate obfuscation
  private readonly OBFUSCATION_PATTERNS = [
    /[\u200b-\u200f\u2028-\u202f\u2060-\u206f]/g, // Zero-width characters
    /[\u0300-\u036f]{3,}/g, // Excessive combining characters
    /(.)\1{10,}/g, // Repeated characters
  ];

  constructor(private readonly auditLogger: AuditLoggerService) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest<Request>();
    
    // Collect all text content to scan
    const contentToScan = this.extractContent(request);
    
    // Run detection
    const results = this.detectInjection(contentToScan);
    
    if (results.detected) {
      // Log the security event
      this.auditLogger.logSecurityEvent(
        AuditEventType.SUSPICIOUS_ACTIVITY,
        `Prompt injection attempt detected: ${results.pattern}`,
        {
          location: results.location,
          severity: results.severity,
          confidence: results.confidence,
          ip: request.ip,
          path: request.path,
          method: request.method,
        },
        {
          ipAddress: request.ip,
          userAgent: request.headers['user-agent'],
        },
      );

      this.logger.warn(
        `🚨 Prompt injection blocked: ${results.pattern} (${results.severity}) from ${request.ip}`,
      );

      throw new ForbiddenException({
        error: 'Security violation detected',
        message: 'Request contains potentially malicious content',
        code: 'PROMPT_INJECTION_DETECTED',
      });
    }

    return true;
  }

  /**
   * Extract all text content from the request for scanning
   */
  private extractContent(request: Request): Map<string, string> {
    const content = new Map<string, string>();

    // Scan request body
    if (request.body && typeof request.body === 'object') {
      content.set('body', JSON.stringify(request.body));
      
      // Deep scan nested strings
      this.extractNestedStrings(request.body, 'body', content);
    }

    // Scan query parameters
    if (request.query) {
      content.set('query', JSON.stringify(request.query));
    }

    // Scan path parameters
    if (request.params) {
      content.set('params', JSON.stringify(request.params));
    }

    // Scan specific headers that might carry agent payloads
    const sensitiveHeaders = ['x-agentkern-identity', 'authorization', 'x-agent-id'];
    for (const header of sensitiveHeaders) {
      const value = request.headers[header];
      if (value && typeof value === 'string') {
        content.set(`header:${header}`, value);
      }
    }

    return content;
  }

  /**
   * Recursively extract string values from nested objects
   */
  private extractNestedStrings(
    obj: unknown,
    path: string,
    content: Map<string, string>,
  ): void {
    if (typeof obj === 'string') {
      // Check if it's base64 encoded and decode
      if (this.isBase64(obj) && obj.length > 20) {
        try {
          const decoded = Buffer.from(obj, 'base64').toString('utf8');
          content.set(`${path}:decoded`, decoded);
        } catch {
          // Not valid base64, ignore
        }
      }
      content.set(path, obj);
    } else if (Array.isArray(obj)) {
      obj.forEach((item, index) => {
        this.extractNestedStrings(item, `${path}[${index}]`, content);
      });
    } else if (obj && typeof obj === 'object') {
      for (const [key, value] of Object.entries(obj)) {
        this.extractNestedStrings(value, `${path}.${key}`, content);
      }
    }
  }

  /**
   * Check if string appears to be base64 encoded
   */
  private isBase64(str: string): boolean {
    if (str.length < 4 || str.length % 4 !== 0) return false;
    return /^[A-Za-z0-9+/=]+$/.test(str);
  }

  /**
   * Run injection detection on all content
   */
  private detectInjection(content: Map<string, string>): InjectionDetectionResult {
    for (const [location, text] of content) {
      // Check for obfuscation first
      const obfuscationResult = this.checkObfuscation(text, location);
      if (obfuscationResult.detected) {
        return obfuscationResult;
      }

      // Check against known patterns
      for (const { pattern, name, severity } of this.INJECTION_PATTERNS) {
        if (pattern.test(text)) {
          return {
            detected: true,
            pattern: name,
            location,
            severity,
            confidence: 0.9,
          };
        }
      }
    }

    return { detected: false };
  }

  /**
   * Check for obfuscation attempts
   */
  private checkObfuscation(text: string, location: string): InjectionDetectionResult {
    for (const pattern of this.OBFUSCATION_PATTERNS) {
      const matches = text.match(pattern);
      if (matches && matches.length > 0) {
        return {
          detected: true,
          pattern: 'OBFUSCATION_DETECTED',
          location,
          severity: 'HIGH',
          confidence: 0.7,
        };
      }
    }

    // Check for suspicious entropy (random-looking strings that might be encoded payloads)
    if (this.hasHighEntropy(text) && text.length > 100) {
      return {
        detected: true,
        pattern: 'HIGH_ENTROPY_PAYLOAD',
        location,
        severity: 'MEDIUM',
        confidence: 0.5,
      };
    }

    return { detected: false };
  }

  /**
   * Calculate Shannon entropy to detect potentially obfuscated payloads
   */
  private hasHighEntropy(text: string): boolean {
    if (text.length < 50) return false;

    const freq = new Map<string, number>();
    for (const char of text) {
      freq.set(char, (freq.get(char) || 0) + 1);
    }

    let entropy = 0;
    for (const count of freq.values()) {
      const p = count / text.length;
      entropy -= p * Math.log2(p);
    }

    // High entropy threshold (natural text is usually 3-5 bits/char)
    return entropy > 5.5;
  }
}

/**
 * Decorator to apply prompt injection protection to specific routes
 */
export function ProtectFromInjection() {
  return (
    target: object,
    propertyKey: string,
    descriptor: PropertyDescriptor,
  ) => {
    // Can be extended to add route-specific injection rules
    return descriptor;
  };
}
