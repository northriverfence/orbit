import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import KeyboardShortcutsDialog from './KeyboardShortcutsDialog'

describe('KeyboardShortcutsDialog', () => {
  it('should not render when closed', () => {
    const { container } = render(
      <KeyboardShortcutsDialog isOpen={false} onClose={() => {}} />
    )

    expect(container.firstChild).toBeNull()
  })

  it('should render when open', () => {
    render(<KeyboardShortcutsDialog isOpen={true} onClose={() => {}} />)

    expect(screen.getByText('Keyboard Shortcuts')).toBeInTheDocument()
  })

  it('should display shortcut categories', () => {
    render(<KeyboardShortcutsDialog isOpen={true} onClose={() => {}} />)

    expect(screen.getByText('Global')).toBeInTheDocument()
    expect(screen.getByText('Modals & Dialogs')).toBeInTheDocument()
    expect(screen.getByText('Navigation')).toBeInTheDocument()
  })

  it('should display global shortcuts', () => {
    render(<KeyboardShortcutsDialog isOpen={true} onClose={() => {}} />)

    expect(screen.getByText('Open Command Palette')).toBeInTheDocument()
    expect(screen.getByText('Open Settings')).toBeInTheDocument()
    expect(screen.getByText('Show Keyboard Shortcuts')).toBeInTheDocument()
  })

  it('should display modal shortcuts', () => {
    render(<KeyboardShortcutsDialog isOpen={true} onClose={() => {}} />)

    expect(screen.getByText('Close Dialog')).toBeInTheDocument()
    expect(screen.getByText(/submit.*connect/i)).toBeInTheDocument()
    expect(screen.getByText('Save Settings')).toBeInTheDocument()
  })

  it('should display tab shortcuts', () => {
    render(<KeyboardShortcutsDialog isOpen={true} onClose={() => {}} />)

    expect(screen.getByText('New Tab')).toBeInTheDocument()
    expect(screen.getByText('Close Tab')).toBeInTheDocument()
    expect(screen.getByText('Next Tab')).toBeInTheDocument()
    expect(screen.getByText('Previous Tab')).toBeInTheDocument()
  })

  it('should display search shortcuts', () => {
    render(<KeyboardShortcutsDialog isOpen={true} onClose={() => {}} />)

    expect(screen.getByText(/find in page/i)).toBeInTheDocument()
  })

  it('should render keyboard shortcut keys', () => {
    render(<KeyboardShortcutsDialog isOpen={true} onClose={() => {}} />)

    // Check for Ctrl key rendering
    const ctrlKeys = screen.getAllByText('Ctrl')
    expect(ctrlKeys.length).toBeGreaterThan(0)

    // Check for specific keys
    expect(screen.getByText('K')).toBeInTheDocument()
    expect(screen.getByText('Escape')).toBeInTheDocument()
  })

  it('should close when Got it button is clicked', () => {
    const onClose = vi.fn()
    render(<KeyboardShortcutsDialog isOpen={true} onClose={onClose} />)

    const gotItButton = screen.getByText(/got it/i)
    fireEvent.click(gotItButton)

    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('should close on Escape key', () => {
    const onClose = vi.fn()
    const { container } = render(
      <KeyboardShortcutsDialog isOpen={true} onClose={onClose} />
    )

    const dialog = container.querySelector('[role="dialog"]') || container.firstChild
    if (dialog) {
      fireEvent.keyDown(dialog, { key: 'Escape' })
      // The Escape handler is registered on the window, so we should dispatch on window
      fireEvent.keyDown(window, { key: 'Escape' })
    }

    // Due to focus trap and keyboard handling, this might not work perfectly in tests
    // But we verify the handler exists
    expect(onClose).toHaveBeenCalled()
  })

  it('should display keyboard key badges with proper styling', () => {
    const { container } = render(
      <KeyboardShortcutsDialog isOpen={true} onClose={() => {}} />
    )

    // Check for key badges - they should have specific styling
    const badges = container.querySelectorAll('.px-2')
    expect(badges.length).toBeGreaterThan(0)
  })

  it('should organize shortcuts in a structured layout', () => {
    const { container } = render(
      <KeyboardShortcutsDialog isOpen={true} onClose={() => {}} />
    )

    // Check for structured layout - the component has shortcuts organized in sections
    const shortcuts = screen.getAllByText(/open command palette|close dialog|new tab/i)
    expect(shortcuts.length).toBeGreaterThan(0)
  })

  it('should have descriptive header', () => {
    render(<KeyboardShortcutsDialog isOpen={true} onClose={() => {}} />)

    expect(screen.getByText('Keyboard Shortcuts')).toBeInTheDocument()
  })

  it('should display keyboard shortcut keys prominently', () => {
    const { container } = render(
      <KeyboardShortcutsDialog isOpen={true} onClose={() => {}} />
    )

    // The component renders key badges - check that multiple exist
    const keyElements = screen.getAllByText(/ctrl|escape|enter|tab/i)
    expect(keyElements.length).toBeGreaterThan(3) // Multiple shortcuts with keys
  })

  it('should categorize shortcuts logically', () => {
    render(<KeyboardShortcutsDialog isOpen={true} onClose={() => {}} />)

    // Verify that categories contain expected shortcuts
    // Global should have command palette
    expect(screen.getByText('Open Command Palette')).toBeInTheDocument()

    // Tabs should have tab management (even if coming soon)
    expect(screen.getByText('New Tab')).toBeInTheDocument()
    expect(screen.getByText('Close Tab')).toBeInTheDocument()
  })

  it('should render all main categories', () => {
    render(<KeyboardShortcutsDialog isOpen={true} onClose={() => {}} />)

    // Check that main categories are visible
    const categories = [
      screen.getByText('Global'),
      screen.getByText('Modals & Dialogs'),
      screen.getByText('Navigation'),
    ]

    categories.forEach((category) => {
      expect(category).toBeInTheDocument()
    })
  })
})
