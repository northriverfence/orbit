/**
 * Command Capture Module
 *
 * Captures and tracks commands executed in terminal sessions.
 * Integrates with xterm.js to intercept command input.
 */

import type { Terminal } from '@xterm/xterm'

export interface CommandHistoryEntry {
  id: string
  sessionId: string
  command: string
  timestamp: string
  exitCode?: number
  duration?: number // milliseconds
  workingDirectory?: string
  hostname?: string
}

/**
 * Command Capture Handler
 *
 * Attaches to a terminal instance to capture commands
 */
export class CommandCaptureHandler {
  private terminal: Terminal
  private sessionId: string
  private currentCommand: string = ''
  private commandStartTime: number | null = null
  private onCommandExecuted: (entry: CommandHistoryEntry) => void

  constructor(
    terminal: Terminal,
    sessionId: string,
    onCommandExecuted: (entry: CommandHistoryEntry) => void
  ) {
    this.terminal = terminal
    this.sessionId = sessionId
    this.onCommandExecuted = onCommandExecuted
  }

  /**
   * Start capturing commands
   */
  startCapture(): void {
    this.terminal.onData((data) => {
      this.handleTerminalData(data)
    })
  }

  /**
   * Handle terminal data input
   */
  private handleTerminalData(data: string): void {
    // Check for Enter key (command execution)
    if (data === '\r' || data === '\n') {
      if (this.currentCommand.trim().length > 0) {
        this.recordCommand(this.currentCommand.trim())
        this.currentCommand = ''
      }
      return
    }

    // Check for backspace
    if (data === '\x7f') {
      this.currentCommand = this.currentCommand.slice(0, -1)
      return
    }

    // Check for Ctrl+C (cancel command)
    if (data === '\x03') {
      this.currentCommand = ''
      return
    }

    // Append to current command
    this.currentCommand += data
  }

  /**
   * Record a command to history
   */
  private recordCommand(command: string): void {
    const entry: CommandHistoryEntry = {
      id: `cmd-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`,
      sessionId: this.sessionId,
      command,
      timestamp: new Date().toISOString(),
    }

    this.commandStartTime = Date.now()
    this.onCommandExecuted(entry)
  }

  /**
   * Update the last command with exit code and duration
   */
  updateLastCommand(_exitCode: number): void {
    if (this.commandStartTime) {
      // Calculate duration for potential callback
      // (Implementation in historyStorage.ts)
      this.commandStartTime = null
    }
  }
}

/**
 * Parse command line to extract command name and arguments
 */
export function parseCommand(commandLine: string): {
  command: string
  args: string[]
  rawArgs: string
} {
  const trimmed = commandLine.trim()
  const parts = trimmed.split(/\s+/)

  return {
    command: parts[0] || '',
    args: parts.slice(1),
    rawArgs: parts.slice(1).join(' '),
  }
}

/**
 * Check if command is a shell builtin
 */
export function isBuiltinCommand(command: string): boolean {
  const builtins = [
    'cd',
    'pwd',
    'echo',
    'export',
    'alias',
    'unalias',
    'source',
    '.',
    'exit',
    'logout',
    'history',
    'set',
    'unset',
  ]

  return builtins.includes(command)
}

/**
 * Check if command is likely to be long-running
 */
export function isLongRunningCommand(command: string): boolean {
  const longRunning = [
    'npm',
    'yarn',
    'pnpm',
    'cargo',
    'make',
    'docker',
    'kubectl',
    'terraform',
    'ansible',
    'ssh',
    'scp',
    'rsync',
    'git',
    'vim',
    'nano',
    'emacs',
    'watch',
    'tail',
    'less',
    'more',
    'top',
    'htop',
  ]

  return longRunning.some((cmd) => command.startsWith(cmd))
}

/**
 * Sanitize command for storage (remove sensitive data)
 */
export function sanitizeCommand(command: string): string {
  // Remove passwords from common commands
  let sanitized = command

  // SSH password
  sanitized = sanitized.replace(/(ssh\s+.*-p\s+)(\S+)/, '$1***')

  // MySQL/PostgreSQL password
  sanitized = sanitized.replace(/(-p|--password=)(\S+)/, '$1***')

  // Environment variables with PASSWORD
  sanitized = sanitized.replace(/(PASSWORD=)(\S+)/, '$1***')

  // Generic token patterns
  sanitized = sanitized.replace(/(token=|key=|secret=)(\S+)/gi, '$1***')

  return sanitized
}

/**
 * Extract command metadata
 */
export function extractCommandMetadata(command: string): {
  isRoot: boolean
  isDangerous: boolean
  isSafe: boolean
  tags: string[]
} {
  const parsed = parseCommand(command)
  const cmd = parsed.command

  const isDangerous =
    cmd === 'rm' ||
    cmd === 'rmdir' ||
    cmd === 'dd' ||
    cmd === 'mkfs' ||
    cmd === 'fdisk' ||
    command.includes('sudo rm') ||
    command.includes('sudo dd')

  const isRoot = command.startsWith('sudo') || command.startsWith('su ')

  const isSafe =
    cmd === 'ls' ||
    cmd === 'pwd' ||
    cmd === 'echo' ||
    cmd === 'cat' ||
    cmd === 'grep' ||
    cmd === 'find' ||
    cmd === 'which' ||
    cmd === 'whereis'

  const tags: string[] = []
  if (isRoot) tags.push('root')
  if (isDangerous) tags.push('dangerous')
  if (isSafe) tags.push('safe')
  if (isBuiltinCommand(cmd)) tags.push('builtin')
  if (isLongRunningCommand(cmd)) tags.push('long-running')

  return { isRoot, isDangerous, isSafe, tags }
}

/**
 * Format command for display (with syntax highlighting hints)
 */
export function formatCommandForDisplay(command: string): {
  command: string
  commandName: string
  args: string
  metadata: ReturnType<typeof extractCommandMetadata>
} {
  const parsed = parseCommand(command)
  const metadata = extractCommandMetadata(command)

  return {
    command,
    commandName: parsed.command,
    args: parsed.rawArgs,
    metadata,
  }
}
