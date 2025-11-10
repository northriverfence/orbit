import { useRef } from 'react'
import { useFocusTrap } from '../hooks/useFocusTrap'
import { useKeyboardShortcut, SHORTCUTS } from '../hooks/useKeyboardShortcut'

interface KeyboardShortcutsDialogProps {
  isOpen: boolean
  onClose: () => void
}

interface ShortcutGroup {
  category: string
  shortcuts: ShortcutItem[]
}

interface ShortcutItem {
  keys: string[]
  description: string
  context?: string
}

const shortcutGroups: ShortcutGroup[] = [
  {
    category: 'Global',
    shortcuts: [
      {
        keys: ['Ctrl', 'K'],
        description: 'Open Command Palette',
      },
      {
        keys: ['Ctrl', ','],
        description: 'Open Settings',
      },
      {
        keys: ['?'],
        description: 'Show Keyboard Shortcuts',
      },
    ],
  },
  {
    category: 'Modals & Dialogs',
    shortcuts: [
      {
        keys: ['Escape'],
        description: 'Close Dialog',
        context: 'Any open dialog',
      },
      {
        keys: ['Ctrl', 'Enter'],
        description: 'Submit/Connect',
        context: 'Connection dialog',
      },
      {
        keys: ['Ctrl', 'S'],
        description: 'Save Settings',
        context: 'Settings dialog',
      },
    ],
  },
  {
    category: 'Navigation',
    shortcuts: [
      {
        keys: ['↑', '↓'],
        description: 'Navigate Items',
        context: 'Command palette, lists',
      },
      {
        keys: ['Home'],
        description: 'Jump to First Item',
        context: 'Lists',
      },
      {
        keys: ['End'],
        description: 'Jump to Last Item',
        context: 'Lists',
      },
      {
        keys: ['Enter'],
        description: 'Select Item',
        context: 'Command palette, lists',
      },
      {
        keys: ['Tab'],
        description: 'Next Field',
        context: 'Forms and dialogs',
      },
      {
        keys: ['Shift', 'Tab'],
        description: 'Previous Field',
        context: 'Forms and dialogs',
      },
    ],
  },
  {
    category: 'Tabs (Coming Soon)',
    shortcuts: [
      {
        keys: ['Ctrl', 'T'],
        description: 'New Tab',
      },
      {
        keys: ['Ctrl', 'W'],
        description: 'Close Tab',
      },
      {
        keys: ['Ctrl', 'Tab'],
        description: 'Next Tab',
      },
      {
        keys: ['Ctrl', 'Shift', 'Tab'],
        description: 'Previous Tab',
      },
    ],
  },
  {
    category: 'Search (Coming Soon)',
    shortcuts: [
      {
        keys: ['Ctrl', 'F'],
        description: 'Find in Page',
      },
      {
        keys: ['Ctrl', 'P'],
        description: 'Quick Switcher',
      },
    ],
  },
]

export default function KeyboardShortcutsDialog({
  isOpen,
  onClose,
}: KeyboardShortcutsDialogProps) {
  const dialogRef = useRef<HTMLDivElement>(null)

  // Keyboard navigation
  useFocusTrap(dialogRef, isOpen)
  useKeyboardShortcut(SHORTCUTS.ESCAPE, onClose, isOpen)

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-60 flex items-center justify-center z-50 modal-backdrop">
      <div
        ref={dialogRef}
        className="bg-white rounded-lg shadow-2xl w-full max-w-3xl max-h-[85vh] flex flex-col modal-content"
      >
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <div>
            <h2 className="text-2xl font-bold text-gray-900 flex items-center space-x-2">
              <span>⌨️</span>
              <span>Keyboard Shortcuts</span>
            </h2>
            <p className="text-sm text-gray-600 mt-1">
              Master Pulsar with these keyboard shortcuts
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
            aria-label="Close"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        {/* Shortcuts List */}
        <div className="flex-1 overflow-y-auto p-6">
          <div className="space-y-8">
            {shortcutGroups.map((group) => (
              <div key={group.category} className="animate-fadeIn">
                {/* Category Header */}
                <div className="mb-4">
                  <h3 className="text-lg font-bold text-gray-900 border-b-2 border-blue-500 pb-2">
                    {group.category}
                  </h3>
                </div>

                {/* Shortcuts in Category */}
                <div className="space-y-3">
                  {group.shortcuts.map((shortcut, index) => (
                    <div
                      key={index}
                      className="flex items-center justify-between py-2 px-3 rounded hover:bg-gray-50 transition-colors"
                    >
                      {/* Description */}
                      <div className="flex-1">
                        <div className="font-medium text-gray-900">
                          {shortcut.description}
                        </div>
                        {shortcut.context && (
                          <div className="text-xs text-gray-500 mt-0.5">
                            {shortcut.context}
                          </div>
                        )}
                      </div>

                      {/* Key Combination */}
                      <div className="flex items-center space-x-1 ml-4">
                        {shortcut.keys.map((key, keyIndex) => (
                          <div key={keyIndex} className="flex items-center space-x-1">
                            <kbd className="px-3 py-1.5 text-sm font-mono font-semibold bg-gray-100 text-gray-800 border border-gray-300 rounded shadow-sm">
                              {key}
                            </kbd>
                            {keyIndex < shortcut.keys.length - 1 && (
                              <span className="text-gray-400 text-sm font-medium">+</span>
                            )}
                          </div>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 bg-gray-50 border-t border-gray-200 rounded-b-lg">
          <div className="flex items-center justify-between">
            <div className="text-sm text-gray-600">
              <span className="font-medium">Pro Tip:</span> Press{' '}
              <kbd className="px-2 py-0.5 text-xs font-mono bg-white border border-gray-300 rounded">
                ?
              </kbd>{' '}
              anytime to open this dialog
            </div>
            <button
              onClick={onClose}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors font-medium btn-press"
            >
              Got it!
            </button>
          </div>
        </div>

        {/* Platform Note */}
        <div className="px-6 py-3 bg-blue-50 border-t border-blue-100 text-xs text-blue-800 rounded-b-lg">
          <div className="flex items-start space-x-2">
            <svg className="w-4 h-4 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
              <path
                fillRule="evenodd"
                d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                clipRule="evenodd"
              />
            </svg>
            <div>
              <strong>Note:</strong> On macOS, use <strong>Cmd</strong> instead of{' '}
              <strong>Ctrl</strong> for most shortcuts. The application automatically adapts to
              your operating system.
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
