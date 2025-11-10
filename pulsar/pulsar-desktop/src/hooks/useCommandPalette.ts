import { useState, useCallback } from 'react'
import { useKeyboardShortcut } from './useKeyboardShortcut'
import type { Command } from '../components/CommandPalette'

interface UseCommandPaletteOptions {
  commands?: Command[]
}

export function useCommandPalette(options: UseCommandPaletteOptions = {}) {
  const [isOpen, setIsOpen] = useState(false)
  const [customCommands, setCustomCommands] = useState<Command[]>(options.commands || [])

  // Open the command palette
  const open = useCallback(() => {
    setIsOpen(true)
  }, [])

  // Close the command palette
  const close = useCallback(() => {
    setIsOpen(false)
  }, [])

  // Toggle the command palette
  const toggle = useCallback(() => {
    setIsOpen((prev) => !prev)
  }, [])

  // Register a new command
  const registerCommand = useCallback((command: Command) => {
    setCustomCommands((prev) => {
      // Remove existing command with same ID
      const filtered = prev.filter((cmd) => cmd.id !== command.id)
      return [...filtered, command]
    })
  }, [])

  // Unregister a command
  const unregisterCommand = useCallback((commandId: string) => {
    setCustomCommands((prev) => prev.filter((cmd) => cmd.id !== commandId))
  }, [])

  // Register multiple commands
  const registerCommands = useCallback((commands: Command[]) => {
    setCustomCommands((prev) => {
      const existingIds = new Set(commands.map((cmd) => cmd.id))
      const filtered = prev.filter((cmd) => !existingIds.has(cmd.id))
      return [...filtered, ...commands]
    })
  }, [])

  // Clear all custom commands
  const clearCommands = useCallback(() => {
    setCustomCommands([])
  }, [])

  // Keyboard shortcut to open command palette (Ctrl/Cmd+K)
  useKeyboardShortcut(
    [
      { key: 'k', ctrlKey: true, preventDefault: true },
      { key: 'k', metaKey: true, preventDefault: true },
    ],
    toggle,
    true
  )

  return {
    isOpen,
    open,
    close,
    toggle,
    commands: customCommands,
    registerCommand,
    unregisterCommand,
    registerCommands,
    clearCommands,
  }
}

// Helper to create command objects
export function createCommand(
  id: string,
  label: string,
  action: () => void,
  options?: {
    description?: string
    icon?: string
    keywords?: string[]
    category?: string
  }
): Command {
  return {
    id,
    label,
    action,
    description: options?.description,
    icon: options?.icon,
    keywords: options?.keywords || [],
    category: options?.category || 'Other',
  }
}
