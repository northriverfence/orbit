# Section A + C Implementation Plan

**Date**: 2025-11-06
**Goal**: Complete Section A (100%) + Implement Section C (Workspace Management)
**Estimated Time**: 3-4 weeks

---

## 📋 Features to Implement

### Section A: Session Management (Complete to 100%)
1. ✅ Split-pane terminal view (horizontal/vertical splits)
2. ✅ Command history tracking and search
3. ✅ Session replay (record and playback terminal sessions)

### Section C: Workspace Management (0% → 100%)
4. ✅ Workspace data model and backend
5. ✅ Workspace switcher UI
6. ✅ Save/load workspace layouts
7. ✅ Workspace templates

---

## 🏗️ Implementation Order

### Phase 1: Split-Pane View (Week 1, Days 1-3)

**Why First**: Foundation for workspace layouts

**Components**:
1. **SplitPane.tsx** - React component for split views
2. **PaneContainer.tsx** - Container managing multiple panes
3. **Resizer.tsx** - Draggable divider between panes
4. **Split logic** - Horizontal/vertical split management

**Features**:
- Drag-to-resize panes
- Minimum pane size enforcement (200px)
- Nested splits (split within split)
- Close pane button
- Keyboard shortcuts (Ctrl+Shift+H/V for split)
- Save pane layout state

**Technical Details**:
```typescript
interface Pane {
  id: string
  sessionId: string
  size: number // percentage or pixels
  position: 'top' | 'bottom' | 'left' | 'right'
  children?: Pane[] // for nested splits
}

interface Layout {
  type: 'single' | 'horizontal' | 'vertical'
  panes: Pane[]
  activePane: string
}
```

**Files**:
- `src/components/SplitPane.tsx` (new)
- `src/components/PaneContainer.tsx` (new)
- `src/components/Resizer.tsx` (new)
- `src/lib/splitPaneManager.ts` (new)

---

### Phase 2: Command History (Week 1, Days 4-5)

**Why Second**: Enhances terminal, independent feature

**Components**:
1. **CommandHistory.tsx** - History panel UI
2. **HistorySearch.tsx** - Search/filter interface
3. **historyStorage.ts** - Persistence layer

**Features**:
- Capture all commands typed in terminal
- Store with timestamp and session context
- Search/filter by keyword
- Click to re-execute command
- Export history to file
- Ctrl+R for reverse search (like bash)
- Per-session and global history

**Technical Details**:
```typescript
interface CommandHistoryEntry {
  id: string
  sessionId: string
  command: string
  timestamp: string
  exitCode?: number
  duration?: number // milliseconds
}

interface HistoryState {
  entries: CommandHistoryEntry[]
  maxEntries: number // default 10000
}
```

**Storage**:
- `~/.config/pulsar/command_history.json`
- Auto-save with debounce (1 second)
- Rotation when maxEntries exceeded

**Files**:
- `src/components/CommandHistory.tsx` (new)
- `src/components/HistorySearch.tsx` (new)
- `src/lib/historyStorage.ts` (new)
- `src/lib/commandCapture.ts` (new)

---

### Phase 3: Session Replay (Week 1-2, Days 6-8)

**Why Third**: Advanced feature, builds on history

**Components**:
1. **SessionRecorder.ts** - Capture terminal output
2. **SessionPlayer.tsx** - Playback UI
3. **replayStorage.ts** - Recording persistence

**Features**:
- Record terminal output with timing
- Pause/resume recording
- Playback with speed control (0.5x, 1x, 2x, 4x)
- Seek to timestamp
- Export recording (asciicast v2 format)
- Import existing recordings
- Recording size management

**Technical Details**:
```typescript
interface RecordingFrame {
  timestamp: number // milliseconds from start
  type: 'output' | 'input'
  data: string
}

interface Recording {
  id: string
  sessionId: string
  name: string
  startTime: string
  duration: number // milliseconds
  frames: RecordingFrame[]
  metadata: {
    width: number
    height: number
    shell: string
  }
}
```

