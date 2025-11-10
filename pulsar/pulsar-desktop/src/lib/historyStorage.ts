/**
 * Command History Storage
 *
 * Manages persistence of command history to disk.
 * Storage location: ~/.config/pulsar/command_history.json
 */

import { exists, readTextFile, writeTextFile, BaseDirectory, mkdir } from '@tauri-apps/plugin-fs'
import type { CommandHistoryEntry } from './commandCapture'

const CONFIG_DIR = 'pulsar'
const HISTORY_FILE = 'command_history.json'
const MAX_ENTRIES = 10000 // Maximum number of entries to keep

export interface HistoryState {
  version: string
  entries: CommandHistoryEntry[]
  maxEntries: number
  lastSaved: string
}

export interface HistoryFilter {
  sessionId?: string
  startDate?: string
  endDate?: string
  command?: string // partial match
  exitCode?: number
}

export interface HistorySearchOptions {
  query: string
  caseSensitive?: boolean
  regex?: boolean
  sessionId?: string
  limit?: number
}

/**
 * Initialize config directory
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
 * Load command history from disk
 */
export async function loadHistory(): Promise<CommandHistoryEntry[]> {
  try {
    await ensureConfigDir()

    const filePath = `${CONFIG_DIR}/${HISTORY_FILE}`
    const fileExists = await exists(filePath, { baseDir: BaseDirectory.AppConfig })

    if (!fileExists) {
      console.log('No command history found')
      return []
    }

    const content = await readTextFile(filePath, { baseDir: BaseDirectory.AppConfig })
    const state: HistoryState = JSON.parse(content)

    console.log(`Loaded ${state.entries.length} commands from history`)
    return state.entries
  } catch (error) {
    console.error('Failed to load command history:', error)
    return []
  }
}

/**
 * Save command history to disk
 */
export async function saveHistory(entries: CommandHistoryEntry[]): Promise<void> {
  try {
    await ensureConfigDir()

    // Rotate history if exceeding max entries
    const trimmed = entries.slice(-MAX_ENTRIES)

    const state: HistoryState = {
      version: '1.0.0',
      entries: trimmed,
      maxEntries: MAX_ENTRIES,
      lastSaved: new Date().toISOString(),
    }

    const filePath = `${CONFIG_DIR}/${HISTORY_FILE}`
    await writeTextFile(filePath, JSON.stringify(state, null, 2), {
      baseDir: BaseDirectory.AppConfig,
    })

    console.log(`Saved ${trimmed.length} commands to history`)
  } catch (error) {
    console.error('Failed to save command history:', error)
    throw error
  }
}

/**
 * Add a command to history
 */
export async function addCommandToHistory(entry: CommandHistoryEntry): Promise<void> {
  const history = await loadHistory()
  history.push(entry)
  await saveHistory(history)
}

/**
 * Update a command in history
 */
export async function updateCommandInHistory(
  commandId: string,
  updates: Partial<CommandHistoryEntry>
): Promise<void> {
  const history = await loadHistory()
  const index = history.findIndex((entry) => entry.id === commandId)

  if (index !== -1) {
    history[index] = { ...history[index], ...updates }
    await saveHistory(history)
  }
}

/**
 * Search command history
 */
export function searchHistory(
  entries: CommandHistoryEntry[],
  options: HistorySearchOptions
): CommandHistoryEntry[] {
  let results = entries

  // Filter by session if specified
  if (options.sessionId) {
    results = results.filter((entry) => entry.sessionId === options.sessionId)
  }

  // Apply search query
  if (options.query) {
    const query = options.caseSensitive ? options.query : options.query.toLowerCase()

    if (options.regex) {
      try {
        const regex = new RegExp(query, options.caseSensitive ? '' : 'i')
        results = results.filter((entry) => regex.test(entry.command))
      } catch (error) {
        console.error('Invalid regex pattern:', error)
        return []
      }
    } else {
      results = results.filter((entry) => {
        const command = options.caseSensitive ? entry.command : entry.command.toLowerCase()
        return command.includes(query)
      })
    }
  }

  // Sort by timestamp (newest first)
  results.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())

  // Apply limit
  if (options.limit && options.limit > 0) {
    results = results.slice(0, options.limit)
  }

  return results
}

/**
 * Filter command history
 */
export function filterHistory(
  entries: CommandHistoryEntry[],
  filter: HistoryFilter
): CommandHistoryEntry[] {
  let results = entries

  // Filter by session
  if (filter.sessionId) {
    results = results.filter((entry) => entry.sessionId === filter.sessionId)
  }

  // Filter by date range
  if (filter.startDate) {
    const startTime = new Date(filter.startDate).getTime()
    results = results.filter((entry) => new Date(entry.timestamp).getTime() >= startTime)
  }

  if (filter.endDate) {
    const endTime = new Date(filter.endDate).getTime()
    results = results.filter((entry) => new Date(entry.timestamp).getTime() <= endTime)
  }

  // Filter by command (partial match)
  if (filter.command) {
    const query = filter.command.toLowerCase()
    results = results.filter((entry) => entry.command.toLowerCase().includes(query))
  }

  // Filter by exit code
  if (filter.exitCode !== undefined) {
    results = results.filter((entry) => entry.exitCode === filter.exitCode)
  }

  return results
}

