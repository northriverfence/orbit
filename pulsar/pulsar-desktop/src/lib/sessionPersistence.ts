/**
 * Session Persistence Utility
 *
 * Handles saving and restoring terminal sessions to/from disk.
 * Sessions are stored in ~/.config/pulsar/sessions.json
 */

import { Session } from '../components/SessionTabs'
import { invoke } from '@tauri-apps/api/core'
import { exists, readTextFile, writeTextFile, BaseDirectory, mkdir } from '@tauri-apps/plugin-fs'

const CONFIG_DIR = 'pulsar'
const SESSIONS_FILE = 'sessions.json'

export interface PersistedSession extends Session {
  createdAt: string
  lastActive: string
  sessionConfig?: {
    host?: string
    port?: number
    username?: string
    // password is NOT persisted for security
  }
}

export interface SessionState {
  version: string
  sessions: PersistedSession[]
  activeSessionId: string | null
  lastSaved: string
}

/**
 * Initialize config directory if it doesn't exist
 */
async function ensureConfigDir(): Promise<void> {
  try {
    const dirExists = await exists(CONFIG_DIR, { baseDir: BaseDirectory.AppConfig })
    if (!dirExists) {
      await mkdir(CONFIG_DIR, { baseDir: BaseDirectory.AppConfig, recursive: true })
    }
  } catch (error) {
    console.error('Failed to create config directory:', error)
    throw error
  }
}

/**
 * Save sessions to disk
 */
export async function saveSessions(
  sessions: PersistedSession[],
  activeSessionId: string | null
): Promise<void> {
  try {
    await ensureConfigDir()

    const state: SessionState = {
      version: '1.0.0',
      sessions: sessions.map((session) => ({
        ...session,
        lastActive: session.id === activeSessionId ? new Date().toISOString() : session.lastActive,
        // Remove password from persistence
        sessionConfig: session.sessionConfig
          ? {
              host: session.sessionConfig.host,
              port: session.sessionConfig.port,
              username: session.sessionConfig.username,
            }
          : undefined,
      })),
      activeSessionId,
      lastSaved: new Date().toISOString(),
    }

    const filePath = `${CONFIG_DIR}/${SESSIONS_FILE}`
    await writeTextFile(filePath, JSON.stringify(state, null, 2), {
      baseDir: BaseDirectory.AppConfig,
    })

    console.log(`Saved ${sessions.length} sessions to ${filePath}`)
  } catch (error) {
    console.error('Failed to save sessions:', error)
    throw error
  }
}

/**
 * Load sessions from disk
 */
export async function loadSessions(): Promise<{
  sessions: PersistedSession[]
  activeSessionId: string | null
} | null> {
  try {
    await ensureConfigDir()

    const filePath = `${CONFIG_DIR}/${SESSIONS_FILE}`
    const fileExists = await exists(filePath, { baseDir: BaseDirectory.AppConfig })

    if (!fileExists) {
      console.log('No saved sessions found')
      return null
    }

    const content = await readTextFile(filePath, { baseDir: BaseDirectory.AppConfig })
    const state: SessionState = JSON.parse(content)

    console.log(`Loaded ${state.sessions.length} sessions from ${filePath}`)

    return {
      sessions: state.sessions,
      activeSessionId: state.activeSessionId,
    }
  } catch (error) {
    console.error('Failed to load sessions:', error)
    return null
  }
}

/**
 * Clear all saved sessions
 */
export async function clearSessions(): Promise<void> {
  try {
    const filePath = `${CONFIG_DIR}/${SESSIONS_FILE}`
    const fileExists = await exists(filePath, { baseDir: BaseDirectory.AppConfig })

    if (fileExists) {
      await writeTextFile(filePath, JSON.stringify({ sessions: [], version: '1.0.0' }, null, 2), {
        baseDir: BaseDirectory.AppConfig,
      })
      console.log('Cleared all saved sessions')
    }
  } catch (error) {
    console.error('Failed to clear sessions:', error)
    throw error
  }
}

/**
 * Auto-save sessions with debouncing
 */
export class SessionAutoSaver {
  private timeoutId: NodeJS.Timeout | null = null
  private readonly debounceMs: number

  constructor(debounceMs: number = 1000) {
    this.debounceMs = debounceMs
  }

  /**
   * Schedule an auto-save operation
   * Previous pending saves are cancelled
   */
  scheduleSave(sessions: PersistedSession[], activeSessionId: string | null): void {
    if (this.timeoutId) {
      clearTimeout(this.timeoutId)
    }

    this.timeoutId = setTimeout(async () => {
      try {
        await saveSessions(sessions, activeSessionId)
      } catch (error) {
        console.error('Auto-save failed:', error)
      }
      this.timeoutId = null
    }, this.debounceMs)
  }

  /**
   * Cancel any pending save
   */
  cancel(): void {
    if (this.timeoutId) {
      clearTimeout(this.timeoutId)
      this.timeoutId = null
    }
  }

  /**
   * Force immediate save (flushes any pending save)
   */
  async flush(sessions: PersistedSession[], activeSessionId: string | null): Promise<void> {
    this.cancel()
    await saveSessions(sessions, activeSessionId)
  }
}

/**
 * Export sessions to a custom location
 */
export async function exportSessions(
  sessions: PersistedSession[],
  activeSessionId: string | null,
  exportPath: string
): Promise<void> {
  try {
    const state: SessionState = {
      version: '1.0.0',
      sessions,
      activeSessionId,
      lastSaved: new Date().toISOString(),
    }

    // Use Tauri's file dialog and save API
    await invoke('write_file', {
      path: exportPath,
      contents: JSON.stringify(state, null, 2),
    })

    console.log(`Exported ${sessions.length} sessions to ${exportPath}`)
  } catch (error) {
    console.error('Failed to export sessions:', error)
    throw error
  }
}

/**
 * Import sessions from a file
 */
export async function importSessions(importPath: string): Promise<{
  sessions: PersistedSession[]
  activeSessionId: string | null
}> {
  try {
    const content = await invoke<string>('read_file', { path: importPath })
    const state: SessionState = JSON.parse(content)

    console.log(`Imported ${state.sessions.length} sessions from ${importPath}`)

    return {
      sessions: state.sessions,
      activeSessionId: state.activeSessionId,
    }
  } catch (error) {
    console.error('Failed to import sessions:', error)
    throw error
  }
}
