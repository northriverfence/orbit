/**
 * Split-pane type definitions
 */

export type SplitDirection = 'horizontal' | 'vertical'
export type PanePosition = 'top' | 'bottom' | 'left' | 'right' | 'center'

export interface Pane {
  id: string
  sessionId: string | null
  size: number // percentage (0-100) or pixels
  direction?: SplitDirection
  children?: Pane[]
  minSize?: number // minimum size in pixels
  maxSize?: number // maximum size in pixels
}

export interface SplitPaneLayout {
  id: string
  type: 'single' | 'split'
  direction?: SplitDirection
  panes: Pane[]
  activePane: string | null
}

export interface SplitPaneState {
  layouts: Map<string, SplitPaneLayout>
  activeLayout: string | null
}

export interface ResizeEvent {
  paneId: string
  newSize: number
  direction: SplitDirection
}

export interface SplitEvent {
  paneId: string
  direction: SplitDirection
  ratio?: number // split ratio (0-1), default 0.5
}

export interface RemovePaneEvent {
  paneId: string
}
