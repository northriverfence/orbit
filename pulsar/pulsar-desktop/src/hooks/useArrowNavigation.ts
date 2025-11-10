import { useEffect, useState, RefObject } from 'react'

interface ArrowNavigationOptions {
  containerRef: RefObject<HTMLElement>
  enabled?: boolean
  onSelect?: (index: number) => void
  loop?: boolean
}

/**
 * Enables arrow key navigation for lists
 */
export function useArrowNavigation({
  containerRef,
  enabled = true,
  onSelect,
  loop = true,
}: ArrowNavigationOptions) {
  const [activeIndex, setActiveIndex] = useState(0)

  useEffect(() => {
    if (!enabled || !containerRef.current) return

    const handleKeyDown = (e: KeyboardEvent) => {
      if (!['ArrowUp', 'ArrowDown', 'Enter', 'Home', 'End'].includes(e.key)) {
        return
      }

      e.preventDefault()

      const items = Array.from(
        containerRef.current?.querySelectorAll('[role="option"], [role="menuitem"], .list-item') || []
      )

      if (items.length === 0) return

      let newIndex = activeIndex

      switch (e.key) {
        case 'ArrowDown':
          newIndex = activeIndex + 1
          if (loop && newIndex >= items.length) {
            newIndex = 0
          } else {
            newIndex = Math.min(newIndex, items.length - 1)
          }
          break

        case 'ArrowUp':
          newIndex = activeIndex - 1
          if (loop && newIndex < 0) {
            newIndex = items.length - 1
          } else {
            newIndex = Math.max(newIndex, 0)
          }
          break

        case 'Home':
          newIndex = 0
          break

        case 'End':
          newIndex = items.length - 1
          break

        case 'Enter':
          if (onSelect) {
            onSelect(activeIndex)
          }
          return
      }

      if (newIndex !== activeIndex) {
        setActiveIndex(newIndex)
        ;(items[newIndex] as HTMLElement).focus()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [containerRef, activeIndex, enabled, onSelect, loop])

  return { activeIndex, setActiveIndex }
}
