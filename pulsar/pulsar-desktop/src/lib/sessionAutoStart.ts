/**
 * Session Auto-Start Service
 *
 * Handles automatic session creation when loading workspaces
 */

import { invoke } from '@tauri-apps/api/core';
import type { WorkspaceStartupConfig, SessionStartupConfig } from '../types/sessionAutoStart';
import {
  escapeShellArg,
  isValidEnvVarName,
  validatePath,
  validateHostname,
  validatePort,
  sanitizeCommand,
  SecureStorage,
} from './security';

export class SessionAutoStartService {
  /**
   * Start all configured sessions for a workspace
   */
  static async startWorkspaceSessions(
    config: WorkspaceStartupConfig,
    onSessionCreated?: (paneId: string, sessionId: string) => void,
    onError?: (paneId: string, error: string) => void
  ): Promise<Map<string, string>> {
    const sessionMap = new Map<string, string>(); // paneId -> sessionId

    if (!config.autoStart) {
      console.log('Auto-start disabled for this workspace');
      return sessionMap;
    }

    // Start sessions in order
    for (const paneId of config.startupOrder) {
      const sessionConfig = config.panes[paneId];
      if (!sessionConfig || !sessionConfig.enabled) {
        continue;
      }

      // Apply startup delay if configured
      if (sessionConfig.startupDelay && sessionConfig.startupDelay > 0) {
        await new Promise((resolve) => setTimeout(resolve, sessionConfig.startupDelay));
      }

      try {
        const sessionId = await this.startSession(sessionConfig, config.globalEnv);
        sessionMap.set(paneId, sessionId);

        if (onSessionCreated) {
          onSessionCreated(paneId, sessionId);
        }

        console.log(`Started session for pane ${paneId}: ${sessionId}`);
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Unknown error';
        console.error(`Failed to start session for pane ${paneId}:`, errorMsg);

        if (onError) {
          onError(paneId, errorMsg);
        }
      }
    }

    return sessionMap;
  }

  /**
   * Start a single session based on configuration
   */
  private static async startSession(
    config: SessionStartupConfig,
    globalEnv?: Record<string, string>
  ): Promise<string> {
    if (config.type === 'local') {
      return this.startLocalSession(config, globalEnv);
    } else if (config.type === 'ssh') {
      return this.startSshSession(config);
    } else {
      throw new Error(`Unknown session type: ${config.type}`);
    }
  }

  /**
   * Start a local terminal session
   */
  private static async startLocalSession(
    config: SessionStartupConfig,
    globalEnv?: Record<string, string>
  ): Promise<string> {
    const name = config.name || 'Local Terminal';

    // Validate working directory if specified
    if (config.cwd) {
      const pathValidation = validatePath(config.cwd);
      if (!pathValidation.valid) {
        throw new Error(`Invalid working directory: ${pathValidation.error}`);
      }
    }

    // Create session via daemon
    const sessionId = await invoke<string>('daemon_create_local_session', {
      name,
      cols: 80,
      rows: 24,
    });

    // Attach to session
    await invoke('daemon_attach_session', {
      sessionId,
    });

    // Send startup command if configured
    if (config.command) {
      // Merge global and local env vars
      const env = { ...globalEnv, ...config.env };

      // Build command with env vars and cwd (with proper escaping)
      let fullCommand = '';

      // Change directory if specified (with escaping)
      if (config.cwd) {
        fullCommand += `cd ${escapeShellArg(config.cwd)}\n`;
      }

      // Export env vars (with validation and escaping)
      for (const [key, value] of Object.entries(env)) {
        // Validate environment variable name
        if (!isValidEnvVarName(key)) {
          throw new Error(`Invalid environment variable name: ${key}`);
        }
        // Escape the value
        fullCommand += `export ${key}=${escapeShellArg(value)}\n`;
      }

      // Sanitize and add the actual command
      const sanitized = sanitizeCommand(config.command);
      fullCommand += `${sanitized}\n`;

      // Send to terminal
      await invoke('daemon_send_input', {
        sessionId,
        data: fullCommand,
      });
    }

    return sessionId;
  }

  /**
   * Start an SSH session
   */
  private static async startSshSession(config: SessionStartupConfig): Promise<string> {
    if (!config.host) {
      throw new Error('SSH host is required');
    }

    // Validate hostname
    const hostValidation = validateHostname(config.host);
    if (!hostValidation.valid) {
      throw new Error(`Invalid SSH host: ${hostValidation.error}`);
    }

    const name = config.name || `SSH: ${config.host}`;
    const port = config.port || 22;

    // Validate port
    const portValidation = validatePort(port);
    if (!portValidation.valid) {
      throw new Error(`Invalid SSH port: ${portValidation.error}`);
    }

    const sessionId = await invoke<string>('daemon_create_ssh_session', {
      name,
      host: config.host,
      port,
      cols: 80,
      rows: 24,
    });

    // Attach to session
    await invoke('daemon_attach_session', {
      sessionId,
    });

    return sessionId;
  }

  /**
   * Stop all sessions in a workspace
   */
  static async stopWorkspaceSessions(sessionIds: string[]): Promise<void> {
    for (const sessionId of sessionIds) {
      try {
        await invoke('daemon_terminate_session', {
          sessionId,
        });
        console.log(`Terminated session: ${sessionId}`);
      } catch (error) {
        console.error(`Failed to terminate session ${sessionId}:`, error);
      }
    }
  }

  /**
   * Validate session configuration
   */
  static validateConfig(config: SessionStartupConfig): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (config.type === 'ssh') {
      if (!config.host) {
        errors.push('SSH host is required');
      }
      if (config.port && (config.port < 1 || config.port > 65535)) {
        errors.push('SSH port must be between 1 and 65535');
      }
    }

    if (config.startupDelay && config.startupDelay < 0) {
      errors.push('Startup delay cannot be negative');
    }

    return {
      valid: errors.length === 0,
      errors,
    };
  }

  /**
   * Extract startup config from workspace layout
   */
  static extractStartupConfig(workspace: any): WorkspaceStartupConfig | null {
    // Check if workspace has startup metadata
    if (!workspace.metadata?.startupConfig) {
      return null;
    }

    return workspace.metadata.startupConfig as WorkspaceStartupConfig;
  }

  /**
   * Save startup config to workspace (encrypted)
   */
  static async saveStartupConfig(
    workspaceId: string,
    config: WorkspaceStartupConfig
  ): Promise<void> {
    const key = `workspace_startup_${workspaceId}`;
    const data = JSON.stringify(config);

    try {
      await SecureStorage.setItem(key, data);
    } catch (error) {
      console.error('Failed to save startup config:', error);
      throw new Error('Failed to save configuration. Please try again.');
    }
  }

  /**
   * Load startup config for workspace (decrypted)
   */
  static async loadStartupConfig(workspaceId: string): Promise<WorkspaceStartupConfig | null> {
    const key = `workspace_startup_${workspaceId}`;

    try {
      const stored = await SecureStorage.getItem(key);
      if (!stored) {
        return null;
      }

      return JSON.parse(stored) as WorkspaceStartupConfig;
    } catch (error) {
      console.error('Failed to load startup config:', error);
      return null;
    }
  }
}

export default SessionAutoStartService;
