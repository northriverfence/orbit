import { useState, useEffect, useRef, useMemo } from 'react'
import { useKeyboardShortcut } from '../hooks/useKeyboardShortcut'
import { useFocusTrap } from '../hooks/useFocusTrap'
import { useArrowNavigation } from '../hooks/useArrowNavigation'

export interface Command {
  id: string
  label: string
  description?: string
  icon?: string
  keywords?: string[]
  action: () => void
  category?: string
}

interface CommandPaletteProps {
  commands: Command[]
  isOpen: boolean
  onClose: () => void
}

export default function CommandPalette({ commands, isOpen, onClose }: CommandPaletteProps) {
  const [searchQuery, setSearchQuery] = useState('')
  const dialogRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLInputElement>(null)
  const listRef = useRef<HTMLDivElement>(null)

  // Filter commands based on search query
  const filteredCommands = useMemo(() => {
    if (!searchQuery.trim()) return commands

    const query = searchQuery.toLowerCase()
    return commands.filter((cmd) => {
      const labelMatch = cmd.label.toLowerCase().includes(query)
      const descMatch = cmd.description?.toLowerCase().includes(query)
      const keywordMatch = cmd.keywords?.some((kw) => kw.toLowerCase().includes(query))
      return labelMatch || descMatch || keywordMatch
    })
  }, [commands, searchQuery])

  // Group commands by category
  const groupedCommands = useMemo(() => {
    const groups: Record<string, Command[]> = {}

    filteredCommands.forEach((cmd) => {
      const category = cmd.category || 'Other'
      if (!groups[category]) {
        groups[category] = []
      }
      groups[category].push(cmd)
    })

    return groups
  }, [filteredCommands])

  // Arrow navigation for command list
  const { activeIndex, setActiveIndex } = useArrowNavigation({
    containerRef: listRef,
    enabled: isOpen && filteredCommands.length > 0,
    onSelect: (index) => {
      if (filteredCommands[index]) {
        executeCommand(filteredCommands[index])
      }
    },
    loop: true,
  })

  // Focus management
  useFocusTrap(dialogRef, isOpen)
  useKeyboardShortcut({ key: 'Escape' }, handleClose, isOpen)

  // Reset state when dialog opens
  useEffect(() => {
    if (isOpen) {
      setSearchQuery('')
      setActiveIndex(0)
      // Focus the input after a brief delay to ensure dialog is rendered
      setTimeout(() => {
        inputRef.current?.focus()
      }, 50)
    }
  }, [isOpen, setActiveIndex])

  function handleClose() {
    setSearchQuery('')
    onClose()
  }

  function executeCommand(command: Command) {
    command.action()
    handleClose()
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    // Prevent arrow keys from moving cursor in input
    if (['ArrowUp', 'ArrowDown', 'Home', 'End'].includes(e.key)) {
      e.preventDefault()
    }
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-60 flex items-start justify-center z-50 pt-[15vh] modal-backdrop">
      <div
        ref={dialogRef}
        className="bg-white rounded-lg shadow-2xl w-full max-w-2xl overflow-hidden modal-content"
      >
        {/* Search Input */}
        <div className="p-4 border-b border-gray-200">
          <div className="relative">
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg
                className="w-5 h-5 text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
            </div>
            <input
              ref={inputRef}
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Type a command or search..."
              className="w-full pl-10 pr-4 py-3 text-lg border-0 focus:outline-none focus:ring-0"
              autoComplete="off"
              spellCheck="false"
            />
          </div>
        </div>

        {/* Command List */}
        <div
          ref={listRef}
          className="max-h-[400px] overflow-y-auto"
          role="listbox"
        >
          {filteredCommands.length === 0 ? (
            <div className="p-8 text-center text-gray-500">
              <div className="text-4xl mb-3">🔍</div>
              <p className="text-sm font-medium">No commands found</p>
              <p className="text-xs mt-1">Try a different search term</p>
            </div>
          ) : (
            Object.entries(groupedCommands).map(([category, categoryCommands]) => (
              <div key={category}>
                {/* Category Header */}
                <div className="px-4 py-2 bg-gray-50 border-b border-gray-200">
                  <h3 className="text-xs font-semibold text-gray-600 uppercase tracking-wide">
                    {category}
                  </h3>
                </div>

                {/* Commands in Category */}
                {categoryCommands.map((command) => {
                  const globalIndex = filteredCommands.indexOf(command)
                  const isActive = globalIndex === activeIndex

                  return (
                    <button
                      key={command.id}
                      role="option"
                      aria-selected={isActive}
                      tabIndex={0}
                      onClick={() => executeCommand(command)}
                      className={`
                        w-full px-4 py-3 flex items-center justify-between
                        hover:bg-gray-100 transition-colors text-left
                        border-b border-gray-100 last:border-0
                        ${isActive ? 'bg-blue-50 border-l-4 border-l-blue-500' : ''}
                      `}
                    >
                      <div className="flex items-center space-x-3 flex-1">
                        {command.icon && (
                          <span className="text-2xl flex-shrink-0">{command.icon}</span>
                        )}
                        <div className="flex-1 min-w-0">
                          <div className="font-medium text-gray-900 truncate">
                            {command.label}
                          </div>
                          {command.description && (
                            <div className="text-sm text-gray-500 truncate">
                              {command.description}
                            </div>
                          )}
                        </div>
                      </div>

                      {/* Enter hint for active item */}
                      {isActive && (
                        <div className="ml-4 flex-shrink-0">
                          <kbd className="px-2 py-1 text-xs font-mono bg-gray-200 text-gray-700 rounded">
                            Enter
                          </kbd>
                        </div>
                      )}
                    </button>
                  )
                })}
              </div>
            ))
          )}
        </div>

        {/* Footer with hints */}
        <div className="px-4 py-2 bg-gray-50 border-t border-gray-200 flex items-center justify-between text-xs text-gray-600">
          <div className="flex items-center space-x-4">
            <span className="flex items-center space-x-1">
              <kbd className="px-1.5 py-0.5 bg-white border border-gray-300 rounded font-mono">↑↓</kbd>
              <span>Navigate</span>
            </span>
            <span className="flex items-center space-x-1">
              <kbd className="px-1.5 py-0.5 bg-white border border-gray-300 rounded font-mono">Enter</kbd>
              <span>Select</span>
            </span>
            <span className="flex items-center space-x-1">
              <kbd className="px-1.5 py-0.5 bg-white border border-gray-300 rounded font-mono">Esc</kbd>
              <span>Close</span>
            </span>
          </div>
          <div className="text-gray-500">
            {filteredCommands.length} {filteredCommands.length === 1 ? 'command' : 'commands'}
          </div>
        </div>
      </div>
    </div>
  )
}
