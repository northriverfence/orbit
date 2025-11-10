# Pulsar MVP Completion Plan

**Start Date:** 2025-11-09
**Target Completion:** 2025-11-18 (7-9 days)
**Status:** 🚀 **IN PROGRESS**

---

## 📋 Overview

This plan covers the final 3 tracks needed to reach production-ready MVP status:
- **Track 1:** Settings UI (2-3 days)
- **Track 2:** Daemon Completion (2-3 days)
- **Track 3:** Essential UI Polish (2-3 days)

**Total Estimated:** 7-9 days to MVP

---

## 🎯 Track 1: Settings UI (Days 1-3)

### Day 1: Settings Architecture & Backend

#### Task 1.1: Design Settings Data Model
**Duration:** 2 hours
**Deliverables:**
- Settings schema definition
- Default values
- Validation rules
- Storage format (TOML/JSON)

**Data Model:**
```rust
struct AppSettings {
    appearance: AppearanceSettings,
    connection: ConnectionSettings,
    security: SecuritySettings,
    shortcuts: KeyboardShortcuts,
    general: GeneralSettings,
}

struct AppearanceSettings {
    theme: Theme,               // "light", "dark", "system"
    font_family: String,        // "Menlo", "Monaco", etc.
    font_size: u8,             // 12-24
    line_height: f32,          // 1.0-2.0
    cursor_style: CursorStyle, // "block", "beam", "underline"
    cursor_blink: bool,
    scrollback_lines: u32,     // 1000-50000
}

struct ConnectionSettings {
    default_port: u16,
    default_username: String,
    connect_timeout: u64,      // seconds
    keepalive_interval: u64,   // seconds
    auto_reconnect: bool,
    max_reconnect_attempts: u32,
}

struct SecuritySettings {
    accept_unknown_hosts: bool,
    accept_changed_hosts: bool,
    save_passwords: bool,
    auto_lock_vault_timeout: u64, // minutes, 0 = never
    require_confirmation_dangerous_commands: bool,
}

struct KeyboardShortcuts {
    new_tab: String,           // "Ctrl+T"
    close_tab: String,         // "Ctrl+W"
    next_tab: String,          // "Ctrl+Tab"
    prev_tab: String,          // "Ctrl+Shift+Tab"
    split_horizontal: String,  // "Ctrl+Shift+H"
    split_vertical: String,    // "Ctrl+Shift+V"
    toggle_vault: String,      // "Ctrl+Shift+K"
    open_settings: String,     // "Ctrl+,"
}

struct GeneralSettings {
    check_for_updates: bool,
    send_analytics: bool,
    restore_sessions_on_startup: bool,
    confirm_before_exit: bool,
}
```

#### Task 1.2: Implement Settings Backend
**Duration:** 3 hours
**Files:**
- `src-tauri/src/settings/mod.rs` - Settings manager
- `src-tauri/src/settings/storage.rs` - Persistence layer
- `src-tauri/src/settings_commands.rs` - Tauri commands

**Backend Commands:**
```rust
// Settings state
settings_get_all() -> AppSettings
settings_get_appearance() -> AppearanceSettings
settings_get_connection() -> ConnectionSettings
settings_get_security() -> SecuritySettings
settings_get_shortcuts() -> KeyboardShortcuts

// Settings updates
settings_update_appearance(settings: AppearanceSettings) -> Result<()>
settings_update_connection(settings: ConnectionSettings) -> Result<()>
settings_update_security(settings: SecuritySettings) -> Result<()>
settings_update_shortcuts(shortcuts: KeyboardShortcuts) -> Result<()>

// Settings management
settings_reset_to_defaults() -> Result<()>
settings_export(path: String) -> Result<()>
settings_import(path: String) -> Result<AppSettings>
```

**Implementation Details:**
- Use `serde` for serialization
- Store in `~/.config/orbit/pulsar_settings.toml`
- Validate on load, use defaults if invalid
- Atomic writes (write to temp, then rename)
- Watch for external changes (reload if modified)

