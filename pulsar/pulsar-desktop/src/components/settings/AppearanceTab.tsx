import type { AppearanceSettings } from '../../types/settings'

interface AppearanceTabProps {
  settings: AppearanceSettings
  onChange: (settings: AppearanceSettings) => void
}

export default function AppearanceTab({ settings, onChange }: AppearanceTabProps) {
  const updateSetting = <K extends keyof AppearanceSettings>(
    key: K,
    value: AppearanceSettings[K]
  ) => {
    onChange({ ...settings, [key]: value })
  }

  return (
    <div className="space-y-6">
      {/* Theme */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Theme
        </label>
        <div className="flex gap-4">
          {['light', 'dark', 'system'].map((theme) => (
            <label key={theme} className="flex items-center">
              <input
                type="radio"
                name="theme"
                value={theme}
                checked={settings.theme === theme}
                onChange={(e) => updateSetting('theme', e.target.value as any)}
                className="mr-2"
              />
              <span className="text-sm text-gray-700 capitalize">{theme}</span>
            </label>
          ))}
        </div>
      </div>

      {/* Font Family */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Font Family
        </label>
        <select
          value={settings.font_family}
          onChange={(e) => updateSetting('font_family', e.target.value)}
          className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
        >
          <option value="Menlo">Menlo</option>
          <option value="Monaco">Monaco</option>
          <option value="Courier New">Courier New</option>
          <option value="Consolas">Consolas</option>
          <option value="DejaVu Sans Mono">DejaVu Sans Mono</option>
          <option value="Cascadia Code">Cascadia Code</option>
          <option value="Fira Code">Fira Code</option>
          <option value="JetBrains Mono">JetBrains Mono</option>
        </select>
      </div>

      {/* Font Size */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Font Size: {settings.font_size}px
        </label>
        <input
          type="range"
          min="12"
          max="24"
          value={settings.font_size}
          onChange={(e) => updateSetting('font_size', parseInt(e.target.value))}
          className="block w-full"
        />
        <div className="flex justify-between text-xs text-gray-500 mt-1">
          <span>12px</span>
          <span>24px</span>
        </div>
      </div>

      {/* Line Height */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Line Height: {settings.line_height.toFixed(1)}
        </label>
        <input
          type="range"
          min="1.0"
          max="2.0"
          step="0.1"
          value={settings.line_height}
          onChange={(e) => updateSetting('line_height', parseFloat(e.target.value))}
          className="block w-full"
        />
        <div className="flex justify-between text-xs text-gray-500 mt-1">
          <span>1.0</span>
          <span>2.0</span>
        </div>
      </div>

      {/* Cursor Style */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Cursor Style
        </label>
        <div className="flex gap-4">
          {['block', 'beam', 'underline'].map((style) => (
            <label key={style} className="flex items-center">
              <input
                type="radio"
                name="cursor_style"
                value={style}
                checked={settings.cursor_style === style}
                onChange={(e) => updateSetting('cursor_style', e.target.value as any)}
                className="mr-2"
              />
              <span className="text-sm text-gray-700 capitalize">{style}</span>
            </label>
          ))}
        </div>
      </div>

      {/* Cursor Blink */}
      <div>
        <label className="flex items-center">
          <input
            type="checkbox"
            checked={settings.cursor_blink}
            onChange={(e) => updateSetting('cursor_blink', e.target.checked)}
            className="mr-2"
          />
          <span className="text-sm font-medium text-gray-700">
            Enable cursor blinking
          </span>
        </label>
      </div>

      {/* Scrollback Lines */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Scrollback Lines
        </label>
        <input
          type="number"
          min="1000"
          max="50000"
          step="1000"
          value={settings.scrollback_lines}
          onChange={(e) => updateSetting('scrollback_lines', parseInt(e.target.value))}
          className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
        />
        <p className="text-xs text-gray-500 mt-1">
          Number of lines to keep in scrollback buffer (1000-50000)
        </p>
      </div>

      {/* Preview Section */}
      <div className="mt-8 p-4 bg-gray-900 rounded-md">
        <div className="text-white font-mono" style={{
          fontFamily: settings.font_family,
          fontSize: `${settings.font_size}px`,
          lineHeight: settings.line_height,
        }}>
          <p>$ echo "Hello, Pulsar Terminal!"</p>
          <p className="text-green-400">Hello, Pulsar Terminal!</p>
          <p>$ ls -la</p>
          <p className="text-blue-400">
            total 48<br />
            drwxr-xr-x  12 user  staff   384 Nov  9 14:30 .<br />
            drwxr-xr-x   5 user  staff   160 Nov  9 12:00 ..
          </p>
          <p>
            <span className={`inline-block w-2 h-4 ${settings.cursor_blink ? 'animate-pulse' : ''} bg-white`}
                  style={{
                    width: settings.cursor_style === 'block' ? '0.6em' : '2px',
                    height: settings.cursor_style === 'underline' ? '2px' : '1.2em',
                    marginTop: settings.cursor_style === 'underline' ? '1em' : '0',
                  }}
            ></span>
          </p>
        </div>
        <p className="text-xs text-gray-400 mt-4">Live preview of your terminal settings</p>
      </div>
    </div>
  )
}
