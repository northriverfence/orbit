import { useState, useEffect } from 'react'
import Sidebar from './components/Sidebar'
import MainContentMultiSession from './components/MainContentMultiSession'
import FileTransferView from './components/FileTransferView'
import VaultView from './components/VaultView'
import SettingsDialog from './components/SettingsDialog'
import KeyboardShortcutsDialog from './components/KeyboardShortcutsDialog'
import ErrorBoundary from './components/ErrorBoundary'
import { ToastProvider } from './components/ToastContainer'
import CommandPalette from './components/CommandPalette'
import { useCommandPalette, createCommand } from './hooks/useCommandPalette'
import './App.css'
import './animations.css'

export type SidebarSection = 'workspaces' | 'servers' | 'file-transfer' | 'vaults' | 'settings' | null

function App() {
  const [expandedSection, setExpandedSection] = useState<SidebarSection>('workspaces')
  const [activeView, setActiveView] = useState<'terminals' | 'file-transfer' | 'vaults' | 'settings'>('terminals')
  const [isSettingsOpen, setIsSettingsOpen] = useState(false)
  const [isShortcutsOpen, setIsShortcutsOpen] = useState(false)

  // Command palette
  const commandPalette = useCommandPalette()

  // Register commands
  useEffect(() => {
    const commands = [
      // View commands
      createCommand('view.terminals', 'Open Terminal View', () => setActiveView('terminals'), {
        description: 'Switch to terminal workspace view',
        icon: '💻',
        keywords: ['terminal', 'workspace', 'sessions'],
        category: 'View',
      }),
      createCommand('view.file-transfer', 'Open File Transfer', () => setActiveView('file-transfer'), {
        description: 'Switch to file transfer view',
        icon: '📁',
        keywords: ['files', 'transfer', 'upload', 'download'],
        category: 'View',
      }),
      createCommand('view.vaults', 'Open Vaults', () => setActiveView('vaults'), {
        description: 'Switch to vault credentials view',
        icon: '🔐',
        keywords: ['vault', 'credentials', 'keys', 'passwords'],
        category: 'View',
      }),

      // Settings commands
      createCommand('settings.open', 'Open Settings', () => setIsSettingsOpen(true), {
        description: 'Configure application settings',
        icon: '⚙️',
        keywords: ['settings', 'preferences', 'config'],
        category: 'Settings',
      }),

      // Navigation commands
      createCommand('nav.workspaces', 'Focus Workspaces Section', () => setExpandedSection('workspaces'), {
        description: 'Navigate to workspaces in sidebar',
        icon: '🗂️',
        keywords: ['workspaces', 'sidebar'],
        category: 'Navigation',
      }),
      createCommand('nav.servers', 'Focus Servers Section', () => setExpandedSection('servers'), {
        description: 'Navigate to servers in sidebar',
        icon: '🖥️',
        keywords: ['servers', 'sidebar'],
        category: 'Navigation',
      }),

      // Help commands
      createCommand('help.shortcuts', 'Show Keyboard Shortcuts', () => setIsShortcutsOpen(true), {
        description: 'Display keyboard shortcuts reference',
        icon: '⌨️',
        keywords: ['help', 'shortcuts', 'keyboard', 'hotkeys'],
        category: 'Help',
      }),
    ]

    commandPalette.registerCommands(commands)

    return () => {
      commandPalette.clearCommands()
    }
  }, [setActiveView, setIsSettingsOpen, setExpandedSection, commandPalette])

  // Keyboard shortcut handler for settings (Ctrl+, or Cmd+,)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Check for Ctrl+, or Cmd+, (key is ',' with ctrlKey or metaKey)
      if ((e.ctrlKey || e.metaKey) && e.key === ',') {
        e.preventDefault()
        setIsSettingsOpen(true)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  // Keyboard shortcut handler for shortcuts dialog (?)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Check for ? key (Shift+/)
      if (e.key === '?' && !e.ctrlKey && !e.metaKey && !e.altKey) {
        // Only trigger if not in an input field
        const target = e.target as HTMLElement
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
          return
        }
        e.preventDefault()
        setIsShortcutsOpen(true)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  // Handle section toggle - switch views based on section
  const handleSectionToggle = (section: SidebarSection) => {
    setExpandedSection(section)

    // Switch main view based on section (or open settings dialog)
    if (section === 'file-transfer') {
      setActiveView('file-transfer')
    } else if (section === 'vaults') {
      setActiveView('vaults')
    } else if (section === 'settings') {
      // Open settings dialog instead of switching view
      setIsSettingsOpen(true)
    } else {
      setActiveView('terminals')
    }
  }

  return (
    <ErrorBoundary>
      <ToastProvider>
        <div className="flex h-screen w-screen bg-white">
          <Sidebar
            expandedSection={expandedSection}
            onSectionToggle={handleSectionToggle}
          />

          {/* Main content area - conditionally render based on active view */}
          <div className="flex-1 overflow-hidden">
            {activeView === 'terminals' && <MainContentMultiSession />}
            {activeView === 'file-transfer' && <FileTransferView />}
            {activeView === 'vaults' && <VaultView />}
          </div>

          {/* Settings Dialog - opens as modal overlay */}
          <SettingsDialog
            isOpen={isSettingsOpen}
            onClose={() => setIsSettingsOpen(false)}
          />

          {/* Command Palette - Ctrl/Cmd+K */}
          <CommandPalette
            commands={commandPalette.commands}
            isOpen={commandPalette.isOpen}
            onClose={commandPalette.close}
          />

          {/* Keyboard Shortcuts Dialog - ? */}
          <KeyboardShortcutsDialog
            isOpen={isShortcutsOpen}
            onClose={() => setIsShortcutsOpen(false)}
          />
        </div>
      </ToastProvider>
    </ErrorBoundary>
  )
}

export default App
