import { useState } from 'react'
import type { KeyboardShortcuts } from '../../types/settings'

interface ShortcutsTabProps {
  settings: KeyboardShortcuts
  onChange: (settings: KeyboardShortcuts) => void
}

type ShortcutKey = keyof KeyboardShortcuts

export default function ShortcutsTab({ settings, onChange }: ShortcutsTabProps) {
  const [recordingKey, setRecordingKey] = useState<ShortcutKey | null>(null)
  const [searchQuery, setSearchQuery] = useState('')

  const shortcuts: Array<{
    key: ShortcutKey
    label: string
    category: string
    description: string
  }> = [
    { key: 'new_tab', label: 'New Tab', category: 'Tabs', description: 'Open a new terminal tab' },
    { key: 'close_tab', label: 'Close Tab', category: 'Tabs', description: 'Close the current tab' },
    { key: 'next_tab', label: 'Next Tab', category: 'Tabs', description: 'Switch to the next tab' },
    { key: 'prev_tab', label: 'Previous Tab', category: 'Tabs', description: 'Switch to the previous tab' },
    { key: 'split_horizontal', label: 'Split Horizontal', category: 'Splits', description: 'Split terminal horizontally' },
    { key: 'split_vertical', label: 'Split Vertical', category: 'Splits', description: 'Split terminal vertically' },
    { key: 'toggle_vault', label: 'Toggle Vault', category: 'Application', description: 'Open/close the vault' },
    { key: 'open_settings', label: 'Open Settings', category: 'Application', description: 'Open settings dialog' },
    { key: 'open_file_transfer', label: 'Open File Transfer', category: 'Application', description: 'Open file transfer view' },
    { key: 'open_workspace', label: 'Open Workspace', category: 'Application', description: 'Open workspace manager' },
  ]

  const filteredShortcuts = shortcuts.filter(
    (sc) =>
      sc.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
      sc.description.toLowerCase().includes(searchQuery.toLowerCase())
  )

  const groupedShortcuts = filteredShortcuts.reduce((acc, sc) => {
    if (!acc[sc.category]) acc[sc.category] = []
    acc[sc.category].push(sc)
    return acc
  }, {} as Record<string, typeof shortcuts>)

  const handleKeyPress = (e: React.KeyboardEvent, key: ShortcutKey) => {
    if (!recordingKey) return

    e.preventDefault()

    const keys: string[] = []

    if (e.ctrlKey) keys.push('Ctrl')
    if (e.shiftKey) keys.push('Shift')
    if (e.altKey) keys.push('Alt')
    if (e.metaKey) keys.push('Cmd')

    // Map key codes to readable names
    let mainKey = e.key
    if (mainKey === ' ') mainKey = 'Space'
    if (mainKey === 'Tab') keys.push('Tab')
    else if (mainKey.length === 1) keys.push(mainKey.toUpperCase())
    else if (mainKey.startsWith('Arrow')) keys.push(mainKey.replace('Arrow', ''))
    else keys.push(mainKey)

    const shortcut = keys.join('+')

    // Check for conflicts
    const conflict = Object.entries(settings).find(
      ([k, v]) => k !== key && v === shortcut
    )

    if (conflict) {
      alert(`Shortcut "${shortcut}" is already used for "${conflict[0].replace(/_/g, ' ')}"`)
      return
    }

    onChange({ ...settings, [key]: shortcut })
    setRecordingKey(null)
  }

  const resetShortcut = (key: ShortcutKey, defaultValue: string) => {
    onChange({ ...settings, [key]: defaultValue })
  }

  const getDefaultShortcut = (key: ShortcutKey): string => {
    const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0
    const defaults: Record<ShortcutKey, string> = isMac
      ? {
          new_tab: 'Cmd+T',
          close_tab: 'Cmd+W',
          next_tab: 'Cmd+Tab',
          prev_tab: 'Cmd+Shift+Tab',
          split_horizontal: 'Cmd+Shift+H',
          split_vertical: 'Cmd+Shift+V',
          toggle_vault: 'Cmd+Shift+K',
          open_settings: 'Cmd+,',
          open_file_transfer: 'Cmd+Shift+F',
          open_workspace: 'Cmd+Shift+W',
        }
      : {
          new_tab: 'Ctrl+T',
          close_tab: 'Ctrl+W',
          next_tab: 'Ctrl+Tab',
          prev_tab: 'Ctrl+Shift+Tab',
          split_horizontal: 'Ctrl+Shift+H',
          split_vertical: 'Ctrl+Shift+V',
          toggle_vault: 'Ctrl+Shift+K',
          open_settings: 'Ctrl+,',
          open_file_transfer: 'Ctrl+Shift+F',
          open_workspace: 'Ctrl+Shift+W',
        }
    return defaults[key]
  }

  return (
    <div className="space-y-6">
      {/* Search */}
      <div>
        <input
          type="text"
          placeholder="Search shortcuts..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
        />
      </div>

      {/* Shortcuts by category */}
      {Object.entries(groupedShortcuts).map(([category, shortcuts]) => (
        <div key={category}>
          <h3 className="text-sm font-semibold text-gray-900 uppercase tracking-wide mb-3">
            {category}
          </h3>

          <div className="space-y-2">
            {shortcuts.map((sc) => (
              <div
                key={sc.key}
                className="flex items-center justify-between p-3 bg-gray-50 rounded-md hover:bg-gray-100"
              >
                <div className="flex-1">
                  <div className="text-sm font-medium text-gray-900">{sc.label}</div>
                  <div className="text-xs text-gray-500">{sc.description}</div>
                </div>

                <div className="flex items-center gap-2">
                  {recordingKey === sc.key ? (
                    <input
                      type="text"
                      value="Press keys..."
                      readOnly
                      onKeyDown={(e) => handleKeyPress(e, sc.key)}
                      onBlur={() => setRecordingKey(null)}
                      autoFocus
                      className="px-3 py-1 text-sm border-2 border-blue-500 rounded-md"
                    />
                  ) : (
                    <>
                      <button
                        onClick={() => setRecordingKey(sc.key)}
                        className="px-3 py-1 text-sm font-mono bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                      >
                        {settings[sc.key]}
                      </button>
                      {settings[sc.key] !== getDefaultShortcut(sc.key) && (
                        <button
                          onClick={() => resetShortcut(sc.key, getDefaultShortcut(sc.key))}
                          className="text-xs text-blue-600 hover:text-blue-700"
                          title="Reset to default"
                        >
                          Reset
                        </button>
                      )}
                    </>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      ))}

      {/* Info Box */}
      <div className="mt-8 p-4 bg-blue-50 border border-blue-200 rounded-md">
        <div className="flex items-start">
          <span className="text-blue-600 mr-3 text-xl">⌨️</span>
          <div>
            <h4 className="text-sm font-medium text-blue-900 mb-1">
              Keyboard Shortcuts Tips
            </h4>
            <ul className="text-sm text-blue-800 space-y-1">
              <li>• Click a shortcut to record a new key combination</li>
              <li>• Press Esc to cancel recording</li>
              <li>• Click "Reset" to restore the default shortcut</li>
              <li>• Conflicts will be detected automatically</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}
