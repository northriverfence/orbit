import { useEffect, useCallback } from 'react'

export interface KeyboardShortcut {
  key: string
  ctrlKey?: boolean
  shiftKey?: boolean
  altKey?: boolean
  metaKey?: boolean
  preventDefault?: boolean
}

export function useKeyboardShortcut(
  shortcut: KeyboardShortcut | KeyboardShortcut[],
  callback: () => void,
  enabled: boolean = true
) {
  const shortcuts = Array.isArray(shortcut) ? shortcut : [shortcut]

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!enabled) return

      const matchesShortcut = shortcuts.some((sc) => {
        const keyMatches = event.key.toLowerCase() === sc.key.toLowerCase()
        const ctrlMatches = sc.ctrlKey === undefined || event.ctrlKey === sc.ctrlKey
        const shiftMatches = sc.shiftKey === undefined || event.shiftKey === sc.shiftKey
        const altMatches = sc.altKey === undefined || event.altKey === sc.altKey
        const metaMatches = sc.metaKey === undefined || event.metaKey === sc.metaKey

        return keyMatches && ctrlMatches && shiftMatches && altMatches && metaMatches
      })

      if (matchesShortcut) {
        const shouldPreventDefault = shortcuts.some((sc) => sc.preventDefault !== false)
        if (shouldPreventDefault) {
          event.preventDefault()
        }
        callback()
      }
    },
    [shortcuts, callback, enabled]
  )

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleKeyDown])
}

// Common shortcuts
export const SHORTCUTS = {
  // Navigation
  ESCAPE: { key: 'Escape' },
  ENTER: { key: 'Enter' },
  SPACE: { key: ' ' },

  // Editing
  SAVE: [
    { key: 's', ctrlKey: true, preventDefault: true },
    { key: 's', metaKey: true, preventDefault: true },
  ],
  COPY: [
    { key: 'c', ctrlKey: true },
    { key: 'c', metaKey: true },
  ],
  PASTE: [
    { key: 'v', ctrlKey: true },
    { key: 'v', metaKey: true },
  ],

  // Tabs
  NEW_TAB: [
    { key: 't', ctrlKey: true, preventDefault: true },
    { key: 't', metaKey: true, preventDefault: true },
  ],
  CLOSE_TAB: [
    { key: 'w', ctrlKey: true, preventDefault: true },
    { key: 'w', metaKey: true, preventDefault: true },
  ],
  NEXT_TAB: [
    { key: 'Tab', ctrlKey: true, preventDefault: true },
    { key: 'Tab', metaKey: true, preventDefault: true },
  ],
  PREV_TAB: [
    { key: 'Tab', ctrlKey: true, shiftKey: true, preventDefault: true },
    { key: 'Tab', metaKey: true, shiftKey: true, preventDefault: true },
  ],

  // Search
  FIND: [
    { key: 'f', ctrlKey: true, preventDefault: true },
    { key: 'f', metaKey: true, preventDefault: true },
  ],

  // Settings
  SETTINGS: [
    { key: ',', ctrlKey: true, preventDefault: true },
    { key: ',', metaKey: true, preventDefault: true },
  ],

  // Arrow navigation
  ARROW_UP: { key: 'ArrowUp' },
  ARROW_DOWN: { key: 'ArrowDown' },
  ARROW_LEFT: { key: 'ArrowLeft' },
  ARROW_RIGHT: { key: 'ArrowRight' },
}
