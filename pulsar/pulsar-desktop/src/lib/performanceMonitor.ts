/**
 * Performance Monitoring Utilities for Pulsar Desktop
 *
 * Provides tools for measuring and optimizing React component performance
 */

import React from 'react';

interface PerformanceMetric {
  name: string;
  duration: number;
  timestamp: number;
  metadata?: Record<string, unknown>;
}

class PerformanceMonitor {
  private metrics: PerformanceMetric[] = [];
  private marks: Map<string, number> = new Map();
  private enabled: boolean = import.meta.env.DEV;

  /**
   * Start measuring an operation
   */
  mark(name: string): void {
    if (!this.enabled) return;
    this.marks.set(name, performance.now());
  }

  /**
   * End measurement and record the metric
   */
  measure(name: string, metadata?: Record<string, unknown>): number | null {
    if (!this.enabled) return null;

    const startTime = this.marks.get(name);
    if (!startTime) {
      console.warn(`[Performance] No mark found for "${name}"`);
      return null;
    }

    const duration = performance.now() - startTime;
    this.metrics.push({
      name,
      duration,
      timestamp: Date.now(),
      metadata,
    });

    this.marks.delete(name);

    if (duration > 100) {
      console.warn(`[Performance] Slow operation: ${name} took ${duration.toFixed(2)}ms`);
    }

    return duration;
  }

  /**
   * Measure a synchronous function
   */
  measureSync<T>(name: string, fn: () => T, metadata?: Record<string, unknown>): T {
    if (!this.enabled) return fn();

    this.mark(name);
    const result = fn();
    this.measure(name, metadata);
    return result;
  }

  /**
   * Measure an async function
   */
  async measureAsync<T>(
    name: string,
    fn: () => Promise<T>,
    metadata?: Record<string, unknown>
  ): Promise<T> {
    if (!this.enabled) return fn();

    this.mark(name);
    try {
      const result = await fn();
      this.measure(name, metadata);
      return result;
    } catch (error) {
      this.measure(name, { ...metadata, error: true });
      throw error;
    }
  }

  /**
   * Get all recorded metrics
   */
  getMetrics(): PerformanceMetric[] {
    return [...this.metrics];
  }

  /**
   * Get metrics for a specific operation
   */
  getMetricsFor(name: string): PerformanceMetric[] {
    return this.metrics.filter((m) => m.name === name);
  }

  /**
   * Get average duration for an operation
   */
  getAverage(name: string): number {
    const metrics = this.getMetricsFor(name);
    if (metrics.length === 0) return 0;

    const sum = metrics.reduce((acc, m) => acc + m.duration, 0);
    return sum / metrics.length;
  }

  /**
   * Get slowest operations
   */
  getSlowest(count: number = 10): PerformanceMetric[] {
    return [...this.metrics]
      .sort((a, b) => b.duration - a.duration)
      .slice(0, count);
  }

  /**
   * Clear all metrics
   */
  clear(): void {
    this.metrics = [];
    this.marks.clear();
  }

  /**
   * Export metrics as JSON
   */
  export(): string {
    return JSON.stringify({
      metrics: this.metrics,
      summary: {
        totalOperations: this.metrics.length,
        slowest: this.getSlowest(5),
        averages: this.getUniqueOperations().map((name) => ({
          name,
          average: this.getAverage(name),
          count: this.getMetricsFor(name).length,
        })),
      },
    }, null, 2);
  }

  /**
   * Get unique operation names
   */
  private getUniqueOperations(): string[] {
    return Array.from(new Set(this.metrics.map((m) => m.name)));
  }

  /**
   * Enable or disable monitoring
   */
  setEnabled(enabled: boolean): void {
    this.enabled = enabled;
  }

  /**
   * Print performance report to console
   */
  printReport(): void {
    if (!this.enabled || this.metrics.length === 0) {
      console.log('[Performance] No metrics recorded');
      return;
    }

    console.group('📊 Performance Report');
    console.log(`Total operations: ${this.metrics.length}`);
    console.log('');

    console.log('🐌 Slowest operations:');
    this.getSlowest(5).forEach((m, i) => {
      console.log(`  ${i + 1}. ${m.name}: ${m.duration.toFixed(2)}ms`);
    });
    console.log('');

    console.log('📈 Averages:');
    this.getUniqueOperations()
      .map((name) => ({
        name,
        average: this.getAverage(name),
        count: this.getMetricsFor(name).length,
      }))
      .sort((a, b) => b.average - a.average)
      .slice(0, 10)
      .forEach((op) => {
        console.log(`  ${op.name}: ${op.average.toFixed(2)}ms (${op.count} calls)`);
      });

    console.groupEnd();
  }
}

// Global instance
export const perfMonitor = new PerformanceMonitor();

/**
 * React component performance profiler
 */
export function profileComponent<P extends object>(
  Component: React.ComponentType<P>,
  componentName: string
): React.ComponentType<P> {
  return (props: P) => {
    perfMonitor.mark(`${componentName}:render`);

    React.useEffect(() => {
      perfMonitor.measure(`${componentName}:render`);
    });

    return React.createElement(Component, props);
  };
}

/**
 * Hook to measure component render performance
 */
export function useRenderPerformance(componentName: string): void {
  const renderCount = React.useRef(0);
  const startTime = React.useRef(performance.now());

  renderCount.current++;

  React.useEffect(() => {
    const duration = performance.now() - startTime.current;
    perfMonitor.measure(`${componentName}:render:${renderCount.current}`, {
      renderCount: renderCount.current,
    });

    if (duration > 16.67) {
      // Slower than 60fps
      console.warn(
        `[Performance] ${componentName} render #${renderCount.current} took ${duration.toFixed(2)}ms (> 16.67ms)`
      );
    }

    startTime.current = performance.now();
  });
}

