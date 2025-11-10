/**
 * History Search Component
 *
 * Search bar for filtering command history
 */

import { useState, useCallback, useEffect, useRef } from 'react'

interface HistorySearchProps {
  query: string
  onQueryChange: (query: string) => void
  onClear: () => void
  placeholder?: string
  caseSensitive?: boolean
  onCaseSensitiveChange?: (value: boolean) => void
  regex?: boolean
  onRegexChange?: (value: boolean) => void
}

export default function HistorySearch({
  query,
  onQueryChange,
  onClear,
  placeholder = 'Search commands... (Ctrl+R for reverse search)',
  caseSensitive = false,
  onCaseSensitiveChange,
  regex = false,
  onRegexChange,
}: HistorySearchProps) {
  const [isFocused, setIsFocused] = useState(false)
  const [showOptions, setShowOptions] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  // Focus input on Ctrl+R (reverse search)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === 'r') {
        e.preventDefault()
        inputRef.current?.focus()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onQueryChange(e.target.value)
    },
    [onQueryChange]
  )

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Escape') {
        onClear()
        inputRef.current?.blur()
      }
    },
    [onClear]
  )

  return (
    <div className="relative">
      <div
        className={`flex items-center gap-2 px-3 py-2 border rounded-lg transition-all ${
          isFocused
            ? 'border-blue-500 ring-2 ring-blue-200 bg-white'
            : 'border-gray-300 bg-gray-50'
        }`}
      >
        {/* Search Icon */}
        <svg
          className="w-4 h-4 text-gray-400"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
          />
        </svg>

        {/* Input */}
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          onFocus={() => setIsFocused(true)}
          onBlur={() => setIsFocused(false)}
          placeholder={placeholder}
          className="flex-1 bg-transparent border-none outline-none text-sm text-gray-900 placeholder-gray-400"
        />

        {/* Clear Button */}
        {query && (
          <button
            onClick={onClear}
            className="p-1 hover:bg-gray-200 rounded transition-colors"
            title="Clear search (Esc)"
          >
            <svg className="w-4 h-4 text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        )}

        {/* Options Toggle */}
        {(onCaseSensitiveChange || onRegexChange) && (
          <button
            onClick={() => setShowOptions(!showOptions)}
            className="p-1 hover:bg-gray-200 rounded transition-colors"
            title="Search options"
          >
            <svg className="w-4 h-4 text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"
              />
            </svg>
          </button>
        )}
      </div>

      {/* Search Options Dropdown */}
      {showOptions && (onCaseSensitiveChange || onRegexChange) && (
        <div className="absolute top-full left-0 right-0 mt-1 p-2 bg-white border border-gray-200 rounded-lg shadow-lg z-10">
          {onCaseSensitiveChange && (
            <label className="flex items-center gap-2 p-2 hover:bg-gray-50 rounded cursor-pointer">
              <input
                type="checkbox"
                checked={caseSensitive}
                onChange={(e) => onCaseSensitiveChange(e.target.checked)}
                className="w-4 h-4 text-blue-600 rounded border-gray-300 focus:ring-2 focus:ring-blue-500"
              />
              <span className="text-sm text-gray-700">Case sensitive</span>
              <kbd className="ml-auto px-1.5 py-0.5 text-xs bg-gray-100 rounded border border-gray-300">
                Aa
              </kbd>
            </label>
          )}

          {onRegexChange && (
            <label className="flex items-center gap-2 p-2 hover:bg-gray-50 rounded cursor-pointer">
              <input
                type="checkbox"
                checked={regex}
                onChange={(e) => onRegexChange(e.target.checked)}
                className="w-4 h-4 text-blue-600 rounded border-gray-300 focus:ring-2 focus:ring-blue-500"
              />
              <span className="text-sm text-gray-700">Regular expression</span>
              <kbd className="ml-auto px-1.5 py-0.5 text-xs bg-gray-100 rounded border border-gray-300">
                .*
              </kbd>
            </label>
          )}

          <div className="mt-2 pt-2 border-t border-gray-200">
            <div className="text-xs text-gray-500 px-2">
              <div className="font-semibold mb-1">Search Tips:</div>
              <ul className="space-y-0.5">
                <li>• Press <kbd className="px-1 bg-gray-100 rounded">Ctrl+R</kbd> to focus search</li>
                <li>• Press <kbd className="px-1 bg-gray-100 rounded">Esc</kbd> to clear</li>
                <li>• Use regex for advanced patterns</li>
              </ul>
            </div>
          </div>
        </div>
      )}

      {/* Search Status */}
      {query && (
        <div className="flex items-center gap-2 mt-1 text-xs text-gray-500">
          <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <span>
            Searching for: <code className="px-1 bg-gray-100 rounded">{query}</code>
          </span>
          {caseSensitive && <span className="px-1.5 py-0.5 bg-blue-100 text-blue-700 rounded">Case sensitive</span>}
          {regex && <span className="px-1.5 py-0.5 bg-purple-100 text-purple-700 rounded">Regex</span>}
        </div>
      )}
    </div>
  )
}
