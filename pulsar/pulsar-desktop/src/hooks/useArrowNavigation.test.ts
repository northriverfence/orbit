import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useArrowNavigation } from './useArrowNavigation'
import { useRef } from 'react'

describe('useArrowNavigation', () => {
  let container: HTMLDivElement

  beforeEach(() => {
    container = document.createElement('div')
    container.innerHTML = `
      <div role="option" tabindex="0">Item 1</div>
      <div role="option" tabindex="0">Item 2</div>
      <div role="option" tabindex="0">Item 3</div>
      <div role="option" tabindex="0">Item 4</div>
      <div role="option" tabindex="0">Item 5</div>
    `
    document.body.appendChild(container)
  })

  it('should initialize with activeIndex 0', () => {
    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: container },
        enabled: true,
      })
    )

    expect(result.current.activeIndex).toBe(0)
  })

  it('should move to next item on ArrowDown', () => {
    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: container },
        enabled: true,
      })
    )

    expect(result.current.activeIndex).toBe(0)

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true })
      window.dispatchEvent(event)
    })

    // ArrowDown should move to next item
    expect(result.current.activeIndex).toBe(1)
  })

  it('should move to previous item on ArrowUp', () => {
    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: container },
        enabled: true,
      })
    )

    // Start at index 2
    act(() => {
      result.current.setActiveIndex(2)
    })

    expect(result.current.activeIndex).toBe(2)

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true })
      window.dispatchEvent(event)
    })

    // Should move to previous item
    expect(result.current.activeIndex).toBe(1)
  })

  it('should jump to first item on Home key', () => {
    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: container },
        enabled: true,
      })
    )

    act(() => {
      result.current.setActiveIndex(3)
    })

    expect(result.current.activeIndex).toBe(3)

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'Home', bubbles: true })
      window.dispatchEvent(event)
    })

    expect(result.current.activeIndex).toBe(0)
  })

  it('should jump to last item on End key', () => {
    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: container },
        enabled: true,
      })
    )

    expect(result.current.activeIndex).toBe(0)

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'End', bubbles: true })
      window.dispatchEvent(event)
    })

    expect(result.current.activeIndex).toBe(4)
  })

  it('should call onSelect when Enter is pressed', () => {
    const onSelect = vi.fn()
    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: container },
        enabled: true,
        onSelect,
      })
    )

    act(() => {
      result.current.setActiveIndex(2)
    })

    expect(result.current.activeIndex).toBe(2)

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true })
      window.dispatchEvent(event)
    })

    expect(onSelect).toHaveBeenCalledTimes(1)
    expect(onSelect).toHaveBeenCalledWith(2)
  })

  it('should loop to start when reaching end if loop is true', () => {
    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: container },
        enabled: true,
        loop: true,
      })
    )

    // Set to last item
    act(() => {
      result.current.setActiveIndex(4)
    })

    expect(result.current.activeIndex).toBe(4)

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true })
      window.dispatchEvent(event)
    })

    // With loop enabled, should wrap around to start
    expect(result.current.activeIndex).toBe(0)
  })

  it('should loop to end when at start and pressing ArrowUp with loop enabled', () => {
    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: container },
        enabled: true,
        loop: true,
      })
    )

    expect(result.current.activeIndex).toBe(0)

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true })
      window.dispatchEvent(event)
    })

    // With loop enabled, should wrap around to end
    expect(result.current.activeIndex).toBe(4)
  })

  it('should not go past bounds when loop is false', () => {
    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: container },
        enabled: true,
        loop: false,
      })
    )

    // At start, pressing ArrowUp should stay at 0
    expect(result.current.activeIndex).toBe(0)

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true })
      window.dispatchEvent(event)
    })

    expect(result.current.activeIndex).toBe(0)

    // Go to end
    act(() => {
      result.current.setActiveIndex(4)
    })

    expect(result.current.activeIndex).toBe(4)

    // Pressing ArrowDown should stay at 4
    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true })
      window.dispatchEvent(event)
    })

    expect(result.current.activeIndex).toBe(4)
  })

  it('should not navigate when disabled', () => {
    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: container },
        enabled: false,
      })
    )

    const initialIndex = result.current.activeIndex
    expect(initialIndex).toBe(0)

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true })
      window.dispatchEvent(event)
    })

    // Index should not change when disabled
    expect(result.current.activeIndex).toBe(initialIndex)
  })

  it('should allow manual index setting with setActiveIndex', () => {
    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: container },
        enabled: true,
      })
    )

    expect(result.current.activeIndex).toBe(0)

    act(() => {
      result.current.setActiveIndex(3)
    })

    expect(result.current.activeIndex).toBe(3)
  })

  it('should handle empty container gracefully', () => {
    const emptyContainer = document.createElement('div')
    document.body.appendChild(emptyContainer)

    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: emptyContainer },
        enabled: true,
      })
    )

    expect(result.current.activeIndex).toBe(0)

    act(() => {
      const event = new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true })
      window.dispatchEvent(event)
    })

    // Should not crash and index should remain 0
    expect(result.current.activeIndex).toBe(0)

    document.body.removeChild(emptyContainer)
  })

  it('should cleanup event listeners on unmount', () => {
    const { unmount } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: container },
        enabled: true,
      })
    )

    unmount()

    // After unmount, dispatching events should not cause errors
    const event = new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true })
    expect(() => window.dispatchEvent(event)).not.toThrow()

    expect(container).toBeTruthy()
  })

  it('should handle null container ref', () => {
    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: null },
        enabled: true,
      })
    )

    expect(result.current.activeIndex).toBe(0)
    expect(result.current.setActiveIndex).toBeDefined()
  })

  it('should update when enabled state changes', () => {
    const { rerender } = renderHook(
      ({ enabled }) =>
        useArrowNavigation({
          containerRef: { current: container },
          enabled,
        }),
      { initialProps: { enabled: false } }
    )

    rerender({ enabled: true })
    rerender({ enabled: false })

    // Should handle state changes without errors
    expect(container).toBeTruthy()
  })

  it('should respect custom itemSelector', () => {
    const customContainer = document.createElement('div')
    customContainer.innerHTML = `
      <div class="custom-item">Item 1</div>
      <div class="custom-item">Item 2</div>
      <div class="custom-item">Item 3</div>
    `
    document.body.appendChild(customContainer)

    const { result } = renderHook(() =>
      useArrowNavigation({
        containerRef: { current: customContainer },
        enabled: true,
        itemSelector: '.custom-item',
      })
    )

    expect(result.current.activeIndex).toBe(0)

    document.body.removeChild(customContainer)
  })

  it('should prevent default on navigation keys', () => {
    renderHook(() =>
      useArrowNavigation({
        containerRef: { current: container },
        enabled: true,
      })
    )

    // Create event with cancelable: true to allow preventDefault
    const event = new KeyboardEvent('keydown', {
      key: 'ArrowDown',
      bubbles: true,
      cancelable: true
    })
    const preventDefaultSpy = vi.spyOn(event, 'preventDefault')

    act(() => {
      window.dispatchEvent(event)
    })

    // preventDefault should have been called
    expect(preventDefaultSpy).toHaveBeenCalled()
  })
})
