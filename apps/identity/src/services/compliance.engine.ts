/**
 * AgentKern Identity - Compliance Engine (2026 Standard)
 *
 * Implements strict compliance checks for:
 * - PCI-DSS v4.0 (Payment Card Industry Data Security Standard)
 * - HIPAA (Health Insurance Portability and Accountability Act - 2025 Rule)
 * - GDPR (General Data Protection Regulation) & EU AI Act
 *
 * Uses advanced pattern matching and validation algorithms (Luhn).
 */

import { Logger } from '@nestjs/common';

export interface ComplianceIssue {
  code: string;
  severity: 'info' | 'warning' | 'error' | 'critical';
  message: string;
  path?: string;
  remediation?: string;
}

export interface ComplianceReport {
  compliant: boolean;
  standard: string;
  version: string;
  issues: ComplianceIssue[];
  checkedAt: string;
  metadata?: Record<string, unknown>;
}

export class ComplianceEngine {
  private static readonly logger = new Logger(ComplianceEngine.name);

  /**
   * PCI-DSS v4.0 Compliance Check
   * Validates against Requirement 3.4 and 3.5 (PAN masking/encryption)
   */
  static checkPciDss(data: Record<string, unknown>): ComplianceReport {
    const issues: ComplianceIssue[] = [];
    const flattened = this.flattenObject(data);

    // PCI-DSS v4.0 Requirement 3.4: Render PAN unreadable anywhere it is stored
    for (const [path, value] of Object.entries(flattened)) {
      if (typeof value === 'string') {
        // Detect potential PANs (13-19 digits)
        const potentialPANs = value.match(/\b\d{13,19}\b/g);

        if (potentialPANs) {
          for (const pan of potentialPANs) {
            if (this.luhnCheck(pan)) {
              issues.push({
                code: 'PCI-DSS-4.0-3.4',
                severity: 'critical',
                message:
                  'Unencrypted Primary Account Number (PAN) detected in plain text',
                path,
                remediation:
                  'Implement tokenization or strong encryption (AES-256) immediately.',
              });
            }
          }
        }

        // Check for CVV/CVC (3-4 digits, typically labeled)
        if (/cvv|cvc|security.code/i.test(path) && /\b\d{3,4}\b/.test(value)) {
          issues.push({
            code: 'PCI-DSS-4.0-3.2',
            severity: 'critical',
            message: 'Sensitive Authentication Data (CVV/CVC) detected',
            path,
            remediation:
              'Do not store sensitive authentication data after authorization.',
          });
        }
      }
    }

    return {
      compliant: issues.length === 0,
      standard: 'PCI-DSS',
      version: '4.0',
      issues,
      checkedAt: new Date().toISOString(),
    };
  }

  /**
   * HIPAA Security Rule Check (2025 Update)
   * Focuses on PHI (Protected Health Information) detection
   */
  static checkHipaa(data: Record<string, unknown>): ComplianceReport {
    const issues: ComplianceIssue[] = [];
    const flattened = this.flattenObject(data);

    // HIPAA 18 identifiers (Safe Harbor Method) matching patterns
    // ePHI: Electronic Protected Health Information
    const phiPatterns = [
      {
        pattern: /\b\d{3}-\d{2}-\d{4}\b/,
        type: 'SSN',
        code: 'HIPAA-164.514(b)(2)(i)(A)',
      },
      {
        pattern: /\b\d{10}\b/,
        type: 'NPI/Medical ID',
        context: ['npi', 'med', 'patient'],
        code: 'HIPAA-164.514(b)(2)(i)(F)',
      },
      {
        pattern: /[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}/i,
        type: 'Email',
        context: ['patient', 'health'],
        code: 'HIPAA-164.514(b)(2)(i)(D)',
      },
    ];

    for (const [path, value] of Object.entries(flattened)) {
      if (typeof value === 'string') {
        // Detect SSNs
        if (phiPatterns[0].pattern.test(value)) {
          issues.push({
            code: phiPatterns[0].code,
            severity: 'error',
            message: `Potential PHI detected: ${phiPatterns[0].type}`,
            path,
            remediation:
              'Ensure de-identification or encryption of PHI at rest and in transit.',
          });
        }

        // Context-aware checking for other identifiers
        for (const p of phiPatterns.slice(1)) {
          const isContextMatch = p.context?.some((c) =>
            path.toLowerCase().includes(c),
          );
          if (isContextMatch && p.pattern.test(value)) {
            issues.push({
              code: p.code,
              severity: 'warning',
              message: `Potential ePHI detected: ${p.type} in context '${path}'`,
              path,
              remediation:
                'Apply Minimum Necessary Standard (45 CFR 164.502(b)).',
            });
          }
        }

        // Diagnoses codes (ICD-10 checks roughly)
        if (/icd|diagnosis|condition/i.test(path) && /[A-Z]\d{2}/.test(value)) {
          issues.push({
            code: 'HIPAA-164.514(b)(2)(i)(O)',
            severity: 'error',
            message: 'Medical condition/diagnosis data detected',
            path,
            remediation: 'Ensure this ePHI is encrypted and access-controlled.',
          });
        }
      }
    }

    return {
      compliant:
        issues.filter(
          (i) => i.severity === 'error' || i.severity === 'critical',
        ).length === 0,
      standard: 'HIPAA Security Rule',
      version: '2025',
      issues,
      checkedAt: new Date().toISOString(),
    };
  }

