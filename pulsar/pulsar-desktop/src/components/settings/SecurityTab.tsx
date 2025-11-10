import type { SecuritySettings } from '../../types/settings'

interface SecurityTabProps {
  settings: SecuritySettings
  onChange: (settings: SecuritySettings) => void
}

export default function SecurityTab({ settings, onChange }: SecurityTabProps) {
  const updateSetting = <K extends keyof SecuritySettings>(
    key: K,
    value: SecuritySettings[K]
  ) => {
    onChange({ ...settings, [key]: value })
  }

  return (
    <div className="space-y-6">
      {/* SSH Host Keys */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-3">SSH Host Keys</h3>

        <div className="space-y-3">
          <label className="flex items-start">
            <input
              type="checkbox"
              checked={settings.accept_unknown_hosts}
              onChange={(e) => updateSetting('accept_unknown_hosts', e.target.checked)}
              className="mr-2 mt-1"
            />
            <div>
              <span className="text-sm font-medium text-gray-700">
                Accept unknown host keys automatically
              </span>
              <p className="text-xs text-orange-600 mt-1">
                ⚠️ INSECURE: Only enable for development. This bypasses host key verification.
              </p>
            </div>
          </label>

          <label className="flex items-start">
            <input
              type="checkbox"
              checked={settings.accept_changed_hosts}
              onChange={(e) => updateSetting('accept_changed_hosts', e.target.checked)}
              className="mr-2 mt-1"
            />
            <div>
              <span className="text-sm font-medium text-gray-700">
                Accept changed host keys automatically
              </span>
              <p className="text-xs text-red-600 mt-1">
                🚨 VERY INSECURE: This could indicate a man-in-the-middle attack. Only enable for development.
              </p>
            </div>
          </label>
        </div>
      </div>

      {/* Vault Settings */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-3">Vault Security</h3>

        <div className="space-y-3">
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={settings.save_passwords}
              onChange={(e) => updateSetting('save_passwords', e.target.checked)}
              className="mr-2"
            />
            <span className="text-sm font-medium text-gray-700">
              Save passwords in vault
            </span>
          </label>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Auto-lock vault after inactivity
            </label>
            <select
              value={settings.auto_lock_vault_timeout}
              onChange={(e) => updateSetting('auto_lock_vault_timeout', parseInt(e.target.value))}
              className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
            >
              <option value="0">Never</option>
              <option value="5">5 minutes</option>
              <option value="15">15 minutes</option>
              <option value="30">30 minutes</option>
              <option value="60">1 hour</option>
              <option value="120">2 hours</option>
            </select>
            <p className="text-xs text-gray-500 mt-1">
              Automatically lock the vault after this period of inactivity
            </p>
          </div>
        </div>
      </div>

      {/* Command Execution */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-3">Command Execution</h3>

        <label className="flex items-start">
          <input
            type="checkbox"
            checked={settings.require_confirmation_dangerous}
            onChange={(e) => updateSetting('require_confirmation_dangerous', e.target.checked)}
            className="mr-2 mt-1"
          />
          <div>
            <span className="text-sm font-medium text-gray-700">
              Require confirmation for dangerous commands
            </span>
            <p className="text-xs text-gray-500 mt-1">
              Show a warning before executing potentially dangerous commands (rm -rf, dd, mkfs, etc.)
            </p>
          </div>
        </label>
      </div>

      {/* Notifications */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-3">Notifications</h3>

        <div className="space-y-3">
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={settings.enable_notifications}
              onChange={(e) => updateSetting('enable_notifications', e.target.checked)}
              className="mr-2"
            />
            <span className="text-sm font-medium text-gray-700">
              Enable desktop notifications
            </span>
          </label>

          {settings.enable_notifications && (
            <div className="ml-6 space-y-3">
              <label className="flex items-center">
                <input
                  type="checkbox"
                  checked={settings.notify_session_disconnect}
                  onChange={(e) => updateSetting('notify_session_disconnect', e.target.checked)}
                  className="mr-2"
                />
                <span className="text-sm text-gray-700">
                  Notify when session disconnects
                </span>
              </label>

              <label className="flex items-center">
                <input
                  type="checkbox"
                  checked={settings.notify_file_transfer_complete}
                  onChange={(e) => updateSetting('notify_file_transfer_complete', e.target.checked)}
                  className="mr-2"
                />
                <span className="text-sm text-gray-700">
                  Notify when file transfer completes
                </span>
              </label>

              <div>
                <label className="block text-sm text-gray-700 mb-2">
                  Notify when command takes longer than
                </label>
                <select
                  value={settings.notify_command_threshold}
                  onChange={(e) => updateSetting('notify_command_threshold', parseInt(e.target.value))}
                  className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
                >
                  <option value="0">Never</option>
                  <option value="30">30 seconds</option>
                  <option value="60">1 minute</option>
                  <option value="300">5 minutes</option>
                  <option value="600">10 minutes</option>
                </select>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Security Info */}
      <div className="mt-8 p-4 bg-green-50 border border-green-200 rounded-md">
        <div className="flex items-start">
          <span className="text-green-600 mr-3 text-xl">🔒</span>
          <div>
            <h4 className="text-sm font-medium text-green-900 mb-1">
              Security Best Practices
            </h4>
            <ul className="text-sm text-green-800 space-y-1">
              <li>• Keep "Accept unknown/changed hosts" disabled in production</li>
              <li>• Use SSH keys instead of passwords when possible</li>
              <li>• Enable auto-lock vault for sensitive environments</li>
              <li>• Review dangerous command confirmations before proceeding</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}
