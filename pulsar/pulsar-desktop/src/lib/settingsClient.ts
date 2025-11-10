// Settings API client

import { invoke } from '@tauri-apps/api/core'
import type {
  AppSettings,
  AppearanceSettings,
  ConnectionSettings,
  SecuritySettings,
  KeyboardShortcuts,
  GeneralSettings,
} from '../types/settings'

/**
 * Settings client for managing application settings
 */
class SettingsClient {
  // ========================================
  // Getters
  // ========================================

  /**
   * Get all settings
   */
  async getAll(): Promise<AppSettings> {
    return await invoke<AppSettings>('settings_get_all')
  }

  /**
   * Get appearance settings
   */
  async getAppearance(): Promise<AppearanceSettings> {
    return await invoke<AppearanceSettings>('settings_get_appearance')
  }

  /**
   * Get connection settings
   */
  async getConnection(): Promise<ConnectionSettings> {
    return await invoke<ConnectionSettings>('settings_get_connection')
  }

  /**
   * Get security settings
   */
  async getSecurity(): Promise<SecuritySettings> {
    return await invoke<SecuritySettings>('settings_get_security')
  }

  /**
   * Get keyboard shortcuts
   */
  async getShortcuts(): Promise<KeyboardShortcuts> {
    return await invoke<KeyboardShortcuts>('settings_get_shortcuts')
  }

  /**
   * Get general settings
   */
  async getGeneral(): Promise<GeneralSettings> {
    return await invoke<GeneralSettings>('settings_get_general')
  }

  // ========================================
  // Setters
  // ========================================

  /**
   * Update appearance settings
   */
  async updateAppearance(appearance: AppearanceSettings): Promise<void> {
    await invoke('settings_update_appearance', { appearance })
  }

  /**
   * Update connection settings
   */
  async updateConnection(connection: ConnectionSettings): Promise<void> {
    await invoke('settings_update_connection', { connection })
  }

  /**
   * Update security settings
   */
  async updateSecurity(security: SecuritySettings): Promise<void> {
    await invoke('settings_update_security', { security })
  }

  /**
   * Update keyboard shortcuts
   */
  async updateShortcuts(shortcuts: KeyboardShortcuts): Promise<void> {
    await invoke('settings_update_shortcuts', { shortcuts })
  }

  /**
   * Update general settings
   */
  async updateGeneral(general: GeneralSettings): Promise<void> {
    await invoke('settings_update_general', { general })
  }

  // ========================================
  // Management
  // ========================================

  /**
   * Reset all settings to defaults
   */
  async resetToDefaults(): Promise<void> {
    await invoke('settings_reset_to_defaults')
  }

  /**
   * Export settings to file
   */
  async export(path: string): Promise<void> {
    await invoke('settings_export', { path })
  }

  /**
   * Import settings from file
   */
  async import(path: string): Promise<AppSettings> {
    return await invoke<AppSettings>('settings_import', { path })
  }
}

// Export singleton instance
export default new SettingsClient()