---

### Day 2: Settings UI - Core Tabs

#### Task 1.3: Settings Dialog Component
**Duration:** 2 hours
**File:** `src/components/SettingsDialog.tsx`

**Features:**
- Modal dialog with tabs
- Tabbed interface (Appearance, Connection, Security, Shortcuts, General)
- Save/Cancel/Reset buttons
- Validation and error display
- Dirty state tracking (warn on close with unsaved changes)

#### Task 1.4: Appearance Settings Tab
**Duration:** 2 hours
**File:** `src/components/settings/AppearanceTab.tsx`

**UI Elements:**
```typescript
- Theme selector (Light/Dark/System) with radio buttons
- Font family dropdown (Menlo, Monaco, Courier New, etc.)
- Font size slider (12-24 with preview)
- Line height slider (1.0-2.0)
- Cursor style selector (Block/Beam/Underline)
- Cursor blink toggle
- Scrollback lines input (1000-50000)
- Live preview section showing terminal with current settings
```

#### Task 1.5: Connection Settings Tab
**Duration:** 2 hours
**File:** `src/components/settings/ConnectionTab.tsx`

**UI Elements:**
```typescript
- Default SSH port input (1-65535)
- Default username input
- Connection timeout slider (5-300 seconds)
- Keepalive interval slider (0-600 seconds, 0 = disabled)
- Auto-reconnect toggle
- Max reconnect attempts input (1-10)
```

---

### Day 3: Settings UI - Advanced Tabs

#### Task 1.6: Security Settings Tab
**Duration:** 2 hours
**File:** `src/components/settings/SecurityTab.tsx`

**UI Elements:**
```typescript
- Accept unknown hosts toggle (with warning)
- Accept changed hosts toggle (with danger warning)
- Save passwords toggle
- Auto-lock vault timeout selector (Never, 5m, 15m, 30m, 1h, 2h)
- Require confirmation for dangerous commands toggle
- List of dangerous command patterns (read-only)
```

#### Task 1.7: Keyboard Shortcuts Tab
**Duration:** 2 hours
**File:** `src/components/settings/ShortcutsTab.tsx`

**UI Elements:**
```typescript
- Shortcut list with categories (Tabs, Splits, Application)
- Inline shortcut editor (click to record new shortcut)
- Conflict detection (warn if shortcut already used)
- Reset individual shortcut button
- Reset all shortcuts button
- Search/filter shortcuts
```

#### Task 1.8: General Settings Tab
**Duration:** 1 hour
**File:** `src/components/settings/GeneralTab.tsx`

**UI Elements:**
```typescript
- Check for updates toggle
- Send anonymous analytics toggle (with privacy policy link)
- Restore sessions on startup toggle
- Confirm before exit toggle
- Export settings button
- Import settings button
```

#### Task 1.9: Settings Integration
**Duration:** 1 hour

**Tasks:**
- Add "Settings" button to navbar
- Add keyboard shortcut (Ctrl+,) to open settings
- Apply settings changes to active terminals
- Persist settings on change
- Load settings on app startup

---

## 🎯 Track 2: Daemon Completion (Days 4-6)

### Day 4: Database Persistence

#### Task 2.1: Session Database Schema
**Duration:** 2 hours
**File:** `pulsar-daemon/src/database/schema.rs`

**Schema:**
```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    session_type TEXT NOT NULL,  -- 'ssh' or 'local'
    created_at INTEGER NOT NULL,
    last_active INTEGER NOT NULL,
    status TEXT NOT NULL,        -- 'active', 'detached', 'terminated'
    config TEXT NOT NULL,        -- JSON config
    workspace_id TEXT,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
);

CREATE TABLE session_snapshots (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    snapshot_at INTEGER NOT NULL,
    terminal_buffer BLOB NOT NULL,
    scrollback BLOB,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    layout TEXT NOT NULL,        -- JSON layout
    active_session_id TEXT
);
```