**Format**: asciicast v2 (compatible with asciinema)
```json
{
  "version": 2,
  "width": 80,
  "height": 24,
  "timestamp": 1699999999,
  "env": {"SHELL": "/bin/bash"},
  "title": "Session recording"
}
[0.0, "o", "$ "]
[1.5, "o", "ls\r\n"]
```

**Files**:
- `src/lib/sessionRecorder.ts` (new)
- `src/components/SessionPlayer.tsx` (new)
- `src/lib/replayStorage.ts` (new)
- `src/lib/asciicast.ts` (new)

---

### Phase 4: Workspace Backend (Week 2, Days 9-11)

**Why Fourth**: Backend foundation for workspaces

**Components**:
1. **Workspace schema** - PostgreSQL/SQLite tables
2. **Workspace service** - Rust backend service
3. **Workspace API** - gRPC/REST endpoints

**Database Schema**:
```sql
CREATE TABLE workspaces (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  description TEXT,
  layout JSONB NOT NULL,  -- pane structure
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW(),
  is_template BOOLEAN DEFAULT FALSE,
  icon VARCHAR(50),
  tags TEXT[]
);

CREATE TABLE workspace_sessions (
  workspace_id UUID REFERENCES workspaces(id),
  session_id UUID,
  pane_id VARCHAR(50),
  position INTEGER,
  PRIMARY KEY (workspace_id, session_id)
);

CREATE TABLE workspace_snapshots (
  id UUID PRIMARY KEY,
  workspace_id UUID REFERENCES workspaces(id),
  name VARCHAR(255),
  layout JSONB NOT NULL,
  created_at TIMESTAMP DEFAULT NOW()
);
```

**Workspace Service** (Rust):
```rust
pub struct WorkspaceService {
    db: Arc<DatabasePool>,
}

impl WorkspaceService {
    pub async fn create_workspace(&self, workspace: Workspace) -> Result<Workspace>
    pub async fn get_workspace(&self, id: Uuid) -> Result<Workspace>
    pub async fn list_workspaces(&self) -> Result<Vec<Workspace>>
    pub async fn update_workspace(&self, id: Uuid, workspace: Workspace) -> Result<Workspace>
    pub async fn delete_workspace(&self, id: Uuid) -> Result<()>
    pub async fn save_snapshot(&self, workspace_id: Uuid, name: String) -> Result<Snapshot>
}
```

**Files**:
- `pulsar-daemon/src/workspace/mod.rs` (new)
- `pulsar-daemon/src/workspace/service.rs` (new)
- `pulsar-daemon/src/workspace/models.rs` (new)
- `pulsar-daemon/migrations/003_workspaces.sql` (new)

---

### Phase 5: Workspace UI (Week 2-3, Days 12-15)

**Why Fifth**: User interface for workspace management

**Components**:
1. **WorkspaceSwitcher.tsx** - Dropdown/sidebar selector
2. **WorkspacePanel.tsx** - Workspace management panel
3. **WorkspaceDialog.tsx** - Create/edit dialog
4. **WorkspaceCard.tsx** - Visual workspace card

**Features**:
- Workspace selector (dropdown in toolbar)
- Workspace grid/list view
- Create new workspace dialog
- Edit workspace (name, icon, description)
- Delete workspace with confirmation
- Active workspace indicator
- Keyboard shortcuts (Ctrl+1-9 for quick switch)

**UI Design**:
```
┌────────────────────────────────────────┐
│ [Workspace Dropdown ▼] [+ New]        │
├────────────────────────────────────────┤
│  📁 DevOps (Active)                    │
│  🚀 Frontend                           │
│  🔧 Backend                            │
│  📊 Analytics                          │
└────────────────────────────────────────┘
```

**Files**:
- `src/components/WorkspaceSwitcher.tsx` (new)
- `src/components/WorkspacePanel.tsx` (new)
- `src/components/WorkspaceDialog.tsx` (new)
- `src/components/WorkspaceCard.tsx` (new)

---

### Phase 6: Save/Load Workspace Layouts (Week 3, Days 16-18)

**Why Sixth**: Persistence and restore functionality

