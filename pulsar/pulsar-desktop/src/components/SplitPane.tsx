import { useState, useCallback, useRef, useEffect } from 'react'
import Resizer from './Resizer'
import type { Pane, SplitDirection } from '../types/splitPane'

interface SplitPaneProps {
  pane: Pane
  onSplitPane: (paneId: string, direction: SplitDirection) => void
  onRemovePane: (paneId: string) => void
  onActivatePane: (paneId: string) => void
  activePane: string | null
  renderContent: (sessionId: string | null, paneId: string) => React.ReactNode
}

export default function SplitPane({
  pane,
  onSplitPane,
  onRemovePane,
  onActivatePane,
  activePane,
  renderContent,
}: SplitPaneProps) {
  const [sizes, setSizes] = useState<number[]>(() => {
    if (!pane.children || pane.children.length === 0) return [100]
    return pane.children.map((child) => child.size)
  })

  const containerRef = useRef<HTMLDivElement>(null)

  // Update sizes when pane structure changes
  useEffect(() => {
    if (!pane.children || pane.children.length === 0) {
      setSizes([100])
    } else {
      setSizes(pane.children.map((child) => child.size))
    }
  }, [pane.children])

  const handleResize = useCallback(
    (index: number, delta: number) => {
      if (!pane.children || !containerRef.current) return

      const containerSize =
        pane.direction === 'horizontal'
          ? containerRef.current.clientHeight
          : containerRef.current.clientWidth

      const minSize = pane.children[index].minSize || 100
      const maxSize = pane.children[index].maxSize || containerSize * 0.9

      const deltaPercent = (delta / containerSize) * 100

      setSizes((prevSizes) => {
        const newSizes = [...prevSizes]

        // Adjust current and next pane
        const newCurrentSize = Math.max(
          (minSize / containerSize) * 100,
          Math.min((maxSize / containerSize) * 100, newSizes[index] + deltaPercent)
        )

        const adjustment = newCurrentSize - newSizes[index]
        newSizes[index] = newCurrentSize

        // Adjust next pane
        if (index + 1 < newSizes.length) {
          const nextMinSize = pane.children![index + 1].minSize || 100
          const nextMaxSize = pane.children![index + 1].maxSize || containerSize * 0.9

          newSizes[index + 1] = Math.max(
            (nextMinSize / containerSize) * 100,
            Math.min((nextMaxSize / containerSize) * 100, newSizes[index + 1] - adjustment)
          )
        }

        // Normalize to 100%
        const total = newSizes.reduce((sum, size) => sum + size, 0)
        return newSizes.map((size) => (size / total) * 100)
      })
    },
    [pane.children, pane.direction]
  )

  const handleRemove = useCallback(() => {
    onRemovePane(pane.id)
  }, [pane.id, onRemovePane])

  const handleSplitHorizontal = useCallback(() => {
    onSplitPane(pane.id, 'horizontal')
  }, [pane.id, onSplitPane])

  const handleSplitVertical = useCallback(() => {
    onSplitPane(pane.id, 'vertical')
  }, [pane.id, onSplitPane])

  const handleActivate = useCallback(() => {
    onActivatePane(pane.id)
  }, [pane.id, onActivatePane])

  const isActive = activePane === pane.id
  const hasChildren = pane.children && pane.children.length > 0

  // Render leaf pane (no children)
  if (!hasChildren) {
    return (
      <div
        ref={containerRef}
        className={`relative flex flex-col h-full w-full border-2 ${
          isActive ? 'border-blue-500' : 'border-gray-300'
        } rounded-lg overflow-hidden transition-colors`}
        onClick={handleActivate}
      >
        {/* Pane header */}
        <div className="flex-shrink-0 bg-gray-100 border-b border-gray-300 px-2 py-1 flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <span className="text-xs font-medium text-gray-700">Pane {pane.id.slice(0, 8)}</span>
            {isActive && (
              <span className="w-2 h-2 bg-green-500 rounded-full animate-pulse" title="Active" />
            )}
          </div>

          <div className="flex items-center space-x-1">
            {/* Split buttons */}
            <button
              onClick={handleSplitHorizontal}
              className="p-1 hover:bg-gray-200 rounded text-gray-600 hover:text-gray-900 transition-colors"
              title="Split Horizontal (Ctrl+Shift+H)"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M3 12h18M3 6h18M3 18h18"
                />
              </svg>
            </button>

            <button
              onClick={handleSplitVertical}
              className="p-1 hover:bg-gray-200 rounded text-gray-600 hover:text-gray-900 transition-colors"
              title="Split Vertical (Ctrl+Shift+V)"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 3v18M6 3v18M18 3v18"
                />
              </svg>
            </button>

            {/* Close button */}
            <button
              onClick={handleRemove}
              className="p-1 hover:bg-red-100 rounded text-gray-600 hover:text-red-600 transition-colors"
              title="Close Pane (Ctrl+Shift+W)"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        {/* Pane content */}
        <div className="flex-1 overflow-hidden">{renderContent(pane.sessionId, pane.id)}</div>
      </div>
    )
  }

  // Render split pane (has children)
  const isHorizontal = pane.direction === 'horizontal'

  return (
    <div
      ref={containerRef}
      className={`flex ${isHorizontal ? 'flex-col' : 'flex-row'} h-full w-full`}
    >
      {pane.children!.map((child, index) => (
        <div key={child.id} style={{ flexBasis: `${sizes[index]}%`, flexGrow: 0, flexShrink: 0 }}>
          <SplitPane
            pane={child}
            onSplitPane={onSplitPane}
            onRemovePane={onRemovePane}
            onActivatePane={onActivatePane}
            activePane={activePane}
            renderContent={renderContent}
          />

          {/* Resizer between panes */}
          {index < pane.children!.length - 1 && (
            <Resizer direction={pane.direction!} onResize={(delta) => handleResize(index, delta)} />
          )}
        </div>
      ))}
    </div>
  )
}