/**
 * Get command statistics
 */
export function getHistoryStats(entries: CommandHistoryEntry[]): {
  totalCommands: number
  uniqueCommands: number
  successfulCommands: number
  failedCommands: number
  mostUsedCommands: Array<{ command: string; count: number }>
  averageDuration: number
} {
  const uniqueCommands = new Set(entries.map((entry) => entry.command))
  const successfulCommands = entries.filter((entry) => entry.exitCode === 0)
  const failedCommands = entries.filter(
    (entry) => entry.exitCode !== undefined && entry.exitCode !== 0
  )

  // Count command frequency
  const commandCounts = new Map<string, number>()
  entries.forEach((entry) => {
    const count = commandCounts.get(entry.command) || 0
    commandCounts.set(entry.command, count + 1)
  })

  // Get most used commands
  const mostUsedCommands = Array.from(commandCounts.entries())
    .map(([command, count]) => ({ command, count }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 10)

  // Calculate average duration
  const durationsAvailable = entries.filter((entry) => entry.duration !== undefined)
  const averageDuration =
    durationsAvailable.length > 0
      ? durationsAvailable.reduce((sum, entry) => sum + (entry.duration || 0), 0) /
        durationsAvailable.length
      : 0

  return {
    totalCommands: entries.length,
    uniqueCommands: uniqueCommands.size,
    successfulCommands: successfulCommands.length,
    failedCommands: failedCommands.length,
    mostUsedCommands,
    averageDuration,
  }
}

/**
 * Export history to file
 */
export async function exportHistory(
  entries: CommandHistoryEntry[],
  format: 'json' | 'csv' | 'txt',
  filePath: string
): Promise<void> {
  try {
    let content: string

    switch (format) {
      case 'json':
        content = JSON.stringify(entries, null, 2)
        break

      case 'csv':
        const headers = 'Timestamp,Session ID,Command,Exit Code,Duration\n'
        const rows = entries
          .map((entry) =>
            [
              entry.timestamp,
              entry.sessionId,
              `"${entry.command.replace(/"/g, '""')}"`,
              entry.exitCode ?? '',
              entry.duration ?? '',
            ].join(',')
          )
          .join('\n')
        content = headers + rows
        break

      case 'txt':
        content = entries
          .map((entry) => {
            const time = new Date(entry.timestamp).toLocaleString()
            const status = entry.exitCode === 0 ? '✓' : entry.exitCode !== undefined ? '✗' : '?'
            return `[${time}] ${status} ${entry.command}`
          })
          .join('\n')
        break

      default:
        throw new Error(`Unsupported export format: ${format}`)
    }

    await writeTextFile(filePath, content, { baseDir: BaseDirectory.AppConfig })
    console.log(`Exported ${entries.length} commands to ${filePath}`)
  } catch (error) {
    console.error('Failed to export history:', error)
    throw error
  }
}

/**
 * Clear command history
 */
export async function clearHistory(): Promise<void> {
  await saveHistory([])
  console.log('Command history cleared')
}

/**
 * Auto-save command history with debouncing
 */
export class HistoryAutoSaver {
  private timeoutId: NodeJS.Timeout | null = null
  private readonly debounceMs: number
  private pendingEntries: CommandHistoryEntry[] = []

  constructor(debounceMs: number = 1000) {
    this.debounceMs = debounceMs
  }

  /**
   * Schedule an auto-save operation
   */
  scheduleSave(entries: CommandHistoryEntry[]): void {
    this.pendingEntries = entries

    if (this.timeoutId) {
      clearTimeout(this.timeoutId)
    }

    this.timeoutId = setTimeout(async () => {
      try {
        await saveHistory(this.pendingEntries)
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
   * Force immediate save
   */
  async flush(): Promise<void> {
    this.cancel()
    if (this.pendingEntries.length > 0) {
      await saveHistory(this.pendingEntries)
    }
  }
}

/**
 * Get reverse search suggestions (like Ctrl+R in bash)
 */
export function getReverseSearchSuggestions(
  entries: CommandHistoryEntry[],
  query: string,
  limit: number = 10
): CommandHistoryEntry[] {
  if (!query) return []

  const lowerQuery = query.toLowerCase()
  const matches = entries.filter((entry) => entry.command.toLowerCase().includes(lowerQuery))

  // Remove duplicates (keep most recent)
  const unique = new Map<string, CommandHistoryEntry>()
  matches.reverse().forEach((entry) => {
    if (!unique.has(entry.command)) {
      unique.set(entry.command, entry)
    }
  })

  return Array.from(unique.values()).slice(0, limit)
}
