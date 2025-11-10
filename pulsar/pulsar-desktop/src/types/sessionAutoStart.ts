/**
 * Session Auto-Start Type Definitions
 *
 * Configuration for automatically starting sessions when loading a workspace
 */

export type SessionType = 'local' | 'ssh';

export interface SessionStartupConfig {
  enabled: boolean;
  type: SessionType;
  name?: string;

  // Local session config
  command?: string;
  cwd?: string;
  env?: Record<string, string>;

  // SSH session config
  host?: string;
  port?: number;
  username?: string;

  // Startup options
  autoReconnect?: boolean;
  startupDelay?: number; // milliseconds
  order?: number; // startup order (lower = earlier)
}

export interface PaneSessionConfig extends SessionStartupConfig {
  paneId: string;
}

export interface WorkspaceStartupConfig {
  autoStart: boolean;
  panes: Record<string, SessionStartupConfig>; // paneId -> config
  globalEnv?: Record<string, string>;
  startupOrder: string[]; // paneIds in startup order
}

/**
 * Default session config
 */
export const createDefaultSessionConfig = (): SessionStartupConfig => ({
  enabled: false,
  type: 'local',
  autoReconnect: false,
  startupDelay: 0,
  order: 0,
});

/**
 * Example configurations for common scenarios
 */
export const sessionExamples = {
  localShell: {
    enabled: true,
    type: 'local' as SessionType,
    name: 'Shell',
    command: '',
    cwd: '~',
  },

  npmDev: {
    enabled: true,
    type: 'local' as SessionType,
    name: 'NPM Dev Server',
    command: 'npm run dev',
    cwd: '.',
  },

  dockerCompose: {
    enabled: true,
    type: 'local' as SessionType,
    name: 'Docker Compose',
    command: 'docker-compose up',
    cwd: '.',
  },

  sshServer: {
    enabled: true,
    type: 'ssh' as SessionType,
    name: 'Production Server',
    host: 'prod.example.com',
    port: 22,
    username: 'deploy',
  },

  tailLogs: {
    enabled: true,
    type: 'local' as SessionType,
    name: 'Logs',
    command: 'tail -f /var/log/app.log',
  },
};