#### Task 2.2: Database Operations
**Duration:** 3 hours
**File:** `pulsar-daemon/src/database/operations.rs`

**Functions:**
```rust
// Session CRUD
save_session(session: &Session) -> Result<()>
load_session(id: &str) -> Result<Session>
list_sessions() -> Result<Vec<SessionSummary>>
delete_session(id: &str) -> Result<()>
update_session_status(id: &str, status: SessionStatus) -> Result<()>

// Snapshots
save_snapshot(session_id: &str, buffer: &[u8]) -> Result<()>
load_latest_snapshot(session_id: &str) -> Result<Option<Snapshot>>
list_snapshots(session_id: &str) -> Result<Vec<SnapshotInfo>>

// Workspaces
save_workspace(workspace: &Workspace) -> Result<()>
load_workspace(id: &str) -> Result<Workspace>
list_workspaces() -> Result<Vec<WorkspaceSummary>>
```

#### Task 2.3: Persistence Integration
**Duration:** 2 hours

**Tasks:**
- Connect session manager to database
- Auto-save sessions every 30 seconds
- Save snapshot on detach
- Load sessions on daemon startup
- Clean up old snapshots (keep last 10 per session)

---

### Day 5: Desktop Notifications

#### Task 2.4: Notification Service
**Duration:** 3 hours
**File:** `src-tauri/src/notifications/mod.rs`

**Features:**
```rust
enum NotificationType {
    SessionDisconnected { session_id: String, reason: String },
    FileTransferComplete { filename: String, success: bool },
    CommandCompleted { command: String, exit_code: i32 },
    VaultLocked,
    UpdateAvailable { version: String },
}

fn send_notification(
    title: &str,
    body: &str,
    notification_type: NotificationType,
    actions: Vec<NotificationAction>,
) -> Result<()>
```

**Platform Support:**
- Linux: libnotify (via `notify-rust`)
- macOS: NSUserNotificationCenter
- Windows: Windows 10 Toast notifications

**Integration Points:**
- Session disconnects (SSH connection lost)
- File transfer completion (with success/failure)
- Long-running commands (>30s)
- Vault auto-lock
- Update notifications

#### Task 2.5: Notification Preferences
**Duration:** 2 hours

**Add to Security Settings:**
```typescript
- Enable notifications toggle
- Notify on session disconnect toggle
- Notify on file transfer complete toggle
- Notify on command complete (threshold: 30s/1m/5m/never)
- Notification sound toggle
```

---

### Day 6: Auto-Start Configuration

#### Task 2.6: Platform Auto-Start
**Duration:** 3 hours
**Files:**
- `src-tauri/src/autostart/mod.rs` - Platform abstraction
- `src-tauri/src/autostart/linux.rs` - systemd/XDG autostart
- `src-tauri/src/autostart/macos.rs` - launchd
- `src-tauri/src/autostart/windows.rs` - Registry/Task Scheduler

**Features:**
```rust
// Auto-start management
enable_autostart() -> Result<()>
disable_autostart() -> Result<()>
is_autostart_enabled() -> Result<bool>

// Configuration
set_autostart_minimized(enabled: bool) -> Result<()>
set_autostart_delay(seconds: u64) -> Result<()>
```

**Platform Implementation:**
- **Linux:** Create `~/.config/autostart/pulsar-desktop.desktop`
- **macOS:** Create `~/Library/LaunchAgents/com.singulio.pulsar.plist`
- **Windows:** Add registry key to `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

#### Task 2.7: Daemon Status UI
**Duration:** 2 hours

**Enhancements:**
- Daemon status indicator in navbar (running/stopped)
- Daemon control menu (start/stop/restart)
- Connection status (WebSocket/IPC)
- Session count display
- Quick actions (attach to session, create new)

---

## 🎯 Track 3: Essential UI Polish (Days 7-9)

### Day 7: Loading States

#### Task 3.1: Loading Component Library
**Duration:** 2 hours
**File:** `src/components/ui/Loading.tsx`

**Components:**
```typescript
<Spinner size="sm|md|lg" />
<Skeleton width={100} height={20} />
<ProgressBar value={50} max={100} />
<LoadingOverlay message="Loading..." />
<InlineLoader text="Loading sessions..." />
```

#### Task 3.2: Add Loading States
**Duration:** 3 hours

**Components to Update:**
- ConnectionDialog (connecting state)
- VaultUnlockDialog (unlocking/initializing state)
- FileTransferView (loading transfers)
- SessionTabs (loading sessions)
- SettingsDialog (loading/saving settings)
- WorkspaceDialog (loading workspaces)

**Pattern:**
```typescript
const [loading, setLoading] = useState(false)
const [error, setError] = useState<string | null>(null)