/**
 * Hook to detect excessive re-renders
 */
export function useRenderCount(componentName: string, threshold: number = 10): void {
  const renderCount = React.useRef(0);
  const lastWarning = React.useRef(0);

  renderCount.current++;

  if (renderCount.current > threshold && renderCount.current - lastWarning.current > threshold) {
    console.warn(
      `[Performance] ${componentName} has rendered ${renderCount.current} times. Check for unnecessary re-renders.`
    );
    lastWarning.current = renderCount.current;
  }
}

/**
 * Hook to log why a component re-rendered
 */
export function useWhyDidYouUpdate(name: string, props: Record<string, unknown>): void {
  const previousProps = React.useRef<Record<string, unknown>>();

  React.useEffect(() => {
    if (previousProps.current) {
      const allKeys = Object.keys({ ...previousProps.current, ...props });
      const changedProps: Record<string, { from: unknown; to: unknown }> = {};

      allKeys.forEach((key) => {
        if (previousProps.current?.[key] !== props[key]) {
          changedProps[key] = {
            from: previousProps.current?.[key],
            to: props[key],
          };
        }
      });

      if (Object.keys(changedProps).length > 0) {
        console.log(`[Why Update] ${name}`, changedProps);
      }
    }

    previousProps.current = props;
  });
}

/**
 * Measure memory usage
 */
export function measureMemory(): {
  usedJSHeapSize: number;
  totalJSHeapSize: number;
  jsHeapSizeLimit: number;
} | null {
  if (!('memory' in performance)) {
    console.warn('[Performance] Memory API not available');
    return null;
  }

  const memory = (performance as Performance & { memory: {
    usedJSHeapSize: number;
    totalJSHeapSize: number;
    jsHeapSizeLimit: number;
  } }).memory;

  return {
    usedJSHeapSize: memory.usedJSHeapSize,
    totalJSHeapSize: memory.totalJSHeapSize,
    jsHeapSizeLimit: memory.jsHeapSizeLimit,
  };
}

/**
 * Print memory usage report
 */
export function printMemoryReport(): void {
  const memory = measureMemory();
  if (!memory) return;

  const usedMB = (memory.usedJSHeapSize / 1024 / 1024).toFixed(2);
  const totalMB = (memory.totalJSHeapSize / 1024 / 1024).toFixed(2);
  const limitMB = (memory.jsHeapSizeLimit / 1024 / 1024).toFixed(2);
  const usage = ((memory.usedJSHeapSize / memory.jsHeapSizeLimit) * 100).toFixed(2);

  console.group('💾 Memory Report');
  console.log(`Used: ${usedMB}MB`);
  console.log(`Total: ${totalMB}MB`);
  console.log(`Limit: ${limitMB}MB`);
  console.log(`Usage: ${usage}%`);

  if (parseFloat(usage) > 80) {
    console.warn('⚠️  High memory usage! Consider optimizing.');
  }

  console.groupEnd();
}

/**
 * Debounce utility for performance
 */
export function debounce<T extends (...args: unknown[]) => unknown>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout | null = null;

  return function executedFunction(...args: Parameters<T>) {
    const later = () => {
      timeout = null;
      func(...args);
    };

    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

/**
 * Throttle utility for performance
 */
export function throttle<T extends (...args: unknown[]) => unknown>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean;

  return function executedFunction(...args: Parameters<T>) {
    if (!inThrottle) {
      func(...args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}

/**
 * Measure FPS (Frames Per Second)
 */
export class FPSMonitor {
  private frames: number[] = [];
  private lastTime: number = performance.now();
  private rafId: number | null = null;

  start(): void {
    if (this.rafId !== null) return;

    const measureFrame = () => {
      const currentTime = performance.now();
      const delta = currentTime - this.lastTime;
      const fps = 1000 / delta;

      this.frames.push(fps);
      if (this.frames.length > 60) {
        this.frames.shift(); // Keep last 60 frames
      }

      this.lastTime = currentTime;
      this.rafId = requestAnimationFrame(measureFrame);
    };

    this.rafId = requestAnimationFrame(measureFrame);
  }

  stop(): void {
    if (this.rafId !== null) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
  }

  getAvgFPS(): number {
    if (this.frames.length === 0) return 0;
    const sum = this.frames.reduce((a, b) => a + b, 0);
    return Math.round(sum / this.frames.length);
  }

  getReport(): { avg: number; min: number; max: number; current: number } {
    return {
      avg: this.getAvgFPS(),
      min: Math.round(Math.min(...this.frames)),
      max: Math.round(Math.max(...this.frames)),
      current: Math.round(this.frames[this.frames.length - 1] || 0),
    };
  }
}

// Export global FPS monitor
export const fpsMonitor = new FPSMonitor();

// Auto-start FPS monitoring in dev mode
if (import.meta.env.DEV) {
  fpsMonitor.start();
}

// Global performance helpers (available in console)
if (typeof window !== 'undefined') {
  (window as unknown as { perf: typeof perfMonitor }).perf = perfMonitor;
  (window as unknown as { fps: typeof fpsMonitor }).fps = fpsMonitor;
  (window as unknown as { memReport: typeof printMemoryReport }).memReport = printMemoryReport;
}
