/**
 * Privacy-First Analytics System
 *
 * - Opt-in only (disabled by default)
 * - No PII collected
 * - Anonymous usage data
 * - User control
 * - Transparent data collection
 */

interface AnalyticsEvent {
  event: string;
  properties?: Record<string, unknown>;
  timestamp: number;
}

interface SystemInfo {
  platform: string;
  appVersion: string;
  screenResolution: string;
  availableMemory: number;
  cpuArchitecture: string;
}

class Analytics {
  private enabled: boolean = false;
  private events: AnalyticsEvent[] = [];
  private systemInfo: SystemInfo | null = null;
  private sessionId: string = '';
  private readonly STORAGE_KEY = 'analytics_enabled';
  private readonly MAX_EVENTS = 1000; // Keep last 1000 events

  constructor() {
    this.loadSettings();
    this.generateSessionId();
    this.collectSystemInfo();
  }

  /**
   * Load analytics settings from storage
   */
  private loadSettings(): void {
    const stored = localStorage.getItem(this.STORAGE_KEY);
    this.enabled = stored === 'true';
  }

  /**
   * Enable or disable analytics
   */
  setEnabled(enabled: boolean): void {
    this.enabled = enabled;
    localStorage.setItem(this.STORAGE_KEY, String(enabled));

    if (enabled) {
      this.track('analytics_enabled');
    } else {
      this.track('analytics_disabled');
      // Clear stored events when disabling
      this.clearEvents();
    }
  }

  /**
   * Check if analytics is enabled
   */
  isEnabled(): boolean {
    return this.enabled;
  }