if (loading) return <LoadingOverlay message="Loading..." />
if (error) return <ErrorDisplay error={error} onRetry={handleRetry} />
```

---

### Day 8: Error Handling

#### Task 3.3: Error Component Library
**Duration:** 2 hours
**File:** `src/components/ui/ErrorDisplay.tsx`

**Components:**
```typescript
<ErrorBanner message="Connection failed" onDismiss={() => {}} />
<ErrorDialog title="Error" message="..." onClose={() => {}} />
<ErrorToast message="Failed to save" type="error|warning|info" />
<InlineError message="Invalid input" />
```

#### Task 3.4: Improve Error Handling
**Duration:** 3 hours

**Error Categories:**
- **Connection errors:** Network issues, timeouts, auth failures
- **File errors:** Transfer failures, file not found, permission denied
- **Vault errors:** Wrong password, decryption failed, locked vault
- **Session errors:** Session not found, detach failed, daemon unavailable
- **Settings errors:** Invalid config, save failed, import failed

**Error Handling Pattern:**
```typescript
try {
  await performAction()
} catch (error) {
  if (error instanceof ConnectionError) {
    showErrorToast(`Connection failed: ${error.message}`)
    // Offer retry or reconnect
  } else if (error instanceof AuthError) {
    showErrorDialog('Authentication Failed', error.message)
    // Prompt for new credentials
  } else {
    showErrorDialog('Unexpected Error', error.message)
    // Log to console for debugging
  }
}
```

#### Task 3.5: Error Recovery Actions
**Duration:** 2 hours

**Features:**
- Retry buttons for transient errors
- "Go back" navigation for fatal errors
- Auto-retry with exponential backoff for network errors
- Error reporting button (copy error details)
- Helpful error messages with suggested actions

---

### Day 9: Animations & Keyboard Nav

#### Task 3.6: Add Animations
**Duration:** 2 hours
**File:** `src/styles/animations.css`

**Animations:**
```css
/* Transitions */
- Fade in/out (modals, toasts)
- Slide in/out (sidebars, panels)
- Scale (buttons on hover)
- Shimmer (loading skeletons)

/* CSS classes */
.fade-enter { opacity: 0; }
.fade-enter-active { opacity: 1; transition: opacity 200ms; }
.fade-exit { opacity: 1; }
.fade-exit-active { opacity: 0; transition: opacity 200ms; }

.slide-enter { transform: translateX(-100%); }
.slide-enter-active { transform: translateX(0); transition: transform 300ms; }

