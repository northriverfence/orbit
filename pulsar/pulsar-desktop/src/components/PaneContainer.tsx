import { useState, useCallback, useEffect, useRef } from 'react'
import SplitPane from './SplitPane'
import Terminal from './Terminal'
import PulsarTerminal from './PulsarTerminal'
import { splitPaneManager } from '../lib/splitPaneManager'
import type { SplitPaneLayout, SplitDirection } from '../types/splitPane'
import type { Session } from './SessionTabs'

interface SessionConfig {
  host?: string
  port?: number
  username?: string
  password?: string
}

interface SessionData extends Session {
  createdAt: string
  lastActive: string
  sessionConfig?: SessionConfig
}

interface PaneContainerProps {
  sessions: SessionData[]
  onCreateSession: (type: 'local' | 'ssh', config?: SessionConfig) => string
  onCloseSession?: (sessionId: string) => void
}

/**
 * PaneContainer
 *
 * Manages split-pane layouts for terminal sessions.
 * Each pane can contain a different terminal session.
 */
export default function PaneContainer({
  sessions,
  onCreateSession,
  onCloseSession,
}: PaneContainerProps) {
  const layoutIdRef = useRef('default-layout')
  const [layout, setLayout] = useState<SplitPaneLayout | null>(null)
  const [activePane, setActivePane] = useState<string | null>(null)

  // Initialize layout on mount
  useEffect(() => {
    const layoutId = layoutIdRef.current

    // Try to load saved layout from disk
    const savedLayout = loadLayoutFromDisk(layoutId)
    if (savedLayout) {
      setLayout(savedLayout)
      setActivePane(savedLayout.activePane)
    } else {
      // Create new layout with first session
      const firstSessionId = sessions.length > 0 ? sessions[0].id : null
      const newLayout = splitPaneManager.createLayout(layoutId, firstSessionId)
      setLayout(newLayout)
      setActivePane(newLayout.activePane)
    }
  }, [])

  // Auto-save layout when it changes
  useEffect(() => {
    if (layout) {
      saveLayoutToDisk(layout)
    }
  }, [layout])

  // Handle split pane operation
  const handleSplitPane = useCallback(
    (paneId: string, direction: SplitDirection) => {
      if (!layout) return

      // Create a new local terminal session for the new pane
      const newSessionId = onCreateSession('local')

      // Split the pane
      const updatedLayout = splitPaneManager.splitPane(
        layout.id,
        paneId,
        direction,
        0.5 // 50/50 split
      )

      if (updatedLayout) {
        // Assign the new session to the new pane
        const newPane = findNewPane(updatedLayout)
        if (newPane) {
          newPane.sessionId = newSessionId
        }

        setLayout({ ...updatedLayout })
        setActivePane(updatedLayout.activePane)
      }
    },
    [layout, onCreateSession]
  )

  // Handle remove pane operation
  const handleRemovePane = useCallback(
    (paneId: string) => {
      if (!layout) return

      // Find the session associated with this pane
      const pane = findPaneById(layout.panes[0], paneId)
      if (pane && pane.sessionId && onCloseSession) {
        // Optionally close the session
        // onCloseSession(pane.sessionId)
      }

      const updatedLayout = splitPaneManager.removePane(layout.id, paneId)
      if (updatedLayout) {
        setLayout({ ...updatedLayout })
        setActivePane(updatedLayout.activePane)
      }
    },
    [layout, onCloseSession]
  )

  // Handle activate pane operation
  const handleActivatePane = useCallback(
    (paneId: string) => {
      if (!layout) return

      const updatedLayout = splitPaneManager.setActivePane(layout.id, paneId)
      if (updatedLayout) {
        setLayout({ ...updatedLayout })
        setActivePane(paneId)
      }
    },
    [layout]
  )

  // Render terminal content for a pane
  const renderContent = useCallback(
    (sessionId: string | null, paneId: string) => {
      if (!sessionId) {
        // Empty pane - show placeholder
        return (
          <div className="h-full w-full flex items-center justify-center bg-gray-900 text-gray-400">
            <div className="text-center">
              <div className="text-4xl mb-2">🖥️</div>
              <p className="text-sm">No terminal session</p>
              <button
                onClick={() => {
                  const newSessionId = onCreateSession('local')
                  // Update pane with new session
                  if (layout) {
                    const pane = findPaneById(layout.panes[0], paneId)
                    if (pane) {
                      pane.sessionId = newSessionId
                      setLayout({ ...layout })
                    }
                  }
                }}
                className="mt-4 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors text-xs"
              >
                Create Terminal
              </button>
            </div>
          </div>
        )
      }

      // Find session data
      const session = sessions.find((s) => s.id === sessionId)
      if (!session) {
        return (
          <div className="h-full w-full flex items-center justify-center bg-gray-900 text-gray-400">
            <p className="text-sm">Session not found: {sessionId}</p>
          </div>
        )
      }

      // Render appropriate terminal based on session type
      if (session.type === 'ssh' && session.sessionConfig) {
        return (
          <Terminal
            sessionId={session.id}
            host={session.sessionConfig.host!}
            port={session.sessionConfig.port!}
            username={session.sessionConfig.username!}
            password={session.sessionConfig.password}
          />
        )
      } else if (session.type === 'local') {
        return <PulsarTerminal />
      }

      return (
        <div className="h-full w-full flex items-center justify-center bg-gray-900 text-gray-400">
          <p className="text-sm">Unknown session type</p>
        </div>
      )
    },
    [sessions, onCreateSession, layout]
  )

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!layout || !activePane) return

      // Ctrl+Shift+H - Horizontal split
      if (e.ctrlKey && e.shiftKey && e.key === 'H') {
        e.preventDefault()
        handleSplitPane(activePane, 'horizontal')
      }

      // Ctrl+Shift+V - Vertical split
      if (e.ctrlKey && e.shiftKey && e.key === 'V') {
        e.preventDefault()
        handleSplitPane(activePane, 'vertical')
      }

      // Ctrl+Shift+W - Close pane
      if (e.ctrlKey && e.shiftKey && e.key === 'W') {
        e.preventDefault()
        const totalPanes = countPanes(layout.panes[0])
        if (totalPanes > 1) {
          handleRemovePane(activePane)
        }
      }

      // Ctrl+Shift+Arrow Keys - Navigate panes
      if (e.ctrlKey && e.shiftKey && (e.key === 'ArrowUp' || e.key === 'ArrowDown' || e.key === 'ArrowLeft' || e.key === 'ArrowRight')) {
        e.preventDefault()
        // TODO: Implement pane navigation
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [layout, activePane, handleSplitPane, handleRemovePane])

  if (!layout) {
    return (
      <div className="h-full w-full flex items-center justify-center bg-gray-900">
        <div className="text-gray-400">Loading layout...</div>
      </div>
    )
  }

  return (
    <div className="h-full w-full">
      <SplitPane
        pane={layout.panes[0]}
        onSplitPane={handleSplitPane}
        onRemovePane={handleRemovePane}
        onActivatePane={handleActivatePane}
        activePane={activePane}
        renderContent={renderContent}
      />
    </div>
  )
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Find a pane by ID in the tree
 */
