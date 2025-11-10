// Client for sending notifications from frontend

import { invoke } from '@tauri-apps/api/core'

class NotificationClient {
  /**
   * Send a session disconnected notification
   */
  async sessionDisconnected(sessionId: string, reason: string): Promise<void> {
    await invoke('notify_session_disconnected', { sessionId, reason })
  }

  /**
   * Send a session reconnected notification
   */
  async sessionReconnected(sessionId: string): Promise<void> {
    await invoke('notify_session_reconnected', { sessionId })
  }

  /**
   * Send a file transfer complete notification
   */
  async fileTransferComplete(
    filename: string,
    success: boolean,
    sizeBytes?: number
  ): Promise<void> {
    await invoke('notify_file_transfer_complete', {
      filename,
      success,
      sizeBytes,
    })
  }

  /**
   * Send a command completed notification
   */
  async commandCompleted(
    command: string,
    exitCode: number,
    durationSecs: number
  ): Promise<void> {
    await invoke('notify_command_completed', {
      command,
      exitCode,
      durationSecs,
    })
  }

  /**
   * Send a vault locked notification
   */
  async vaultLocked(reason: string): Promise<void> {
    await invoke('notify_vault_locked', { reason })
  }

  /**
   * Send an update available notification
   */
  async updateAvailable(version: string, url: string): Promise<void> {
    await invoke('notify_update_available', { version, url })
  }

  /**
   * Send a generic info notification
   */
  async info(title: string, message: string): Promise<void> {
    await invoke('notify_info', { title, message })
  }

  /**
   * Send a generic warning notification
   */
  async warning(title: string, message: string): Promise<void> {
    await invoke('notify_warning', { title, message })
  }

  /**
   * Send a generic error notification
   */
  async error(title: string, message: string): Promise<void> {
    await invoke('notify_error', { title, message })
  }

  /**
   * Send a test notification (for settings preview)
   */
  async test(): Promise<void> {
    await invoke('notify_test')
  }

  /**
   * Cleanup old notification records
   */
  async cleanup(): Promise<void> {
    await invoke('notifications_cleanup')
  }
}

export default new NotificationClient()
