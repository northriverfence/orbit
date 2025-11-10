import { useState, useEffect, useRef } from 'react'
import type { AppSettings } from '../types/settings'
import settingsClient from '../lib/settingsClient'
import AppearanceTab from './settings/AppearanceTab'
import ConnectionTab from './settings/ConnectionTab'
import SecurityTab from './settings/SecurityTab'
import ShortcutsTab from './settings/ShortcutsTab'
import GeneralTab from './settings/GeneralTab'
import LoadingOverlay from './LoadingOverlay'
import ErrorAlert from './ErrorAlert'
import { useToast } from './ToastContainer'
import { useFocusTrap } from '../hooks/useFocusTrap'
import { useKeyboardShortcut, SHORTCUTS } from '../hooks/useKeyboardShortcut'

interface SettingsDialogProps {
  isOpen: boolean
  onClose: () => void
}

type TabId = 'appearance' | 'connection' | 'security' | 'shortcuts' | 'general'

export default function SettingsDialog({ isOpen, onClose }: SettingsDialogProps) {
  const dialogRef = useRef<HTMLDivElement>(null)
  const [activeTab, setActiveTab] = useState<TabId>('appearance')
  const [settings, setSettings] = useState<AppSettings | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [isDirty, setIsDirty] = useState(false)
  const [saving, setSaving] = useState(false)
  const toast = useToast()

  // Load settings when dialog opens
  useEffect(() => {
    if (isOpen) {
      loadSettings()
    }
  }, [isOpen])

  const loadSettings = async () => {
    setLoading(true)
    setError(null)
    try {
      const allSettings = await settingsClient.getAll()
      setSettings(allSettings)
      setIsDirty(false)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load settings')
    } finally {
      setLoading(false)
    }
  }

  const handleClose = () => {
    if (isDirty) {
      const confirmed = window.confirm(
        'You have unsaved changes. Are you sure you want to close?'
      )
      if (!confirmed) return
    }
    onClose()
  }

  const handleSave = async () => {
    if (!settings) return

    setSaving(true)
    setError(null)

    try {
      // Save each section
      await settingsClient.updateAppearance(settings.appearance)
      await settingsClient.updateConnection(settings.connection)
      await settingsClient.updateSecurity(settings.security)
      await settingsClient.updateShortcuts(settings.shortcuts)
      await settingsClient.updateGeneral(settings.general)

      setIsDirty(false)
      toast.showSuccess('Settings saved successfully!')
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to save settings'
      setError(errorMessage)
      toast.showError(errorMessage)
    } finally {
      setSaving(false)
    }
  }

  const handleReset = async () => {
    const confirmed = window.confirm(
      'Are you sure you want to reset all settings to defaults? This cannot be undone.'
    )
    if (!confirmed) return

    setLoading(true)
    setError(null)

    try {
      await settingsClient.resetToDefaults()
      await loadSettings()
      toast.showSuccess('Settings reset to defaults')
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to reset settings'
      setError(errorMessage)
      toast.showError(errorMessage)
    } finally {
      setLoading(false)
    }
  }

  const updateSettings = (updates: Partial<AppSettings>) => {
    setSettings((prev) => (prev ? { ...prev, ...updates } : null))
    setIsDirty(true)
  }

  // Keyboard navigation (after function definitions)
  useFocusTrap(dialogRef, isOpen)
  useKeyboardShortcut(SHORTCUTS.ESCAPE, handleClose, isOpen)
  useKeyboardShortcut(SHORTCUTS.SAVE, handleSave, isOpen && isDirty && !saving && !loading)

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 modal-backdrop">
      <div ref={dialogRef} className="bg-white rounded-lg shadow-xl w-[800px] max-h-[90vh] flex flex-col relative modal-content">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b">
          <h2 className="text-2xl font-bold text-gray-900">Settings</h2>
          <button
            onClick={handleClose}
            className="text-gray-400 hover:text-gray-600 text-2xl leading-none"
          >
            ×
          </button>
        </div>

        {/* Tabs */}
        <div className="flex border-b px-6">
          {[
            { id: 'appearance' as TabId, label: '🎨 Appearance', icon: '🎨' },
            { id: 'connection' as TabId, label: '🔌 Connection', icon: '🔌' },
            { id: 'security' as TabId, label: '🔒 Security', icon: '🔒' },
            { id: 'shortcuts' as TabId, label: '⌨️ Shortcuts', icon: '⌨️' },
            { id: 'general' as TabId, label: '⚙️ General', icon: '⚙️' },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`
                px-4 py-3 font-medium text-sm transition-colors
                ${
                  activeTab === tab.id
                    ? 'text-blue-600 border-b-2 border-blue-600'
                    : 'text-gray-600 hover:text-gray-900'
                }
              `}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {/* Saving overlay */}
        {saving && <LoadingOverlay message="Saving settings..." />}

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6">
          {loading && <LoadingOverlay message="Loading settings..." />}

          {error && (
            <ErrorAlert
              message={error}
              type="error"
              onDismiss={() => setError(null)}
              onRetry={loadSettings}
            />
          )}

          {!loading && settings && (
            <>
              {activeTab === 'appearance' && (
                <AppearanceTab
                  settings={settings.appearance}
                  onChange={(appearance) => updateSettings({ appearance })}
                />
              )}

              {activeTab === 'connection' && (
                <ConnectionTab
                  settings={settings.connection}
                  onChange={(connection) => updateSettings({ connection })}
                />
              )}

              {activeTab === 'security' && (
                <SecurityTab
                  settings={settings.security}
                  onChange={(security) => updateSettings({ security })}
                />
              )}

              {activeTab === 'shortcuts' && (
                <ShortcutsTab
                  settings={settings.shortcuts}
                  onChange={(shortcuts) => updateSettings({ shortcuts })}
                />
              )}

              {activeTab === 'general' && (
                <GeneralTab
                  settings={settings.general}
                  onChange={(general) => updateSettings({ general })}
                />
              )}
            </>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-6 border-t bg-gray-50">
          <button
            onClick={handleReset}
            disabled={loading || saving}
            className="px-4 py-2 text-sm font-medium text-red-600 hover:text-red-700 disabled:opacity-50"
          >
            Reset to Defaults
          </button>

          <div className="flex gap-3">
            <button
              onClick={handleClose}
              disabled={saving}
              className="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900 disabled:opacity-50 transition-colors btn-press"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              disabled={!isDirty || saving || loading}
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors btn-press"
            >
              {saving ? 'Saving...' : 'Save Changes'}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