function findPaneById(root: any, paneId: string): any {
  if (root.id === paneId) return root

  if (root.children) {
    for (const child of root.children) {
      const found = findPaneById(child, paneId)
      if (found) return found
    }
  }

  return null
}

/**
 * Find the new pane (the one without a sessionId)
 */
function findNewPane(layout: SplitPaneLayout): any {
  const findInTree = (pane: any): any => {
    if (!pane.sessionId && !pane.children) {
      return pane
    }

    if (pane.children) {
      for (const child of pane.children) {
        const found = findInTree(child)
        if (found) return found
      }
    }

    return null
  }

  return findInTree(layout.panes[0])
}

/**
 * Count total panes in the tree
 */
function countPanes(root: any): number {
  if (!root.children || root.children.length === 0) return 1
  return root.children.reduce((sum: number, child: any) => sum + countPanes(child), 0)
}

/**
 * Save layout to disk
 */
function saveLayoutToDisk(layout: SplitPaneLayout): void {
  try {
    const serialized = JSON.stringify(layout, null, 2)
    localStorage.setItem(`pulsar-layout-${layout.id}`, serialized)
    console.log(`Saved layout ${layout.id} to disk`)
  } catch (error) {
    console.error('Failed to save layout:', error)
  }
}

/**
 * Load layout from disk
 */
function loadLayoutFromDisk(layoutId: string): SplitPaneLayout | null {
  try {
    const serialized = localStorage.getItem(`pulsar-layout-${layoutId}`)
    if (serialized) {
      const layout = JSON.parse(serialized) as SplitPaneLayout
      console.log(`Loaded layout ${layoutId} from disk`)
      return layout
    }
  } catch (error) {
    console.error('Failed to load layout:', error)
  }
  return null
}
