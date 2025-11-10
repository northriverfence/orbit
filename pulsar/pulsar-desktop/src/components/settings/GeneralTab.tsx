import { useState } from 'react'
import type { GeneralSettings } from '../../types/settings'
import settingsClient from '../../lib/settingsClient'

interface GeneralTabProps {
  settings: GeneralSettings
  onChange: (settings: GeneralSettings) => void
}

export default function GeneralTab({ settings, onChange }: GeneralTabProps) {
  const [exporting, setExporting] = useState(false)
  const [importing, setImporting] = useState(false)

  const updateSetting = <K extends keyof GeneralSettings>(
    key: K,
    value: GeneralSettings[K]
  ) => {
    onChange({ ...settings, [key]: value })
  }

  const handleExport = async () => {
    setExporting(true)
    try {
      // Use file picker to select export location
      const fileName = `pulsar-settings-${new Date().toISOString().split('T')[0]}.toml`

      // Create a download link (browser-based approach)
      // In a real implementation, you might use Tauri's file dialog
      const settingsData = await settingsClient.getAll()
      const blob = new Blob([JSON.stringify(settingsData, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = fileName
      a.click()
      URL.revokeObjectURL(url)

      alert('Settings exported successfully!')
    } catch (err) {
      alert(`Failed to export settings: ${err instanceof Error ? err.message : 'Unknown error'}`)
    } finally {
      setExporting(false)
    }
  }

  const handleImport = async () => {
    setImporting(true)
    try {
      // Use file picker to select import file
      const input = document.createElement('input')
      input.type = 'file'
      input.accept = '.toml,.json'

      input.onchange = async (e) => {
        const file = (e.target as HTMLInputElement).files?.[0]
        if (!file) return

        try {
          // In a real implementation, you'd call settingsClient.import(path)
          // For now, just show a message
          alert('Import functionality requires Tauri file dialog integration')
        } catch (err) {
          alert(`Failed to import settings: ${err instanceof Error ? err.message : 'Unknown error'}`)
        }
      }

      input.click()
    } catch (err) {
      alert(`Failed to import settings: ${err instanceof Error ? err.message : 'Unknown error'}`)
    } finally {
      setImporting(false)
    }
  }

  return (
    <div className="space-y-6">
      {/* Application Updates */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-3">Application Updates</h3>

        <label className="flex items-start">
          <input
            type="checkbox"
            checked={settings.check_for_updates}
            onChange={(e) => updateSetting('check_for_updates', e.target.checked)}
            className="mr-2 mt-1"
          />
          <div>
            <span className="text-sm font-medium text-gray-700">
              Check for updates automatically
            </span>
            <p className="text-xs text-gray-500 mt-1">
              Pulsar will check for updates on startup and notify you when a new version is available
            </p>
          </div>
        </label>
      </div>

      {/* Privacy & Analytics */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-3">Privacy & Analytics</h3>

        <label className="flex items-start">
          <input
            type="checkbox"
            checked={settings.send_analytics}
            onChange={(e) => updateSetting('send_analytics', e.target.checked)}
            className="mr-2 mt-1"
          />
          <div>
            <span className="text-sm font-medium text-gray-700">
              Send anonymous usage analytics
            </span>
            <p className="text-xs text-gray-500 mt-1">
              Help improve Pulsar by sending anonymous usage statistics. No personal data or command history is collected.
            </p>
          </div>
        </label>
      </div>

      {/* Session Management */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-3">Session Management</h3>

        <div className="space-y-3">
          <label className="flex items-start">
            <input
              type="checkbox"
              checked={settings.restore_sessions_on_startup}
              onChange={(e) => updateSetting('restore_sessions_on_startup', e.target.checked)}
              className="mr-2 mt-1"
            />
            <div>
              <span className="text-sm font-medium text-gray-700">
                Restore sessions on startup
              </span>
              <p className="text-xs text-gray-500 mt-1">
                Automatically reconnect to all active sessions when Pulsar starts
              </p>
            </div>
          </label>

          <label className="flex items-start">
            <input
              type="checkbox"
              checked={settings.confirm_before_exit}
              onChange={(e) => updateSetting('confirm_before_exit', e.target.checked)}
              className="mr-2 mt-1"
            />
            <div>
              <span className="text-sm font-medium text-gray-700">
                Confirm before exiting with active sessions
              </span>
              <p className="text-xs text-gray-500 mt-1">
                Show a confirmation dialog when closing Pulsar with active connections
              </p>
            </div>
          </label>
        </div>
      </div>

      {/* Daemon Settings */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-3">Daemon Settings</h3>

        <label className="flex items-start">
          <input
            type="checkbox"
            checked={settings.auto_start_daemon}
            onChange={(e) => updateSetting('auto_start_daemon', e.target.checked)}
            className="mr-2 mt-1"
          />
          <div>
            <span className="text-sm font-medium text-gray-700">
              Auto-start orbitd daemon
            </span>
            <p className="text-xs text-gray-500 mt-1">
              Automatically start the orbitd daemon when Pulsar launches (recommended for best performance)
            </p>
          </div>
        </label>
      </div>

      {/* Settings Import/Export */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-3">Settings Management</h3>

        <div className="flex gap-3">
          <button
            onClick={handleExport}
            disabled={exporting}
            className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {exporting ? 'Exporting...' : '📤 Export Settings'}
          </button>

          <button
            onClick={handleImport}
            disabled={importing}
            className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {importing ? 'Importing...' : '📥 Import Settings'}
          </button>
        </div>

        <p className="text-xs text-gray-500 mt-2">
          Export your settings to back them up or transfer to another machine
        </p>
      </div>

      {/* Info Box */}
      <div className="mt-8 p-4 bg-purple-50 border border-purple-200 rounded-md">
        <div className="flex items-start">
          <span className="text-purple-600 mr-3 text-xl">⚙️</span>
          <div>
            <h4 className="text-sm font-medium text-purple-900 mb-1">
              General Settings Tips
            </h4>
            <ul className="text-sm text-purple-800 space-y-1">
              <li>• Settings are saved automatically to ~/.config/orbit/pulsar_settings.toml</li>
              <li>• Export your settings regularly to back up your configuration</li>
              <li>• Session restoration preserves all tabs and connections</li>
              <li>• Analytics data is completely anonymous and helps improve the app</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}