  /**
   * Generate a random session ID (no PII)
   */
  private generateSessionId(): void {
    this.sessionId = `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Collect non-identifying system information
   */
  private async collectSystemInfo(): Promise<void> {
    try {
      this.systemInfo = {
        platform: navigator.platform,
        appVersion: import.meta.env.VITE_APP_VERSION || '0.1.0',
        screenResolution: `${window.screen.width}x${window.screen.height}`,
        availableMemory: (navigator as Navigator & { deviceMemory?: number }).deviceMemory || 0,
        cpuArchitecture: (navigator as Navigator & { userAgentData?: { platform?: string } }).userAgentData?.platform || 'unknown',
      };
    } catch (error) {
      console.warn('[Analytics] Failed to collect system info:', error);
    }
  }

  /**
   * Track an event
   */
  track(event: string, properties?: Record<string, unknown>): void {
    if (!this.enabled) return;

    const analyticsEvent: AnalyticsEvent = {
      event,
      properties: {
        ...properties,
        sessionId: this.sessionId,
        ...this.systemInfo,
      },
      timestamp: Date.now(),
    };

    this.events.push(analyticsEvent);

    // Limit stored events
    if (this.events.length > this.MAX_EVENTS) {
      this.events = this.events.slice(-this.MAX_EVENTS);
    }

    // Send to backend (if configured)
    this.sendToBackend(analyticsEvent).catch((error) => {
      console.warn('[Analytics] Failed to send event:', error);
    });

    // Log in dev mode
    if (import.meta.env.DEV) {
      console.log('[Analytics]', event, properties);
    }
  }

  /**
   * Track application lifecycle events
   */
  trackAppLaunched(): void {
    this.track('app_launched', {
      launchTime: Date.now(),
    });
  }

  trackAppClosed(): void {
    this.track('app_closed', {
      sessionDuration: this.getSessionDuration(),
    });
  }

  /**
   * Track feature usage
   */
  trackFeatureUsed(feature: string, metadata?: Record<string, unknown>): void {
    this.track('feature_used', {
      feature,
      ...metadata,
    });
  }

  /**
   * Track performance metrics
   */
  trackPerformance(metric: string, duration: number, metadata?: Record<string, unknown>): void {
    this.track('performance_metric', {
      metric,
      duration,
      ...metadata,
    });
  }

  /**
   * Track errors (with anonymized stack trace)
   */
  trackError(error: Error, context?: string): void {
    // Anonymize stack trace (remove file paths)
    const anonymizedStack = error.stack
      ?.split('\n')
      .map((line) => line.replace(/file:\/\/.*?\//, ''))
      .join('\n');

    this.track('error_occurred', {
      errorMessage: error.message,
      errorStack: anonymizedStack,
      context,
    });
  }

  /**
   * Get session duration
   */
  private getSessionDuration(): number {
    const sessionStart = parseInt(this.sessionId.split('_')[1]);
    return Date.now() - sessionStart;
  }

  /**
   * Get all tracked events
   */
  getEvents(): AnalyticsEvent[] {
    return [...this.events];
  }

  /**
   * Clear all events
   */
  clearEvents(): void {
    this.events = [];
  }

  /**
   * Export analytics data (for user to review)
   */
  export(): string {
    return JSON.stringify({
      enabled: this.enabled,
      sessionId: this.sessionId,
      systemInfo: this.systemInfo,
      events: this.events,
      summary: {
        totalEvents: this.events.length,
        sessionDuration: this.getSessionDuration(),
        mostUsedFeatures: this.getMostUsedFeatures(),
      },
    }, null, 2);
  }

  /**
   * Get most used features
   */
  private getMostUsedFeatures(): Array<{ feature: string; count: number }> {
    const featureUsage = new Map<string, number>();

    this.events
      .filter((e) => e.event === 'feature_used')
      .forEach((e) => {
        const feature = e.properties?.feature as string;
        if (feature) {
          featureUsage.set(feature, (featureUsage.get(feature) || 0) + 1);
        }
      });

    return Array.from(featureUsage.entries())
      .map(([feature, count]) => ({ feature, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 10);
  }

  /**
   * Send event to backend (placeholder)
   */
  private async sendToBackend(event: AnalyticsEvent): Promise<void> {
    // TODO: Implement backend endpoint
    // For now, just store locally
    // In production, send to analytics service

    // Example implementation:
    // await fetch('https://analytics.pulsar-desktop.com/event', {
    //   method: 'POST',
    //   headers: { 'Content-Type': 'application/json' },
    //   body: JSON.stringify(event),
    // });
  }

  /**
   * Show analytics consent dialog
   */
  showConsentDialog(): void {
    // This should be called by a React component
    // Fire event to show consent dialog
    window.dispatchEvent(new CustomEvent('show-analytics-consent'));
  }
}

// Global analytics instance
export const analytics = new Analytics();

/**
 * React hook for tracking component usage
 */
export function useAnalytics(componentName: string): void {
  React.useEffect(() => {
    analytics.trackFeatureUsed('component_rendered', { component: componentName });
  }, [componentName]);
}

/**
 * Track keyboard shortcut usage
 */
export function trackShortcut(shortcut: string): void {
  analytics.trackFeatureUsed('keyboard_shortcut', { shortcut });
}

/**
 * Track command palette command
 */
export function trackCommand(command: string): void {
  analytics.trackFeatureUsed('command_executed', { command });
}

/**
 * Track session creation
 */
export function trackSessionCreated(type: 'local' | 'ssh'): void {
  analytics.trackFeatureUsed('session_created', { sessionType: type });
}

/**
 * Track file transfer
 */
export function trackFileTransfer(direction: 'upload' | 'download', size: number): void {
  analytics.trackFeatureUsed('file_transfer', {
    direction,
    sizeBytes: size,
  });
}

// Auto-track app lifecycle
window.addEventListener('load', () => {
  analytics.trackAppLaunched();
});

window.addEventListener('beforeunload', () => {
  analytics.trackAppClosed();
});

// Global error tracking
window.addEventListener('error', (event) => {
  analytics.trackError(event.error || new Error(event.message), 'window_error');
});

window.addEventListener('unhandledrejection', (event) => {
  analytics.trackError(
    new Error(event.reason?.message || String(event.reason)),
    'unhandled_promise_rejection'
  );
});

// Expose analytics to console (for debugging)
if (import.meta.env.DEV) {
  (window as Window & { analytics: Analytics }).analytics = analytics;
}

export default analytics;
