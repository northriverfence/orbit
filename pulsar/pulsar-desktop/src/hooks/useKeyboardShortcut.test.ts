import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useKeyboardShortcut, SHORTCUTS } from './useKeyboardShortcut'

describe('useKeyboardShortcut', () => {
  let callback: ReturnType<typeof vi.fn>

  beforeEach(() => {
    callback = vi.fn()
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('should call callback when shortcut is pressed', () => {
    renderHook(() => useKeyboardShortcut({ key: 's', ctrlKey: true }, callback, true))

    const event = new KeyboardEvent('keydown', { key: 's', ctrlKey: true })
    window.dispatchEvent(event)

    expect(callback).toHaveBeenCalledTimes(1)
  })

  it('should not call callback when wrong key is pressed', () => {
    renderHook(() => useKeyboardShortcut({ key: 's', ctrlKey: true }, callback, true))

    const event = new KeyboardEvent('keydown', { key: 'a', ctrlKey: true })
    window.dispatchEvent(event)

    expect(callback).not.toHaveBeenCalled()
  })

  it('should not call callback when modifiers do not match', () => {
    renderHook(() => useKeyboardShortcut({ key: 's', ctrlKey: true }, callback, true))

    // Missing ctrlKey
    const event = new KeyboardEvent('keydown', { key: 's' })
    window.dispatchEvent(event)

    expect(callback).not.toHaveBeenCalled()
  })

  it('should not call callback when disabled', () => {
    renderHook(() => useKeyboardShortcut({ key: 's', ctrlKey: true }, callback, false))

    const event = new KeyboardEvent('keydown', { key: 's', ctrlKey: true })
    window.dispatchEvent(event)

    expect(callback).not.toHaveBeenCalled()
  })

  it('should call preventDefault when configured', () => {
    renderHook(() =>
      useKeyboardShortcut({ key: 's', ctrlKey: true, preventDefault: true }, callback, true)
    )

    const event = new KeyboardEvent('keydown', { key: 's', ctrlKey: true })
    const preventDefaultSpy = vi.spyOn(event, 'preventDefault')
    window.dispatchEvent(event)

    expect(preventDefaultSpy).toHaveBeenCalled()
    expect(callback).toHaveBeenCalled()
  })

  it('should handle multiple shortcuts', () => {
    const shortcuts = [
      { key: 's', ctrlKey: true },
      { key: 's', metaKey: true },
    ]

    renderHook(() => useKeyboardShortcut(shortcuts, callback, true))

    // Test Ctrl+S
    const ctrlEvent = new KeyboardEvent('keydown', { key: 's', ctrlKey: true })
    window.dispatchEvent(ctrlEvent)
    expect(callback).toHaveBeenCalledTimes(1)

    // Test Cmd+S (metaKey)
    const metaEvent = new KeyboardEvent('keydown', { key: 's', metaKey: true })
    window.dispatchEvent(metaEvent)
    expect(callback).toHaveBeenCalledTimes(2)
  })

  it('should cleanup event listener on unmount', () => {
    const { unmount } = renderHook(() =>
      useKeyboardShortcut({ key: 's', ctrlKey: true }, callback, true)
    )

    unmount()

    const event = new KeyboardEvent('keydown', { key: 's', ctrlKey: true })
    window.dispatchEvent(event)

    expect(callback).not.toHaveBeenCalled()
  })

  describe('SHORTCUTS constants', () => {
    it('should have ESCAPE shortcut', () => {
      expect(SHORTCUTS.ESCAPE).toEqual({ key: 'Escape' })
    })

    it('should have SAVE shortcut with cross-platform support', () => {
      expect(SHORTCUTS.SAVE).toEqual([
        { key: 's', ctrlKey: true, preventDefault: true },
        { key: 's', metaKey: true, preventDefault: true },
      ])
    })

    it('should have ENTER shortcut', () => {
      expect(SHORTCUTS.ENTER).toEqual({ key: 'Enter' })
    })
  })

  describe('case sensitivity', () => {
    it('should match keys case-insensitively', () => {
      renderHook(() => useKeyboardShortcut({ key: 's', ctrlKey: true }, callback, true))

      // Uppercase S
      const event = new KeyboardEvent('keydown', { key: 'S', ctrlKey: true })
      window.dispatchEvent(event)

      expect(callback).toHaveBeenCalled()
    })
  })

  describe('shift key handling', () => {
    it('should match shift key correctly', () => {
      renderHook(() =>
        useKeyboardShortcut({ key: 'Tab', shiftKey: true }, callback, true)
      )

      const event = new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true })
      window.dispatchEvent(event)

      expect(callback).toHaveBeenCalled()
    })

    it('should not match when shift is required but not pressed', () => {
      renderHook(() =>
        useKeyboardShortcut({ key: 'Tab', shiftKey: true }, callback, true)
      )

      const event = new KeyboardEvent('keydown', { key: 'Tab' })
      window.dispatchEvent(event)

      expect(callback).not.toHaveBeenCalled()
    })
  })
})
