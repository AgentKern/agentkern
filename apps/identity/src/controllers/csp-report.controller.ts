import { Controller, Post, Body, HttpCode, HttpStatus, Logger } from '@nestjs/common';
import { ApiTags, ApiOperation, ApiBody, ApiResponse } from '@nestjs/swagger';

/**
 * CSP Violation Reporting Endpoint
 * 
 * Receives Content-Security-Policy violation reports from browsers.
 * This enables real-time monitoring of XSS and injection attempts.
 * 
 * @see https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP#violation_reporting
 */

interface CspViolationReport {
  'csp-report'?: {
    'document-uri'?: string;
    'violated-directive'?: string;
    'effective-directive'?: string;
    'original-policy'?: string;
    'blocked-uri'?: string;
    'status-code'?: number;
    'source-file'?: string;
    'line-number'?: number;
    'column-number'?: number;
    'script-sample'?: string;
  };
}

interface ReportToPayload {
  type?: string;
  age?: number;
  url?: string;
  body?: {
    documentURL?: string;
    violatedDirective?: string;
    effectiveDirective?: string;
    originalPolicy?: string;
    blockedURL?: string;
    statusCode?: number;
    sourceFile?: string;
    lineNumber?: number;
    columnNumber?: number;
    sample?: string;
  };
}

@ApiTags('Security')
@Controller('api/v1/security')
export class CspReportController {
  private readonly logger = new Logger(CspReportController.name);

  /**
   * CSP Violation Report Endpoint (Legacy format)
   * Receives Content-Security-Policy violation reports from browsers.
   */
  @Post('csp-report')
  @HttpCode(HttpStatus.NO_CONTENT)
  @ApiOperation({ summary: 'Receive CSP violation reports' })
  @ApiBody({ description: 'CSP violation report from browser' })
  @ApiResponse({ status: 204, description: 'Report received' })
  async receiveCspReport(@Body() report: CspViolationReport): Promise<void> {
    const cspReport = report['csp-report'];
    
    if (!cspReport) {
      this.logger.warn('Received malformed CSP report');
      return;
    }

    // Log for monitoring (integrate with SIEM in production)
    this.logger.warn({
      message: 'CSP Violation Detected',
      documentUri: cspReport['document-uri'],
      violatedDirective: cspReport['violated-directive'],
      blockedUri: cspReport['blocked-uri'],
      sourceFile: cspReport['source-file'],
      lineNumber: cspReport['line-number'],
      scriptSample: cspReport['script-sample']?.substring(0, 100), // Limit sample size
    });

    // In production: Send to security monitoring (Splunk, Datadog, etc.)
    // await this.securityMonitor.reportViolation(cspReport);
  }

  /**
   * Reporting API Endpoint (Modern format)
   * Receives reports via the Reporting API (Report-To header)
   */
  @Post('reports')
  @HttpCode(HttpStatus.NO_CONTENT)
  @ApiOperation({ summary: 'Receive security reports via Reporting API' })
  @ApiBody({ description: 'Reports from browser Reporting API' })
  @ApiResponse({ status: 204, description: 'Reports received' })
  async receiveReports(@Body() reports: ReportToPayload[]): Promise<void> {
    if (!Array.isArray(reports)) {
      this.logger.warn('Received malformed Reporting API payload');
      return;
    }

    for (const report of reports) {
      if (report.type === 'csp-violation') {
        this.logger.warn({
          message: 'CSP Violation (Reporting API)',
          url: report.url,
          violatedDirective: report.body?.violatedDirective,
          blockedURL: report.body?.blockedURL,
          sample: report.body?.sample?.substring(0, 100),
        });
      } else if (report.type === 'permissions-policy-violation') {
        this.logger.warn({
          message: 'Permissions Policy Violation',
          url: report.url,
          body: report.body,
        });
      }
    }
  }
}
