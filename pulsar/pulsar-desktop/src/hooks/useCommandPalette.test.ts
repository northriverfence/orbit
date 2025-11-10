import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useCommandPalette, createCommand } from './useCommandPalette'

describe('useCommandPalette', () => {
  it('should initialize with closed state', () => {
    const { result } = renderHook(() => useCommandPalette())

    expect(result.current.isOpen).toBe(false)
    expect(result.current.commands).toEqual([])
  })

  it('should open command palette', () => {
    const { result } = renderHook(() => useCommandPalette())

    act(() => {
      result.current.open()
    })

    expect(result.current.isOpen).toBe(true)
  })

  it('should close command palette', () => {
    const { result } = renderHook(() => useCommandPalette())

    act(() => {
      result.current.open()
    })

    expect(result.current.isOpen).toBe(true)

    act(() => {
      result.current.close()
    })

    expect(result.current.isOpen).toBe(false)
  })

  it('should toggle command palette', () => {
    const { result } = renderHook(() => useCommandPalette())

    act(() => {
      result.current.toggle()
    })
    expect(result.current.isOpen).toBe(true)

    act(() => {
      result.current.toggle()
    })
    expect(result.current.isOpen).toBe(false)
  })

  it('should register a command', () => {
    const { result } = renderHook(() => useCommandPalette())
    const action = vi.fn()

    act(() => {
      result.current.registerCommand({
        id: 'test-command',
        label: 'Test Command',
        action,
      })
    })

    expect(result.current.commands).toHaveLength(1)
    expect(result.current.commands[0].id).toBe('test-command')
  })

  it('should replace command with same ID', () => {
    const { result } = renderHook(() => useCommandPalette())

    act(() => {
      result.current.registerCommand({
        id: 'test-command',
        label: 'First Label',
        action: () => {},
      })
    })

    act(() => {
      result.current.registerCommand({
        id: 'test-command',
        label: 'Second Label',
        action: () => {},
      })
    })

    expect(result.current.commands).toHaveLength(1)
    expect(result.current.commands[0].label).toBe('Second Label')
  })

  it('should unregister a command', () => {
    const { result } = renderHook(() => useCommandPalette())

    act(() => {
      result.current.registerCommand({
        id: 'test-command',
        label: 'Test Command',
        action: () => {},
      })
    })

    expect(result.current.commands).toHaveLength(1)

    act(() => {
      result.current.unregisterCommand('test-command')
    })

    expect(result.current.commands).toHaveLength(0)
  })

  it('should register multiple commands at once', () => {
    const { result } = renderHook(() => useCommandPalette())

    act(() => {
      result.current.registerCommands([
        { id: 'cmd1', label: 'Command 1', action: () => {} },
        { id: 'cmd2', label: 'Command 2', action: () => {} },
        { id: 'cmd3', label: 'Command 3', action: () => {} },
      ])
    })

    expect(result.current.commands).toHaveLength(3)
  })

  it('should clear all commands', () => {
    const { result } = renderHook(() => useCommandPalette())

    act(() => {
      result.current.registerCommands([
        { id: 'cmd1', label: 'Command 1', action: () => {} },
        { id: 'cmd2', label: 'Command 2', action: () => {} },
      ])
    })

    expect(result.current.commands).toHaveLength(2)

    act(() => {
      result.current.clearCommands()
    })

    expect(result.current.commands).toHaveLength(0)
  })

  it('should initialize with provided commands', () => {
    const initialCommands = [
      { id: 'cmd1', label: 'Command 1', action: () => {} },
      { id: 'cmd2', label: 'Command 2', action: () => {} },
    ]

    const { result } = renderHook(() =>
      useCommandPalette({ commands: initialCommands })
    )

    expect(result.current.commands).toHaveLength(2)
  })
})

describe('createCommand', () => {
  it('should create a basic command', () => {
    const action = vi.fn()
    const command = createCommand('test', 'Test Command', action)

    expect(command.id).toBe('test')
    expect(command.label).toBe('Test Command')
    expect(command.action).toBe(action)
    expect(command.category).toBe('Other')
    expect(command.keywords).toEqual([])
  })

  it('should create a command with options', () => {
    const action = vi.fn()
    const command = createCommand('test', 'Test Command', action, {
      description: 'Test description',
      icon: '🎯',
      keywords: ['test', 'command'],
      category: 'Test Category',
    })

    expect(command.description).toBe('Test description')
    expect(command.icon).toBe('🎯')
    expect(command.keywords).toEqual(['test', 'command'])
    expect(command.category).toBe('Test Category')
  })

  it('should execute command action', () => {
    const action = vi.fn()
    const command = createCommand('test', 'Test Command', action)

    command.action()

    expect(action).toHaveBeenCalledTimes(1)
  })
})
