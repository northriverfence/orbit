import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import SessionRestoreNotification from './SessionRestoreNotification'

describe('SessionRestoreNotification', () => {
  it('should render with session count', () => {
    render(
      <SessionRestoreNotification
        sessionCount={3}
        onRestoreAll={() => {}}
        onDismiss={() => {}}
        onManage={() => {}}
      />
    )

    expect(screen.getByText(/found 3 previous sessions/i)).toBeInTheDocument()
  })

  it('should display singular form for 1 session', () => {
    render(
      <SessionRestoreNotification
        sessionCount={1}
        onRestoreAll={() => {}}
        onDismiss={() => {}}
        onManage={() => {}}
      />
    )

    expect(screen.getByText(/found 1 previous session/i)).toBeInTheDocument()
  })

  it('should display plural form for multiple sessions', () => {
    render(
      <SessionRestoreNotification
        sessionCount={5}
        onRestoreAll={() => {}}
        onDismiss={() => {}}
        onManage={() => {}}
      />
    )

    expect(screen.getByText(/found 5 previous sessions/i)).toBeInTheDocument()
  })

  it('should call onRestoreAll when Restore All button is clicked', () => {
    const onRestoreAll = vi.fn()
    render(
      <SessionRestoreNotification
        sessionCount={3}
        onRestoreAll={onRestoreAll}
        onDismiss={() => {}}
        onManage={() => {}}
      />
    )

    const restoreButton = screen.getByText('Restore All')
    fireEvent.click(restoreButton)

    expect(onRestoreAll).toHaveBeenCalledTimes(1)
  })

  it('should call onManage when Choose Sessions button is clicked', () => {
    const onManage = vi.fn()
    render(
      <SessionRestoreNotification
        sessionCount={3}
        onRestoreAll={() => {}}
        onDismiss={() => {}}
        onManage={onManage}
      />
    )

    const chooseButton = screen.getByText('Choose Sessions')
    fireEvent.click(chooseButton)

    expect(onManage).toHaveBeenCalledTimes(1)
  })

  it('should call onDismiss when Dismiss button is clicked', () => {
    const onDismiss = vi.fn()
    render(
      <SessionRestoreNotification
        sessionCount={3}
        onRestoreAll={() => {}}
        onDismiss={onDismiss}
        onManage={() => {}}
      />
    )

    const dismissButton = screen.getByText(/dismiss.*start fresh/i)
    fireEvent.click(dismissButton)

    expect(onDismiss).toHaveBeenCalledTimes(1)
  })

  it('should display Dismiss button with descriptive text', () => {
    render(
      <SessionRestoreNotification
        sessionCount={3}
        onRestoreAll={() => {}}
        onDismiss={() => {}}
        onManage={() => {}}
      />
    )

    // Dismiss button should have descriptive text
    expect(screen.getByText(/dismiss.*start fresh/i)).toBeInTheDocument()
  })

  it('should display all three action buttons', () => {
    render(
      <SessionRestoreNotification
        sessionCount={3}
        onRestoreAll={() => {}}
        onDismiss={() => {}}
        onManage={() => {}}
      />
    )

    expect(screen.getByText('Restore All')).toBeInTheDocument()
    expect(screen.getByText('Choose Sessions')).toBeInTheDocument()
    expect(screen.getByText(/dismiss.*start fresh/i)).toBeInTheDocument()
  })

  it('should have slide-in animation class', () => {
    const { container } = render(
      <SessionRestoreNotification
        sessionCount={3}
        onRestoreAll={() => {}}
        onDismiss={() => {}}
        onManage={() => {}}
      />
    )

    const notification = container.firstChild as HTMLElement
    expect(notification.className).toContain('animate-slideInFromRight')
  })

  it('should display info icon', () => {
    const { container } = render(
      <SessionRestoreNotification
        sessionCount={3}
        onRestoreAll={() => {}}
        onDismiss={() => {}}
        onManage={() => {}}
      />
    )

    // Check for SVG icon
    const svgs = container.querySelectorAll('svg')
    expect(svgs.length).toBeGreaterThan(0)
  })

  it('should have proper styling for notification card', () => {
    const { container } = render(
      <SessionRestoreNotification
        sessionCount={3}
        onRestoreAll={() => {}}
        onDismiss={() => {}}
        onManage={() => {}}
      />
    )

    const notification = container.firstChild as HTMLElement
    // Top-right positioning
    expect(notification.className).toContain('top-4')
    expect(notification.className).toContain('right-4')
  })

  it('should have gradient header for visual appeal', () => {
    const { container } = render(
      <SessionRestoreNotification
        sessionCount={3}
        onRestoreAll={() => {}}
        onDismiss={() => {}}
        onManage={() => {}}
      />
    )

    const header = container.querySelector('.bg-gradient-to-r')
    expect(header).toBeInTheDocument()
  })

  it('should handle zero sessions gracefully', () => {
    render(
      <SessionRestoreNotification
        sessionCount={0}
        onRestoreAll={() => {}}
        onDismiss={() => {}}
        onManage={() => {}}
      />
    )

    // Should still render with 0 sessions (edge case)
    expect(screen.getByText(/found 0 previous sessions/i)).toBeInTheDocument()
  })

  it('should have primary action button stand out', () => {
    const { container } = render(
      <SessionRestoreNotification
        sessionCount={3}
        onRestoreAll={() => {}}
        onDismiss={() => {}}
        onManage={() => {}}
      />
    )

    const restoreButton = screen.getByText('Restore All')
    expect(restoreButton.className).toContain('bg-blue-600')
    expect(restoreButton.className).toContain('text-white')
  })
})
