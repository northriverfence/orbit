// Settings type definitions matching Rust backend

export interface AppSettings {
  appearance: AppearanceSettings
  connection: ConnectionSettings
  security: SecuritySettings
  shortcuts: KeyboardShortcuts
  general: GeneralSettings
}

export interface AppearanceSettings {
  theme: 'light' | 'dark' | 'system'
  font_family: string
  font_size: number // 12-24
  line_height: number // 1.0-2.0
  cursor_style: 'block' | 'beam' | 'underline'
  cursor_blink: boolean
  scrollback_lines: number // 1000-50000
  color_scheme: string
}

export interface ConnectionSettings {
  default_port: number
  default_username: string
  connect_timeout: number // seconds
  keepalive_interval: number // seconds (0 = disabled)
  auto_reconnect: boolean
  max_reconnect_attempts: number
}

export interface SecuritySettings {
  accept_unknown_hosts: boolean
  accept_changed_hosts: boolean
  save_passwords: boolean
  auto_lock_vault_timeout: number // minutes (0 = never)
  require_confirmation_dangerous: boolean
  enable_notifications: boolean
  notify_session_disconnect: boolean
  notify_file_transfer_complete: boolean
  notify_command_threshold: number // seconds (0 = never)
}

export interface KeyboardShortcuts {
  new_tab: string
  close_tab: string
  next_tab: string
  prev_tab: string
  split_horizontal: string
  split_vertical: string
  toggle_vault: string
  open_settings: string
  open_file_transfer: string
  open_workspace: string
}

export interface GeneralSettings {
  check_for_updates: boolean
  send_analytics: boolean
  restore_sessions_on_startup: boolean
  confirm_before_exit: boolean
  auto_start_daemon: boolean
}
