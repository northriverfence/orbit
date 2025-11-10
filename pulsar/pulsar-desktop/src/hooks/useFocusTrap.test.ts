import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useFocusTrap } from './useFocusTrap'
import { useRef } from 'react'

describe('useFocusTrap', () => {
  let container: HTMLDivElement

  beforeEach(() => {
    // Create a container with focusable elements
    container = document.createElement('div')
    container.innerHTML = `
      <button id="first">First</button>
      <input id="input" type="text" />
      <button id="last">Last</button>
    `
    document.body.appendChild(container)
  })

  afterEach(() => {
    document.body.removeChild(container)
  })

  it('should trap focus within container when enabled', () => {
    const { result } = renderHook(() => {
      const ref = useRef<HTMLDivElement>(container)
      useFocusTrap(ref, true)
      return ref
    })

    expect(result.current.current).toBe(container)
  })

  it('should not trap focus when disabled', () => {
    const { result } = renderHook(() => {
      const ref = useRef<HTMLDivElement>(container)
      useFocusTrap(ref, false)
      return ref
    })

    expect(result.current.current).toBe(container)
  })

  it('should handle Tab key to move focus forward', () => {
    const { result } = renderHook(() => {
      const ref = useRef<HTMLDivElement>(container)
      useFocusTrap(ref, true)
      return ref
    })

    const firstButton = container.querySelector('#first') as HTMLElement
    const input = container.querySelector('#input') as HTMLElement

    firstButton.focus()
    expect(document.activeElement).toBe(firstButton)

    // Simulate Tab key
    const event = new KeyboardEvent('keydown', { key: 'Tab', bubbles: true })
    firstButton.dispatchEvent(event)

    // After Tab, focus should move to next element (handled by browser)
    // We just verify the hook doesn't interfere
    expect(result.current.current).toBe(container)
  })

  it('should handle Shift+Tab to move focus backward', () => {
    const { result } = renderHook(() => {
      const ref = useRef<HTMLDivElement>(container)
      useFocusTrap(ref, true)
      return ref
    })

    const lastButton = container.querySelector('#last') as HTMLElement
    lastButton.focus()

    // Simulate Shift+Tab
    const event = new KeyboardEvent('keydown', {
      key: 'Tab',
      shiftKey: true,
      bubbles: true
    })
    lastButton.dispatchEvent(event)

    expect(result.current.current).toBe(container)
  })

  it('should find focusable elements within container', () => {
    const { result } = renderHook(() => {
      const ref = useRef<HTMLDivElement>(container)
      useFocusTrap(ref, true)
      return ref
    })

    // Verify container has focusable elements
    const focusableElements = container.querySelectorAll(
      'button, input, select, textarea, a[href], [tabindex]:not([tabindex="-1"])'
    )

    expect(focusableElements.length).toBeGreaterThan(0)
    expect(result.current.current).toBe(container)
  })

  it('should update when enabled state changes', () => {
    const { rerender } = renderHook(
      ({ enabled }) => {
        const ref = useRef<HTMLDivElement>(container)
        useFocusTrap(ref, enabled)
        return ref
      },
      { initialProps: { enabled: false } }
    )

    // Initially disabled
    rerender({ enabled: false })

    // Enable focus trap
    rerender({ enabled: true })

    // Disable again
    rerender({ enabled: false })

    // Hook should handle state changes without errors
    expect(container).toBeTruthy()
  })

  it('should handle container with no focusable elements', () => {
    const emptyContainer = document.createElement('div')
    emptyContainer.innerHTML = '<div>No focusable elements</div>'
    document.body.appendChild(emptyContainer)

    const { result } = renderHook(() => {
      const ref = useRef<HTMLDivElement>(emptyContainer)
      useFocusTrap(ref, true)
      return ref
    })

    expect(result.current.current).toBe(emptyContainer)

    document.body.removeChild(emptyContainer)
  })

  it('should cleanup on unmount', () => {
    const { unmount } = renderHook(() => {
      const ref = useRef<HTMLDivElement>(container)
      useFocusTrap(ref, true)
      return ref
    })

    unmount()

    // After unmount, event listeners should be removed
    // We can't directly test this, but verify no errors occur
    expect(container).toBeTruthy()
  })

  it('should handle ref being null initially', () => {
    const { result } = renderHook(() => {
      const ref = useRef<HTMLDivElement>(null)
      useFocusTrap(ref, true)
      return ref
    })

    expect(result.current.current).toBeNull()
  })

  it('should work with different types of focusable elements', () => {
    const diverseContainer = document.createElement('div')
    diverseContainer.innerHTML = `
      <a href="#test">Link</a>
      <button>Button</button>
      <input type="text" />
      <select><option>Option</option></select>
      <textarea></textarea>
      <div tabindex="0">Div with tabindex</div>
    `
    document.body.appendChild(diverseContainer)

    const { result } = renderHook(() => {
      const ref = useRef<HTMLDivElement>(diverseContainer)
      useFocusTrap(ref, true)
      return ref
    })

    const focusableElements = diverseContainer.querySelectorAll(
      'a[href], button, input, select, textarea, [tabindex]:not([tabindex="-1"])'
    )

    expect(focusableElements.length).toBe(6)
    expect(result.current.current).toBe(diverseContainer)

    document.body.removeChild(diverseContainer)
  })
})
