import type { ConnectionSettings } from '../../types/settings'

interface ConnectionTabProps {
  settings: ConnectionSettings
  onChange: (settings: ConnectionSettings) => void
}

export default function ConnectionTab({ settings, onChange }: ConnectionTabProps) {
  const updateSetting = <K extends keyof ConnectionSettings>(
    key: K,
    value: ConnectionSettings[K]
  ) => {
    onChange({ ...settings, [key]: value })
  }

  return (
    <div className="space-y-6">
      {/* Default Port */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Default SSH Port
        </label>
        <input
          type="number"
          min="1"
          max="65535"
          value={settings.default_port}
          onChange={(e) => updateSetting('default_port', parseInt(e.target.value))}
          className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
        />
        <p className="text-xs text-gray-500 mt-1">
          Default port for SSH connections (typically 22)
        </p>
      </div>

      {/* Default Username */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Default Username
        </label>
        <input
          type="text"
          value={settings.default_username}
          onChange={(e) => updateSetting('default_username', e.target.value)}
          className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
          placeholder="username"
        />
        <p className="text-xs text-gray-500 mt-1">
          Default username for SSH connections
        </p>
      </div>

      {/* Connection Timeout */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Connection Timeout: {settings.connect_timeout}s
        </label>
        <input
          type="range"
          min="5"
          max="300"
          step="5"
          value={settings.connect_timeout}
          onChange={(e) => updateSetting('connect_timeout', parseInt(e.target.value))}
          className="block w-full"
        />
        <div className="flex justify-between text-xs text-gray-500 mt-1">
          <span>5s</span>
          <span>300s (5 min)</span>
        </div>
        <p className="text-xs text-gray-500 mt-1">
          How long to wait before timing out a connection attempt
        </p>
      </div>

      {/* Keepalive Interval */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Keepalive Interval: {settings.keepalive_interval === 0 ? 'Disabled' : `${settings.keepalive_interval}s`}
        </label>
        <input
          type="range"
          min="0"
          max="600"
          step="30"
          value={settings.keepalive_interval}
          onChange={(e) => updateSetting('keepalive_interval', parseInt(e.target.value))}
          className="block w-full"
        />
        <div className="flex justify-between text-xs text-gray-500 mt-1">
          <span>Disabled</span>
          <span>600s (10 min)</span>
        </div>
        <p className="text-xs text-gray-500 mt-1">
          Send keepalive packets to prevent connection timeout (0 = disabled)
        </p>
      </div>

      {/* Auto Reconnect */}
      <div>
        <label className="flex items-center">
          <input
            type="checkbox"
            checked={settings.auto_reconnect}
            onChange={(e) => updateSetting('auto_reconnect', e.target.checked)}
            className="mr-2"
          />
          <span className="text-sm font-medium text-gray-700">
            Automatically reconnect on connection loss
          </span>
        </label>
        <p className="text-xs text-gray-500 mt-1 ml-6">
          Attempt to reconnect if the connection is unexpectedly lost
        </p>
      </div>

      {/* Max Reconnect Attempts */}
      {settings.auto_reconnect && (
        <div className="ml-6">
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Maximum Reconnect Attempts
          </label>
          <input
            type="number"
            min="1"
            max="10"
            value={settings.max_reconnect_attempts}
            onChange={(e) => updateSetting('max_reconnect_attempts', parseInt(e.target.value))}
            className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
          />
          <p className="text-xs text-gray-500 mt-1">
            Number of reconnection attempts before giving up (1-10)
          </p>
        </div>
      )}

      {/* Info Box */}
      <div className="mt-8 p-4 bg-blue-50 border border-blue-200 rounded-md">
        <div className="flex items-start">
          <span className="text-blue-600 mr-3 text-xl">ℹ️</span>
          <div>
            <h4 className="text-sm font-medium text-blue-900 mb-1">
              Connection Tips
            </h4>
            <ul className="text-sm text-blue-800 space-y-1">
              <li>• These are default values used when creating new connections</li>
              <li>• You can override them for individual connections</li>
              <li>• Keepalive prevents idle connection timeouts</li>
              <li>• Auto-reconnect is useful for unstable networks</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}
