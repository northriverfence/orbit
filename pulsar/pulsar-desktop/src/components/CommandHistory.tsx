/**
 * Command History Component
 *
 * Displays command history with search, filter, and re-execution capabilities.
 */

import { useState, useCallback, useEffect, useMemo } from 'react'
import HistorySearch from './HistorySearch'
import type { CommandHistoryEntry } from '../lib/commandCapture'
import { formatCommandForDisplay } from '../lib/commandCapture'
import {
  loadHistory,
  searchHistory,
  getHistoryStats,
  exportHistory,
  clearHistory,
  type HistorySearchOptions,
} from '../lib/historyStorage'

interface CommandHistoryProps {
  sessionId?: string // If provided, show only this session's history
  onExecuteCommand?: (command: string) => void
  onClose?: () => void
}

export default function CommandHistory({
  sessionId,
  onExecuteCommand,
  onClose,
}: CommandHistoryProps) {
  const [history, setHistory] = useState<CommandHistoryEntry[]>([])
  const [filteredHistory, setFilteredHistory] = useState<CommandHistoryEntry[]>([])
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedFilter, setSelectedFilter] = useState<'all' | 'success' | 'failed' | 'session'>(
    sessionId ? 'session' : 'all'
  )
  const [isLoading, setIsLoading] = useState(true)
  const [selectedEntry, setSelectedEntry] = useState<CommandHistoryEntry | null>(null)
  const [showStats, setShowStats] = useState(false)

  // Load history on mount
  useEffect(() => {
    const loadCommandHistory = async () => {
      try {
        const entries = await loadHistory()
        setHistory(entries)
        setFilteredHistory(entries)
      } catch (error) {
        console.error('Failed to load history:', error)
      } finally {
        setIsLoading(false)
      }
    }

    loadCommandHistory()
  }, [])

  // Apply filters and search
  useEffect(() => {
    let filtered = history

    // Apply session filter
    if (selectedFilter === 'session' && sessionId) {
      filtered = filtered.filter((entry) => entry.sessionId === sessionId)
    } else if (selectedFilter === 'success') {
      filtered = filtered.filter((entry) => entry.exitCode === 0)
    } else if (selectedFilter === 'failed') {
      filtered = filtered.filter((entry) => entry.exitCode !== undefined && entry.exitCode !== 0)
    }

    // Apply search
    if (searchQuery) {
      const options: HistorySearchOptions = {
        query: searchQuery,
        caseSensitive: false,
      }
      filtered = searchHistory(filtered, options)
    }

    setFilteredHistory(filtered)
  }, [history, searchQuery, selectedFilter, sessionId])

  // Calculate statistics
  const stats = useMemo(() => getHistoryStats(filteredHistory), [filteredHistory])

  // Handle command execution
  const handleExecute = useCallback(
    (command: string) => {
      if (onExecuteCommand) {
        onExecuteCommand(command)
      }
    },
    [onExecuteCommand]
  )

  // Handle export
  const handleExport = useCallback(
    async (format: 'json' | 'csv' | 'txt') => {
      try {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-')
        const filename = `command_history_${timestamp}.${format}`
        await exportHistory(filteredHistory, format, filename)
        alert(`Exported ${filteredHistory.length} commands to ${filename}`)
      } catch (error) {
        console.error('Export failed:', error)
        alert('Export failed. See console for details.')
      }
    },
    [filteredHistory]
  )

  // Handle clear history
  const handleClear = useCallback(async () => {
    if (confirm('Are you sure you want to clear all command history? This cannot be undone.')) {
      try {
        await clearHistory()
        setHistory([])
        setFilteredHistory([])
      } catch (error) {
        console.error('Clear failed:', error)
        alert('Clear failed. See console for details.')
      }
    }
  }, [])

  // Format timestamp
  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMins / 60)
    const diffDays = Math.floor(diffHours / 24)

    if (diffMins < 1) return 'just now'
    if (diffMins < 60) return `${diffMins}m ago`
    if (diffHours < 24) return `${diffHours}h ago`
    if (diffDays < 7) return `${diffDays}d ago`

    return date.toLocaleDateString()
  }

  // Format duration
  const formatDuration = (duration?: number) => {
    if (!duration) return ''
    if (duration < 1000) return `${duration}ms`
    return `${(duration / 1000).toFixed(1)}s`
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-gray-500">Loading command history...</div>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full bg-white">
      {/* Header */}
      <div className="flex-shrink-0 border-b border-gray-200 bg-gray-50 px-4 py-3">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-lg font-semibold text-gray-900">Command History</h2>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setShowStats(!showStats)}
              className="px-3 py-1 text-sm bg-blue-100 text-blue-700 rounded hover:bg-blue-200 transition-colors"
            >
              {showStats ? 'Hide Stats' : 'Show Stats'}
            </button>
            {onClose && (
              <button
                onClick={onClose}
                className="p-1 hover:bg-gray-200 rounded transition-colors"
                title="Close"
              >
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            )}
          </div>
        </div>

        {/* Search and Filters */}
        <HistorySearch
          query={searchQuery}
          onQueryChange={setSearchQuery}
          onClear={() => setSearchQuery('')}
        />

        <div className="flex items-center gap-2 mt-3">
          <button
            onClick={() => setSelectedFilter('all')}
            className={`px-3 py-1 text-sm rounded transition-colors ${
              selectedFilter === 'all'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
            }`}
          >
            All ({history.length})
          </button>
          <button
            onClick={() => setSelectedFilter('success')}
            className={`px-3 py-1 text-sm rounded transition-colors ${
              selectedFilter === 'success'
                ? 'bg-green-600 text-white'
                : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
            }`}
          >
            Success ({history.filter((e) => e.exitCode === 0).length})
          </button>
          <button
            onClick={() => setSelectedFilter('failed')}
            className={`px-3 py-1 text-sm rounded transition-colors ${
              selectedFilter === 'failed'
                ? 'bg-red-600 text-white'
                : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
            }`}
          >
            Failed ({history.filter((e) => e.exitCode && e.exitCode !== 0).length})
          </button>
          {sessionId && (
            <button
              onClick={() => setSelectedFilter('session')}
              className={`px-3 py-1 text-sm rounded transition-colors ${
                selectedFilter === 'session'
                  ? 'bg-purple-600 text-white'
                  : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
              }`}
            >
              This Session ({history.filter((e) => e.sessionId === sessionId).length})
            </button>
          )}
        </div>
      </div>

      {/* Statistics Panel */}
      {showStats && (
        <div className="flex-shrink-0 border-b border-gray-200 bg-blue-50 px-4 py-3">
          <div className="grid grid-cols-4 gap-4 text-sm">
            <div>
              <div className="text-gray-600">Total Commands</div>
              <div className="text-xl font-bold text-gray-900">{stats.totalCommands}</div>
            </div>
            <div>
              <div className="text-gray-600">Unique Commands</div>
              <div className="text-xl font-bold text-gray-900">{stats.uniqueCommands}</div>
            </div>
            <div>
              <div className="text-gray-600">Success Rate</div>
              <div className="text-xl font-bold text-green-600">
                {stats.totalCommands > 0
                  ? ((stats.successfulCommands / stats.totalCommands) * 100).toFixed(1)
                  : 0}
                %
              </div>
            </div>
            <div>
              <div className="text-gray-600">Avg Duration</div>
              <div className="text-xl font-bold text-gray-900">
                {formatDuration(stats.averageDuration)}
              </div>
            </div>
          </div>

          {/* Most Used Commands */}
          {stats.mostUsedCommands.length > 0 && (
            <div className="mt-3">
              <div className="text-xs text-gray-600 mb-2">Most Used Commands:</div>
              <div className="flex flex-wrap gap-2">
                {stats.mostUsedCommands.slice(0, 5).map((cmd) => (
                  <span
                    key={cmd.command}
                    className="px-2 py-1 bg-white rounded text-xs text-gray-700 border border-gray-300"
                  >
                    {cmd.command.substring(0, 20)}
                    {cmd.command.length > 20 ? '...' : ''} ({cmd.count})
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Command List */}
      <div className="flex-1 overflow-y-auto">
        {filteredHistory.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-500">
            <div className="text-4xl mb-2">📜</div>
            <div className="text-lg">No commands found</div>
            {searchQuery && <div className="text-sm">Try a different search query</div>}
          </div>
        ) : (
          <div className="divide-y divide-gray-200">
            {filteredHistory.map((entry) => {
              const formatted = formatCommandForDisplay(entry.command)
              const isSelected = selectedEntry?.id === entry.id

              return (
                <div
                  key={entry.id}
                  onClick={() => setSelectedEntry(isSelected ? null : entry)}
                  className={`px-4 py-3 hover:bg-gray-50 cursor-pointer transition-colors ${
                    isSelected ? 'bg-blue-50 border-l-4 border-blue-600' : ''
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        {/* Status Icon */}
                        {entry.exitCode === 0 ? (
                          <span className="text-green-600" title="Success">
                            ✓
                          </span>
                        ) : entry.exitCode !== undefined ? (
                          <span className="text-red-600" title={`Exit code: ${entry.exitCode}`}>
                            ✗
                          </span>
                        ) : (
                          <span className="text-gray-400" title="Unknown status">
                            ?
                          </span>
                        )}

                        {/* Command */}
                        <code className="text-sm font-mono text-gray-900 truncate">
                          <span className="font-bold text-blue-600">
                            {formatted.commandName}
                          </span>
                          {formatted.args && <span className="text-gray-600"> {formatted.args}</span>}
                        </code>

                        {/* Tags */}
                        {formatted.metadata.tags.map((tag) => (
                          <span
                            key={tag}
                            className={`px-1.5 py-0.5 text-xs rounded ${
                              tag === 'dangerous'
                                ? 'bg-red-100 text-red-700'
                                : tag === 'root'
                                ? 'bg-yellow-100 text-yellow-700'
                                : 'bg-gray-100 text-gray-600'
                            }`}
                          >
                            {tag}
                          </span>
                        ))}
                      </div>

                      <div className="flex items-center gap-3 text-xs text-gray-500">
                        <span>{formatTimestamp(entry.timestamp)}</span>
                        {entry.duration && <span>{formatDuration(entry.duration)}</span>}
                        <span className="text-gray-400">Session: {entry.sessionId.slice(0, 8)}</span>
                      </div>
                    </div>

                    {/* Actions */}
                    <div className="flex items-center gap-1 ml-2">
                      {onExecuteCommand && (
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            handleExecute(entry.command)
                          }}
                          className="p-1.5 hover:bg-blue-100 rounded transition-colors text-blue-600"
                          title="Execute command"
                        >
                          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth={2}
                              d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
                            />
                            <path
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth={2}
                              d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                            />
                          </svg>
                        </button>
                      )}
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          navigator.clipboard.writeText(entry.command)
                        }}
                        className="p-1.5 hover:bg-gray-200 rounded transition-colors text-gray-600"
                        title="Copy to clipboard"
                      >
                        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                          />
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </div>

      {/* Footer Actions */}
      <div className="flex-shrink-0 border-t border-gray-200 bg-gray-50 px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="text-sm text-gray-600">
            Showing {filteredHistory.length} of {history.length} commands
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => handleExport('json')}
              className="px-3 py-1.5 text-sm bg-gray-200 text-gray-700 rounded hover:bg-gray-300 transition-colors"
            >
              Export JSON
            </button>
            <button
              onClick={() => handleExport('csv')}
              className="px-3 py-1.5 text-sm bg-gray-200 text-gray-700 rounded hover:bg-gray-300 transition-colors"
            >
              Export CSV
            </button>
            <button
              onClick={handleClear}
              className="px-3 py-1.5 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
            >
              Clear History
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
