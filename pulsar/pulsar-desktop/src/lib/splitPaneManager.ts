/**
 * Split Pane Manager
 *
 * Manages the state and operations for split-pane layouts
 */

import type { Pane, SplitDirection, SplitPaneLayout } from '../types/splitPane'

export class SplitPaneManager {
  private layouts: Map<string, SplitPaneLayout>
  private activeLayout: string | null

  constructor() {
    this.layouts = new Map()
    this.activeLayout = null
  }

  /**
   * Create a new layout with a single pane
   */
  createLayout(layoutId: string, sessionId: string | null = null): SplitPaneLayout {
    const pane: Pane = {
      id: this.generatePaneId(),
      sessionId,
      size: 100,
      minSize: 100,
    }

    const layout: SplitPaneLayout = {
      id: layoutId,
      type: 'single',
      panes: [pane],
      activePane: pane.id,
    }

    this.layouts.set(layoutId, layout)
    this.activeLayout = layoutId

    return layout
  }

  /**
   * Get a layout by ID
   */
  getLayout(layoutId: string): SplitPaneLayout | undefined {
    return this.layouts.get(layoutId)
  }

  /**
   * Get the active layout
   */
  getActiveLayout(): SplitPaneLayout | null {
    if (!this.activeLayout) return null
    return this.layouts.get(this.activeLayout) || null
  }

  /**
   * Split a pane in the specified direction
   */
  splitPane(
    layoutId: string,
    paneId: string,
    direction: SplitDirection,
    ratio: number = 0.5
  ): SplitPaneLayout | null {
    const layout = this.layouts.get(layoutId)
    if (!layout) return null

    const paneToSplit = this.findPane(layout.panes[0], paneId)
    if (!paneToSplit) return null

    // Create new pane
    const newPane: Pane = {
      id: this.generatePaneId(),
      sessionId: null,
      size: (1 - ratio) * 100,
      minSize: 100,
    }

    // Modify existing pane
    paneToSplit.size = ratio * 100
    paneToSplit.direction = direction

    // If pane already has children, add new pane to children
    if (paneToSplit.children) {
      paneToSplit.children.push(newPane)
    } else {
      // Convert leaf pane to split pane
      const originalSessionId = paneToSplit.sessionId
      paneToSplit.sessionId = null

      // Create child for original content
      const originalPane: Pane = {
        id: this.generatePaneId(),
        sessionId: originalSessionId,
        size: ratio * 100,
        minSize: 100,
      }

      paneToSplit.children = [originalPane, newPane]
    }

    // Update layout type
    layout.type = 'split'
    layout.activePane = newPane.id

    this.layouts.set(layoutId, layout)
    return layout
  }

  /**
   * Remove a pane from the layout
   */
  removePane(layoutId: string, paneId: string): SplitPaneLayout | null {
    const layout = this.layouts.get(layoutId)
    if (!layout) return null

    // Can't remove the last pane
    const totalPanes = this.countPanes(layout.panes[0])
    if (totalPanes <= 1) return layout

    // Find parent and remove pane
    const removed = this.removePaneRecursive(layout.panes[0], paneId)
    if (!removed) return null

    // If active pane was removed, select another
    if (layout.activePane === paneId) {
      layout.activePane = this.getFirstLeafPaneId(layout.panes[0])
    }

    // Update layout type
    const remaining = this.countPanes(layout.panes[0])
    if (remaining === 1) {
      layout.type = 'single'
    }

    this.layouts.set(layoutId, layout)
    return layout
  }

  /**
   * Set the active pane
   */
  setActivePane(layoutId: string, paneId: string): SplitPaneLayout | null {
    const layout = this.layouts.get(layoutId)
    if (!layout) return null

    layout.activePane = paneId
    this.layouts.set(layoutId, layout)
    return layout
  }

  /**
   * Resize a pane
   */
  resizePane(layoutId: string, paneId: string, newSize: number): SplitPaneLayout | null {
    const layout = this.layouts.get(layoutId)
    if (!layout) return null

    const pane = this.findPane(layout.panes[0], paneId)
    if (!pane) return null

    pane.size = Math.max(10, Math.min(90, newSize))

    this.layouts.set(layoutId, layout)
    return layout
  }

  /**
   * Save layout to JSON
   */
  serializeLayout(layoutId: string): string | null {
    const layout = this.layouts.get(layoutId)
    if (!layout) return null

    return JSON.stringify(layout, null, 2)
  }

  /**
   * Load layout from JSON
   */
  deserializeLayout(layoutId: string, json: string): SplitPaneLayout | null {
    try {
      const layout: SplitPaneLayout = JSON.parse(json)
      layout.id = layoutId
      this.layouts.set(layoutId, layout)
      return layout
    } catch (error) {
      console.error('Failed to deserialize layout:', error)
      return null
    }
  }

  /**
   * Helper: Find a pane by ID in the tree
   */
  private findPane(root: Pane, paneId: string): Pane | null {
    if (root.id === paneId) return root

    if (root.children) {
      for (const child of root.children) {
        const found = this.findPane(child, paneId)
        if (found) return found
      }
    }

    return null
  }

  /**
   * Helper: Count total panes in the tree
   */
  private countPanes(root: Pane): number {
    if (!root.children || root.children.length === 0) return 1

    return root.children.reduce((sum, child) => sum + this.countPanes(child), 0)
  }

  /**
   * Helper: Get the first leaf pane ID
   */
  private getFirstLeafPaneId(root: Pane): string {
    if (!root.children || root.children.length === 0) return root.id
    return this.getFirstLeafPaneId(root.children[0])
  }

  /**
   * Helper: Remove a pane recursively
   */
  private removePaneRecursive(parent: Pane, paneId: string): boolean {
    if (!parent.children) return false

    // Find the pane in children
    const index = parent.children.findIndex((child) => child.id === paneId)
    if (index !== -1) {
      parent.children.splice(index, 1)

      // If only one child left, collapse
      if (parent.children.length === 1) {
        const remaining = parent.children[0]
        parent.sessionId = remaining.sessionId
        parent.children = remaining.children
        parent.direction = remaining.direction
      }

      return true
    }

    // Recurse into children
    for (const child of parent.children) {
      if (this.removePaneRecursive(child, paneId)) {
        return true
      }
    }

    return false
  }

  /**
   * Helper: Generate unique pane ID
   */
  private generatePaneId(): string {
    return `pane-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
  }
}

// Singleton instance
export const splitPaneManager = new SplitPaneManager()