**Components**:
1. **workspaceManager.ts** - Layout save/load logic
2. **layoutSerializer.ts** - Layout to JSON conversion
3. **layoutRestorer.ts** - Restore sessions from layout

**Features**:
- Save current layout as workspace
- Load workspace (restore all sessions and panes)
- Auto-save on workspace switch
- Workspace snapshots (save versions)
- Export workspace to file
- Import workspace from file
- Conflict resolution (existing sessions)

**Technical Details**:
```typescript
interface WorkspaceLayout {
  version: string
  workspaceId: string
  name: string
  panes: PaneLayout[]
  sessions: SessionConfig[]
  activePane: string
  metadata: {
    created: string
    updated: string
  }
}

interface PaneLayout {
  id: string
  sessionId: string
  type: 'terminal' | 'editor' | 'browser'
  position: {
    x: number
    y: number
    width: number
    height: number
  }
  split?: {
    direction: 'horizontal' | 'vertical'
    ratio: number
    children: PaneLayout[]
  }
}
```

**Files**:
- `src/lib/workspaceManager.ts` (new)
- `src/lib/layoutSerializer.ts` (new)
- `src/lib/layoutRestorer.ts` (new)

---

### Phase 7: Workspace Templates (Week 3, Days 19-21)

**Why Last**: Nice-to-have feature building on everything

