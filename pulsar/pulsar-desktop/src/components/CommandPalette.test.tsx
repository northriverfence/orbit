import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import CommandPalette from './CommandPalette'
import { Command } from '../hooks/useCommandPalette'

describe('CommandPalette', () => {
  const mockCommands: Command[] = [
    {
      id: 'view.terminals',
      label: 'Open Terminal View',
      description: 'Switch to terminal workspace',
      icon: '💻',
      keywords: ['terminal', 'workspace'],
      category: 'View',
      action: vi.fn(),
    },
    {
      id: 'view.vault',
      label: 'Open Vault',
      description: 'Manage credentials',
      icon: '🔐',
      keywords: ['vault', 'credentials', 'password'],
      category: 'View',
      action: vi.fn(),
    },
    {
      id: 'settings.open',
      label: 'Open Settings',
      description: 'Configure Pulsar',
      icon: '⚙️',
      keywords: ['settings', 'preferences', 'config'],
      category: 'Settings',
      action: vi.fn(),
    },
    {
      id: 'help.shortcuts',
      label: 'Show Keyboard Shortcuts',
      description: 'View all shortcuts',
      icon: '⌨️',
      keywords: ['help', 'keyboard', 'shortcuts'],
      category: 'Help',
      action: vi.fn(),
    },
  ]

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should not render when closed', () => {
    const { container } = render(
      <CommandPalette commands={mockCommands} isOpen={false} onClose={() => {}} />
    )

    expect(container.firstChild).toBeNull()
  })

  it('should render when open', () => {
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />)

    expect(screen.getByPlaceholderText(/type a command/i)).toBeInTheDocument()
  })

  it('should display all commands initially', () => {
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />)

    expect(screen.getByText('Open Terminal View')).toBeInTheDocument()
    expect(screen.getByText('Open Vault')).toBeInTheDocument()
    expect(screen.getByText('Open Settings')).toBeInTheDocument()
    expect(screen.getByText('Show Keyboard Shortcuts')).toBeInTheDocument()
  })

  it('should filter commands by search query', () => {
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />)

    const searchInput = screen.getByPlaceholderText(/type a command/i)
    fireEvent.change(searchInput, { target: { value: 'vault' } })

    // Should show vault command
    expect(screen.getByText('Open Vault')).toBeInTheDocument()

    // Should not show other commands
    expect(screen.queryByText('Open Terminal View')).not.toBeInTheDocument()
    expect(screen.queryByText('Open Settings')).not.toBeInTheDocument()
  })

  it('should filter commands by keywords', () => {
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />)

    const searchInput = screen.getByPlaceholderText(/type a command/i)
    fireEvent.change(searchInput, { target: { value: 'password' } })

    // 'password' is a keyword for vault
    expect(screen.getByText('Open Vault')).toBeInTheDocument()
    expect(screen.queryByText('Open Terminal View')).not.toBeInTheDocument()
  })

  it('should filter commands by description', () => {
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />)

    const searchInput = screen.getByPlaceholderText(/type a command/i)
    fireEvent.change(searchInput, { target: { value: 'workspace' } })

    // 'workspace' is in the description of terminal view
    expect(screen.getByText('Open Terminal View')).toBeInTheDocument()

    // Should not show other commands
    expect(screen.queryByText('Open Settings')).not.toBeInTheDocument()
    expect(screen.queryByText('Open Vault')).not.toBeInTheDocument()
  })

  it('should show no results message when no commands match', () => {
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />)

    const searchInput = screen.getByPlaceholderText(/type a command/i)
    fireEvent.change(searchInput, { target: { value: 'nonexistent' } })

    expect(screen.getByText(/no commands found/i)).toBeInTheDocument()
  })

  it('should execute command on click', () => {
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />)

    const terminalCommand = screen.getByText('Open Terminal View')
    fireEvent.click(terminalCommand)

    expect(mockCommands[0].action).toHaveBeenCalledTimes(1)
  })

  it('should close palette after executing command', () => {
    const onClose = vi.fn()
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={onClose} />)

    const terminalCommand = screen.getByText('Open Terminal View')
    fireEvent.click(terminalCommand)

    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('should close on Escape key', () => {
    const onClose = vi.fn()
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={onClose} />)

    const searchInput = screen.getByPlaceholderText(/type a command/i)
    fireEvent.keyDown(searchInput, { key: 'Escape' })

    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('should display command icons', () => {
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />)

    expect(screen.getByText('💻')).toBeInTheDocument()
    expect(screen.getByText('🔐')).toBeInTheDocument()
    expect(screen.getByText('⚙️')).toBeInTheDocument()
  })

  it('should display command descriptions', () => {
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />)

    expect(screen.getByText('Switch to terminal workspace')).toBeInTheDocument()
    expect(screen.getByText('Manage credentials')).toBeInTheDocument()
    expect(screen.getByText('Configure Pulsar')).toBeInTheDocument()
  })

  it('should group commands by category', () => {
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />)

    // Category headers should be visible
    expect(screen.getByText('View')).toBeInTheDocument()
    expect(screen.getByText('Settings')).toBeInTheDocument()
    expect(screen.getByText('Help')).toBeInTheDocument()
  })

  it('should handle empty commands array', () => {
    render(<CommandPalette commands={[]} isOpen={true} onClose={() => {}} />)

    expect(screen.getByText(/no commands found/i)).toBeInTheDocument()
  })

  it('should focus search input when opened', () => {
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />)

    const searchInput = screen.getByPlaceholderText(/type a command/i)
    expect(searchInput).toHaveFocus()
  })

  it('should clear search when closed and reopened', () => {
    const { rerender } = render(
      <CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />
    )

    const searchInput = screen.getByPlaceholderText(/type a command/i)
    fireEvent.change(searchInput, { target: { value: 'vault' } })

    // Close and reopen
    rerender(<CommandPalette commands={mockCommands} isOpen={false} onClose={() => {}} />)
    rerender(<CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />)

    const newSearchInput = screen.getByPlaceholderText(/type a command/i)
    expect(newSearchInput).toHaveValue('')
  })

  it('should be case-insensitive when filtering', () => {
    render(<CommandPalette commands={mockCommands} isOpen={true} onClose={() => {}} />)

    const searchInput = screen.getByPlaceholderText(/type a command/i)
    fireEvent.change(searchInput, { target: { value: 'VAULT' } })

    expect(screen.getByText('Open Vault')).toBeInTheDocument()
  })
})