.scale-hover:hover { transform: scale(1.05); transition: transform 150ms; }
```

**Apply to:**
- Modal dialogs (fade in)
- Toast notifications (slide in from top)
- Tabs (smooth transition between tabs)
- Dropdown menus (fade + slide)
- Button hover states (scale)
- Loading spinners (rotate animation)

#### Task 3.7: Keyboard Navigation
**Duration:** 3 hours

**Features:**
- **Tab navigation:** All interactive elements reachable
- **Focus indicators:** Visible outline on focused elements
- **Keyboard shortcuts:** Apply global shortcuts from settings
- **Modal trapping:** Tab stays within modal when open
- **Escape key:** Close modals/dialogs
- **Arrow keys:** Navigate lists, tabs, menus
- **Enter key:** Activate buttons, submit forms
- **Space key:** Toggle checkboxes, expand items

**Components to Update:**
```typescript
// Add keyboard handlers
ConnectionDialog: Escape to close, Enter to connect
VaultUnlockDialog: Escape to cancel, Enter to unlock
SettingsDialog: Escape to close, Ctrl+S to save
SessionTabs: Ctrl+Tab to switch, Ctrl+W to close
FileTransferView: Delete key to remove, Enter to retry
```

#### Task 3.8: Final Polish
**Duration:** 2 hours

**Tasks:**
- Fix any layout issues (spacing, alignment)
- Ensure consistent styling across components
- Add tooltips to icon buttons
- Improve mobile responsiveness (if applicable)
- Test all flows end-to-end
- Fix any remaining bugs

---

## ✅ Acceptance Criteria

### Settings UI
- [ ] All 5 settings tabs implemented and functional
- [ ] Settings persist across app restarts
- [ ] Settings apply immediately (or on Save click)
- [ ] Export/import settings works
- [ ] Keyboard shortcut to open settings (Ctrl+,)

### Daemon Completion
- [ ] Sessions persist to database
- [ ] Sessions restore on daemon restart
- [ ] Desktop notifications working on all platforms
- [ ] Auto-start configuration working
- [ ] Daemon status visible in UI

### UI Polish
- [ ] Loading states on all async operations
- [ ] Error handling with recovery actions
- [ ] Smooth animations on modals and transitions
- [ ] Full keyboard navigation support
- [ ] No console errors in production build

---

## 📊 Progress Tracking

### Track 1: Settings UI (0/9 tasks)
- [ ] 1.1: Design Settings data model
- [ ] 1.2: Implement Settings backend
- [ ] 1.3: Settings Dialog component
- [ ] 1.4: Appearance Settings tab
- [ ] 1.5: Connection Settings tab
- [ ] 1.6: Security Settings tab
- [ ] 1.7: Keyboard Shortcuts tab
- [ ] 1.8: General Settings tab
- [ ] 1.9: Settings integration

### Track 2: Daemon Completion (0/7 tasks)
- [ ] 2.1: Session database schema
- [ ] 2.2: Database operations
- [ ] 2.3: Persistence integration
- [ ] 2.4: Notification service
- [ ] 2.5: Notification preferences
- [ ] 2.6: Platform auto-start
- [ ] 2.7: Daemon status UI

### Track 3: Essential UI Polish (0/8 tasks)
- [ ] 3.1: Loading component library
- [ ] 3.2: Add loading states
- [ ] 3.3: Error component library
- [ ] 3.4: Improve error handling
- [ ] 3.5: Error recovery actions
- [ ] 3.6: Add animations
- [ ] 3.7: Keyboard navigation
- [ ] 3.8: Final polish

**Total Progress:** 0/24 tasks (0%)

---

## 🚀 Success Metrics

### Performance
- Settings load in <100ms
- Settings save in <200ms
- Daemon startup in <2s
- No UI freezes or stutters
- Smooth 60fps animations

### User Experience
- Settings are discoverable and intuitive
- Error messages are clear and actionable
- Loading states prevent user confusion
- Keyboard navigation is efficient
- Animations feel polished, not distracting

### Stability
- No crashes or hangs
- All settings persist correctly
- Database operations are atomic
- Notifications work reliably
- Auto-start works after system reboot

---

## 📅 Timeline

**Start:** 2025-11-09 (Today)
**Target End:** 2025-11-18 (MVP Complete)

| Days | Track | Tasks | Deliverable |
|------|-------|-------|-------------|
| 1-3 | Settings UI | 1.1-1.9 | Complete settings system |
| 4-6 | Daemon | 2.1-2.7 | Database, notifications, auto-start |
| 7-9 | Polish | 3.1-3.8 | Loading, errors, animations, keyboard |

**Milestone:** MVP COMPLETE (Day 9) ✅

---

**Let's get started!** 🚀
