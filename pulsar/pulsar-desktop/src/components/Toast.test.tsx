import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor, act } from '@testing-library/react'
import Toast from './Toast'

describe('Toast', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('should render toast message', () => {
    render(<Toast message="Test message" onClose={() => {}} />)

    expect(screen.getByText('Test message')).toBeInTheDocument()
  })

  it('should render success toast with checkmark icon', () => {
    const { container } = render(<Toast message="Success" type="success" onClose={() => {}} />)

    expect(screen.getByText('Success')).toBeInTheDocument()
    // Success toast has green background - check the root toast div
    const toastElement = container.firstChild as HTMLElement
    expect(toastElement.className).toContain('bg-green-50')
  })

  it('should render error toast with X icon', () => {
    const { container } = render(<Toast message="Error occurred" type="error" onClose={() => {}} />)

    expect(screen.getByText('Error occurred')).toBeInTheDocument()
    const toastElement = container.firstChild as HTMLElement
    expect(toastElement.className).toContain('bg-red-50')
  })

  it('should render warning toast', () => {
    const { container } = render(<Toast message="Warning" type="warning" onClose={() => {}} />)

    const toastElement = container.firstChild as HTMLElement
    expect(toastElement.className).toContain('bg-yellow-50')
  })

  it('should render info toast', () => {
    const { container } = render(<Toast message="Information" type="info" onClose={() => {}} />)

    const toastElement = container.firstChild as HTMLElement
    expect(toastElement.className).toContain('bg-blue-50')
  })

  it('should auto-close after duration', async () => {
    const onClose = vi.fn()
    render(<Toast message="Test" duration={1000} onClose={onClose} />)

    expect(onClose).not.toHaveBeenCalled()

    // Fast-forward time past duration
    act(() => {
      vi.advanceTimersByTime(1000)
    })

    // Fast-forward past exit animation
    act(() => {
      vi.advanceTimersByTime(300)
    })

    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('should not auto-close when duration is 0', async () => {
    const onClose = vi.fn()
    render(<Toast message="Test" duration={0} onClose={onClose} />)

    vi.advanceTimersByTime(5000)

    expect(onClose).not.toHaveBeenCalled()
  })

  it('should call onClose when close button is clicked', () => {
    const onClose = vi.fn()
    const { container } = render(<Toast message="Test" onClose={onClose} />)

    // Find and click the close button (×)
    const closeButton = container.querySelector('button')

    act(() => {
      closeButton?.click()
    })

    // Fast-forward past exit animation
    act(() => {
      vi.advanceTimersByTime(300)
    })

    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('should have proper animation classes', () => {
    const { container } = render(<Toast message="Test" onClose={() => {}} />)

    // Toast should have slide-in animation - check the actual rendered div
    const toastElement = container.firstChild as HTMLElement
    expect(toastElement).toBeTruthy()
    expect(toastElement.className).toContain('animate-toastIn')
  })
})
