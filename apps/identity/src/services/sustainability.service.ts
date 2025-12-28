/**
 * AgentKernIdentity - Sustainability Service
 *
 * Tracks the carbon footprint and energy efficiency of the application.
 * Implements recommendation from the sustainability audit.
 *
 * Features:
 * - Request-level energy estimation
 * - CO2 emission calculation (grams)
 * - Energy efficiency metrics (compute, memory, network)
 * - Carbon-aware optimization suggestions
 */

import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { AuditLoggerService, AuditEventType } from './audit-logger.service';

export interface SustainabilityMetrics {
  computeMilliseconds: number;
  memoryMBSeconds: number;
  networkBytes: number;
  estimatedCO2Grams: number;
}

export interface CarbonReport {
  period: string;
  totalCO2Grams: number;
  avgCO2PerRequest: number;
  efficiencyTrend: 'improving' | 'stable' | 'declining';
  topEmissionEndpoints: Array<{ endpoint: string; co2: number }>;
}

@Injectable()
export class SustainabilityService implements OnModuleInit {
  private readonly logger = new Logger(SustainabilityService.name);

  // Constants for emission estimation (simplified)
  // Source: Average data center energy intensity and grid emission factors (2025)
  private readonly ENERGY_PER_CPU_SECOND = 0.05; // Watt-seconds
  private readonly ENERGY_PER_MB_SECOND = 0.0001; // Watt-seconds
  private readonly ENERGY_PER_NETWORK_BYTE = 0.0000001; // Watt-seconds
  private readonly GRID_CARBON_INTENSITY = 0.4; // Grams CO2 per Watt-second (varies by region/time)

  // In-memory metrics store
  private metricsMap: Map<string, SustainabilityMetrics[]> = new Map();

  constructor(private readonly auditLogger: AuditLoggerService) {}

  async onModuleInit(): Promise<void> {
    this.logger.log('🌱 Sustainability Service initialized');
  }

  /**
   * Track resource usage for an API request
   */
  trackRequest(endpoint: string, metrics: Partial<SustainabilityMetrics>): void {
    const fullMetrics: SustainabilityMetrics = {
      computeMilliseconds: metrics.computeMilliseconds || 0,
      memoryMBSeconds: metrics.memoryMBSeconds || 0,
      networkBytes: metrics.networkBytes || 0,
      estimatedCO2Grams: this.calculateCO2(metrics),
    };

    const endpointMetrics = this.metricsMap.get(endpoint) || [];
    endpointMetrics.push(fullMetrics);
    
    // Keep only last 1000 samples per endpoint to prevent memory leak
    if (endpointMetrics.length > 1000) {
      endpointMetrics.shift();
    }
    
    this.metricsMap.set(endpoint, endpointMetrics);

    // Only log high emission requests to audit log
    if (fullMetrics.estimatedCO2Grams > 0.01) {
      this.logger.debug(`High emission request tracked for ${endpoint}: ${fullMetrics.estimatedCO2Grams.toFixed(4)}g CO2`);
    }
  }

  /**
   * Calculate CO2 emissions in grams based on resource usage
   */
  private calculateCO2(metrics: Partial<SustainabilityMetrics>): number {
    const cpuEnergy = (metrics.computeMilliseconds || 0) / 1000 * this.ENERGY_PER_CPU_SECOND;
    const memEnergy = (metrics.memoryMBSeconds || 0) * this.ENERGY_PER_MB_SECOND;
    const netEnergy = (metrics.networkBytes || 0) * this.ENERGY_PER_NETWORK_BYTE;

    const totalEnergy = cpuEnergy + memEnergy + netEnergy;
    return totalEnergy * this.GRID_CARBON_INTENSITY;
  }

  /**
   * Generate a carbon footprint report
   */
  async generateReport(): Promise<CarbonReport> {
    let totalCO2 = 0;
    let totalRequests = 0;
    const items: Array<{ endpoint: string; co2: number }> = [];

    for (const [endpoint, metricsList] of this.metricsMap.entries()) {
      const endpointTotal = metricsList.reduce((sum, m) => sum + m.estimatedCO2Grams, 0);
      totalCO2 += endpointTotal;
      totalRequests += metricsList.length;
      items.push({ endpoint, co2: endpointTotal });
    }

    const report: CarbonReport = {
      period: 'last_hour', // Default
      totalCO2Grams: totalCO2,
      avgCO2PerRequest: totalRequests > 0 ? totalCO2 / totalRequests : 0,
      efficiencyTrend: 'stable',
      topEmissionEndpoints: items.sort((a, b) => b.co2 - a.co2).slice(0, 5),
    };

    this.auditLogger.logSecurityEvent(
      AuditEventType.COMPLIANCE_REPORT_GENERATED,
      `Sustainability report generated`,
      { totalCO2: report.totalCO2Grams, avgCO2: report.avgCO2PerRequest },
    );

    return report;
  }

  /**
   * Get optimization suggestions
   */
  getOptimizationSuggestions(): string[] {
    const suggestions: string[] = [];
    
    // Look for high-emission endpoints
    for (const [endpoint, metrics] of this.metricsMap.entries()) {
      const avgCO2 = metrics.reduce((sum, m) => sum + m.estimatedCO2Grams, 0) / metrics.length;
      if (avgCO2 > 0.05) {
        suggestions.push(`Endpoint ${endpoint} has high carbon intensity. Consider caching or query optimization.`);
      }
    }

    if (suggestions.length === 0) {
      suggestions.push('No critical carbon hotspots detected. Continue monitoring.');
    }

    return suggestions;
  }
}