**Components**:
1. **WorkspaceTemplate.tsx** - Template selector
2. **TemplateGallery.tsx** - Template browser
3. **templates/** - Predefined template files

**Features**:
- Predefined workspace templates
- Template gallery (visual browser)
- Create workspace from template
- Save current workspace as template
- Share templates (export/import)
- Template categories (Development, DevOps, Data Science)

**Built-in Templates**:

1. **DevOps Dashboard** (4 panes)
   - Top-left: Kubernetes cluster
   - Top-right: Application logs
   - Bottom-left: Monitoring (htop)
   - Bottom-right: Local terminal

2. **Full-Stack Development** (3 panes)
   - Left: Backend server (npm run dev)
   - Top-right: Frontend server (npm start)
   - Bottom-right: Database client

3. **Microservices** (6 panes)
   - Grid layout with 6 services
   - Each pane: different microservice

4. **Data Science** (2 panes)
   - Left: Jupyter notebook server
   - Right: Data processing script

**Template Format**:
```json
{
  "name": "DevOps Dashboard",
  "description": "Monitor Kubernetes cluster and application logs",
  "icon": "🚀",
  "category": "DevOps",
  "layout": {
    "type": "grid",
    "panes": [
      {
        "id": "k8s",
        "title": "Kubernetes",
        "command": "kubectl get pods -w",
        "position": "top-left"
      },
      {
        "id": "logs",
        "title": "Logs",
        "command": "tail -f /var/log/app.log",
        "position": "top-right"
      }
    ]
  }
}
```

**Files**:
- `src/components/WorkspaceTemplate.tsx` (new)
- `src/components/TemplateGallery.tsx` (new)
- `src/templates/devops-dashboard.json` (new)
- `src/templates/fullstack-dev.json` (new)
- `src/templates/microservices.json` (new)
- `src/templates/data-science.json` (new)

---

## 📊 Progress Tracking

### Section A: Session Management
| Feature | Status | Completion |
|---------|--------|------------|
| Multi-session tabs | ✅ Complete | 100% |
| Session persistence | ✅ Complete | 100% |
| Keyboard shortcuts | ✅ Complete | 100% |
| Context menus | ✅ Complete | 100% |
| **Split-pane view** | ⏸️ Pending | 0% |
| **Command history** | ⏸️ Pending | 0% |
| **Session replay** | ⏸️ Pending | 0% |

**Section A Target**: 85% → 100% (+15%)

### Section C: Workspace Management
| Feature | Status | Completion |
|---------|--------|------------|
| **Workspace backend** | ⏸️ Pending | 0% |
| **Workspace UI** | ⏸️ Pending | 0% |
| **Save/load layouts** | ⏸️ Pending | 0% |
| **Workspace templates** | ⏸️ Pending | 0% |

**Section C Target**: 0% → 100% (+100%)

---

## 🎯 Timeline

| Week | Days | Features | Deliverable |
|------|------|----------|-------------|
| **Week 1** | 1-3 | Split-pane view | Resizable terminal panes |
| **Week 1** | 4-5 | Command history | History search panel |
| **Week 1-2** | 6-8 | Session replay | Record/playback terminal |
| **Week 2** | 9-11 | Workspace backend | Database + API |
| **Week 2-3** | 12-15 | Workspace UI | Switcher + management panel |
| **Week 3** | 16-18 | Save/load layouts | Workspace persistence |
| **Week 3** | 19-21 | Workspace templates | Template gallery |

**Total**: 21 days (3 weeks)

---

## 🚀 Expected Outcomes

### Section A: 100% Complete ✅
- Professional multi-tab terminal
- Split-pane layouts
- Full command history with search
- Session recording and playback

### Section C: 100% Complete ✅
- Complete workspace management system
- Visual workspace switcher
- Save/restore layouts
- Template library

### Overall Project Progress
- Backend: 85% → 90% (+5%)
- Frontend: 65% → 85% (+20%)
- **Overall: 70% → 85% (+15%)**

---

## 📁 File Structure

```
pulsar/
├── pulsar-daemon/
│   ├── src/
│   │   └── workspace/
│   │       ├── mod.rs
│   │       ├── service.rs
│   │       └── models.rs
│   └── migrations/
│       └── 003_workspaces.sql
│
└── pulsar-desktop/
    ├── src/
    │   ├── components/
    │   │   ├── SplitPane.tsx
    │   │   ├── PaneContainer.tsx
    │   │   ├── Resizer.tsx
    │   │   ├── CommandHistory.tsx
    │   │   ├── HistorySearch.tsx
    │   │   ├── SessionPlayer.tsx
    │   │   ├── WorkspaceSwitcher.tsx
    │   │   ├── WorkspacePanel.tsx
    │   │   ├── WorkspaceDialog.tsx
    │   │   ├── WorkspaceCard.tsx
    │   │   ├── WorkspaceTemplate.tsx
    │   │   └── TemplateGallery.tsx
    │   │
    │   ├── lib/
    │   │   ├── splitPaneManager.ts
    │   │   ├── historyStorage.ts
    │   │   ├── commandCapture.ts
    │   │   ├── sessionRecorder.ts
    │   │   ├── replayStorage.ts
    │   │   ├── asciicast.ts
    │   │   ├── workspaceManager.ts
    │   │   ├── layoutSerializer.ts
    │   │   └── layoutRestorer.ts
    │   │
    │   └── templates/
    │       ├── devops-dashboard.json
    │       ├── fullstack-dev.json
    │       ├── microservices.json
    │       └── data-science.json
    │
    └── tests/
        ├── splitPane.test.ts
        ├── commandHistory.test.ts
        ├── sessionReplay.test.ts
        └── workspaceManager.test.ts
```

---

## 🧪 Testing Strategy

### Unit Tests
- Split-pane resize logic
- Command history capture
- Recording frame timing
- Layout serialization
- Workspace CRUD operations

### Integration Tests
- Full split-pane workflow
- History search and filter
- Playback accuracy
- Workspace save/restore
- Template instantiation

### E2E Tests
- Create split layout manually
- Record and playback session
- Switch workspaces
- Load template workspace

---

## 📚 Documentation

### User Documentation
- Split-pane keyboard shortcuts
- History search tutorial
- Recording session guide
- Workspace management guide
- Template creation guide

### Developer Documentation
- Split-pane component API
- Workspace data model
- Layout serialization format
- Template schema

---

## ✅ Success Criteria

- [ ] Split-pane works smoothly with drag resize
- [ ] Command history captures all commands accurately
- [ ] Session replay plays back correctly at all speeds
- [ ] Workspaces save and restore completely
- [ ] Template gallery is visually appealing
- [ ] No performance degradation with 20+ sessions
- [ ] All features work on Linux, macOS, Windows

---

**Status**: 🟢 READY TO START
**First Task**: Implement split-pane view
**Estimated Completion**: 3 weeks from start