  /**
   * GDPR & EU AI Act Compliance
   * Checks for Lawful Basis (Article 6) and Special Category Data (Article 9)
   */
  static checkGdpr(data: Record<string, unknown>): ComplianceReport {
    const issues: ComplianceIssue[] = [];
    const flattened = this.flattenObject(data);

    // EU AI Act: Transparency requirements
    if (data.role === 'ai_agent' && !data.disclosure) {
      issues.push({
        code: 'EU-AI-Act-Art50',
        severity: 'warning',
        message: 'AI Agent interaction disclosure missing',
        remediation:
          'AI systems must disclose that users are interacting with an AI.',
      });
    }

    // GDPR Article 6: Lawful Basis (Consent check)
    const hasConsent = Object.keys(flattened).some((k) =>
      /consent|lawful_basis|processing_agreement|legitimate_interest/.test(
        k.toLowerCase(),
      ),
    );

    if (!hasConsent) {
      issues.push({
        code: 'GDPR-Art6',
        severity: 'error',
        message:
          'No record of consent or lawful basis for processing found in context',
        remediation:
          'Ensure explict consent is captured and stored with the data payload.',
      });
    }

    // GDPR Article 9: Special Category Data (Biometrics, Political, Religious, etc.)
    const specialCategoryKeywords = [
      'biometric',
      'genetic',
      'political',
      'religious',
      'union',
      'sexual',
      'ethnic',
    ];

    for (const [path] of Object.entries(flattened)) {
      if (specialCategoryKeywords.some((k) => path.toLowerCase().includes(k))) {
        issues.push({
          code: 'GDPR-Art9',
          severity: 'critical',
          message: 'Processing of Special Category Data detected',
          path,
          remediation:
            'Explicit consent (Art 9(2)(a)) or specific exception required for this data type.',
        });
      }
    }

    return {
      compliant:
        issues.filter(
          (i) => i.severity === 'error' || i.severity === 'critical',
        ).length === 0,
      standard: 'GDPR & EU AI Act',
      version: '2026',
      issues,
      checkedAt: new Date().toISOString(),
    };
  }

  /**
   * Luhn Algorithm implementation for valid credit card detection
   * Better than regex which produces false positives
   */
  private static luhnCheck(val: string): boolean {
    let sum = 0;
    let shouldDouble = false;
    // Loop through values starting at the rightmost side
    for (let i = val.length - 1; i >= 0; i--) {
      let digit = parseInt(val.charAt(i));

      if (shouldDouble) {
        if ((digit *= 2) > 9) digit -= 9;
      }

      sum += digit;
      shouldDouble = !shouldDouble;
    }
    return sum % 10 == 0;
  }

  /**
   * Utility to flatten nested objects for deep inspection
   */
  private static flattenObject(
    obj: Record<string, unknown>,
    prefix = '',
  ): Record<string, unknown> {
    return Object.keys(obj).reduce(
      (acc: Record<string, unknown>, k: string) => {
        const pre = prefix.length ? prefix + '.' : '';
        if (
          typeof obj[k] === 'object' &&
          obj[k] !== null &&
          !Array.isArray(obj[k])
        ) {
          Object.assign(
            acc,
            this.flattenObject(obj[k] as Record<string, unknown>, pre + k),
          );
        } else {
          acc[pre + k] = obj[k];
        }
        return acc;
      },
      {},
    );
  }
}
