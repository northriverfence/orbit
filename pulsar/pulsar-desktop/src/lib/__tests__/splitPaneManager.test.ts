/**
 * Split Pane Manager Tests
 *
 * Unit tests for the SplitPaneManager class
 */

import { SplitPaneManager } from '../splitPaneManager'
import type { SplitPaneLayout, SplitDirection } from '../../types/splitPane'

describe('SplitPaneManager', () => {
  let manager: SplitPaneManager

  beforeEach(() => {
    manager = new SplitPaneManager()
  })

  describe('Layout Creation', () => {
    test('should create a layout with a single pane', () => {
      const layout = manager.createLayout('test-layout', 'session-1')

      expect(layout).toBeDefined()
      expect(layout.id).toBe('test-layout')
      expect(layout.type).toBe('single')
      expect(layout.panes).toHaveLength(1)
      expect(layout.panes[0].sessionId).toBe('session-1')
      expect(layout.panes[0].size).toBe(100)
    })

    test('should set active pane to the first pane', () => {
      const layout = manager.createLayout('test-layout', 'session-1')

      expect(layout.activePane).toBe(layout.panes[0].id)
    })

    test('should create layout with null sessionId', () => {
      const layout = manager.createLayout('test-layout', null)

      expect(layout.panes[0].sessionId).toBeNull()
    })
  })

  describe('Layout Retrieval', () => {
    test('should retrieve a layout by ID', () => {
      const created = manager.createLayout('test-layout', 'session-1')
      const retrieved = manager.getLayout('test-layout')

      expect(retrieved).toEqual(created)
    })

    test('should return undefined for non-existent layout', () => {
      const retrieved = manager.getLayout('non-existent')

      expect(retrieved).toBeUndefined()
    })

    test('should retrieve active layout', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const active = manager.getActiveLayout()

      expect(active).toEqual(layout)
    })

    test('should return null when no active layout', () => {
      const active = manager.getActiveLayout()

      expect(active).toBeNull()
    })
  })

  describe('Pane Splitting', () => {
    test('should split a pane horizontally', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const originalPaneId = layout.panes[0].id

      const updatedLayout = manager.splitPane(
        'test-layout',
        originalPaneId,
        'horizontal',
        0.5
      )

      expect(updatedLayout).toBeDefined()
      expect(updatedLayout!.type).toBe('split')
      expect(updatedLayout!.panes[0].direction).toBe('horizontal')
      expect(updatedLayout!.panes[0].children).toHaveLength(2)
    })

    test('should split a pane vertically', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const originalPaneId = layout.panes[0].id

      const updatedLayout = manager.splitPane(
        'test-layout',
        originalPaneId,
        'vertical',
        0.5
      )

      expect(updatedLayout!.panes[0].direction).toBe('vertical')
    })

    test('should respect split ratio', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const originalPaneId = layout.panes[0].id

      const updatedLayout = manager.splitPane(
        'test-layout',
        originalPaneId,
        'horizontal',
        0.3
      )

      const children = updatedLayout!.panes[0].children!
      expect(children[0].size).toBe(30)
      expect(children[1].size).toBe(70)
    })

    test('should set active pane to new pane after split', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const originalPaneId = layout.panes[0].id

      const updatedLayout = manager.splitPane(
        'test-layout',
        originalPaneId,
        'horizontal',
        0.5
      )

      const newPaneId = updatedLayout!.panes[0].children![1].id
      expect(updatedLayout!.activePane).toBe(newPaneId)
    })

    test('should return null for non-existent layout', () => {
      const result = manager.splitPane('non-existent', 'pane-1', 'horizontal', 0.5)

      expect(result).toBeNull()
    })

    test('should return null for non-existent pane', () => {
      manager.createLayout('test-layout', 'session-1')
      const result = manager.splitPane('test-layout', 'non-existent', 'horizontal', 0.5)

      expect(result).toBeNull()
    })

    test('should support nested splits', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const rootPaneId = layout.panes[0].id

      // First split
      let updated = manager.splitPane('test-layout', rootPaneId, 'horizontal', 0.5)
      const firstChildId = updated!.panes[0].children![0].id

      // Second split on first child
      updated = manager.splitPane('test-layout', firstChildId, 'vertical', 0.5)

      expect(updated!.panes[0].children).toHaveLength(2)
      expect(updated!.panes[0].children![0].children).toHaveLength(2)
    })
  })

  describe('Pane Removal', () => {
    test('should remove a pane', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const rootPaneId = layout.panes[0].id

      // Split to create 2 panes
      let updated = manager.splitPane('test-layout', rootPaneId, 'horizontal', 0.5)
      const paneToRemove = updated!.panes[0].children![1].id

      // Remove the second pane
      updated = manager.removePane('test-layout', paneToRemove)

      expect(updated).toBeDefined()
      // After removing one pane from a 2-pane split, should collapse back to single
      expect(updated!.type).toBe('single')
    })

    test('should not remove the last pane', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const paneId = layout.panes[0].id

      const updated = manager.removePane('test-layout', paneId)

      // Should return layout unchanged
      expect(updated).toEqual(layout)
    })

    test('should update active pane when removing active pane', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const rootPaneId = layout.panes[0].id

      // Split to create 2 panes
      let updated = manager.splitPane('test-layout', rootPaneId, 'horizontal', 0.5)
      const activePaneId = updated!.activePane!

      // Remove the active pane
      updated = manager.removePane('test-layout', activePaneId)

      // Active pane should change
      expect(updated!.activePane).not.toBe(activePaneId)
      expect(updated!.activePane).toBeDefined()
    })

    test('should return null for non-existent layout', () => {
      const result = manager.removePane('non-existent', 'pane-1')

      expect(result).toBeNull()
    })

    test('should return layout unchanged when removing non-existent pane from single-pane layout', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const result = manager.removePane('test-layout', 'non-existent')

      // When trying to remove a non-existent pane from a single-pane layout,
      // the implementation returns the unchanged layout (defensive behavior)
      expect(result).not.toBeNull()
      expect(result!.id).toBe(layout.id)
      expect(result!.panes.length).toBe(1)
    })
  })

  describe('Active Pane Management', () => {
    test('should set active pane', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const rootPaneId = layout.panes[0].id

      // Split to create 2 panes
      const updated = manager.splitPane('test-layout', rootPaneId, 'horizontal', 0.5)
      const firstPaneId = updated!.panes[0].children![0].id

      // Set first pane as active
      const result = manager.setActivePane('test-layout', firstPaneId)

      expect(result!.activePane).toBe(firstPaneId)
    })

    test('should return null for non-existent layout', () => {
      const result = manager.setActivePane('non-existent', 'pane-1')

      expect(result).toBeNull()
    })
  })

  describe('Pane Resizing', () => {
    test('should resize a pane', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const paneId = layout.panes[0].id

      const updated = manager.resizePane('test-layout', paneId, 75)

      expect(updated).toBeDefined()
      expect(updated!.panes[0].size).toBe(75)
    })

    test('should enforce minimum size (10%)', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const paneId = layout.panes[0].id

      const updated = manager.resizePane('test-layout', paneId, 5)

      expect(updated!.panes[0].size).toBe(10)
    })

    test('should enforce maximum size (90%)', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const paneId = layout.panes[0].id

      const updated = manager.resizePane('test-layout', paneId, 95)

      expect(updated!.panes[0].size).toBe(90)
    })

    test('should return null for non-existent layout', () => {
      const result = manager.resizePane('non-existent', 'pane-1', 50)

      expect(result).toBeNull()
    })

    test('should return null for non-existent pane', () => {
      manager.createLayout('test-layout', 'session-1')
      const result = manager.resizePane('test-layout', 'non-existent', 50)

      expect(result).toBeNull()
    })
  })

  describe('Layout Serialization', () => {
    test('should serialize layout to JSON', () => {
      const layout = manager.createLayout('test-layout', 'session-1')

      const json = manager.serializeLayout('test-layout')

      expect(json).toBeDefined()
      expect(typeof json).toBe('string')

      const parsed = JSON.parse(json!)
      expect(parsed.id).toBe('test-layout')
      expect(parsed.type).toBe('single')
    })

    test('should return null for non-existent layout', () => {
      const json = manager.serializeLayout('non-existent')

      expect(json).toBeNull()
    })

    test('should deserialize layout from JSON', () => {
      const original = manager.createLayout('test-layout', 'session-1')
      const json = manager.serializeLayout('test-layout')

      const deserialized = manager.deserializeLayout('new-layout', json!)

      expect(deserialized).toBeDefined()
      expect(deserialized!.id).toBe('new-layout')
      expect(deserialized!.type).toBe(original.type)
      expect(deserialized!.panes).toHaveLength(1)
    })

    test('should handle invalid JSON gracefully', () => {
      const deserialized = manager.deserializeLayout('new-layout', 'invalid json')

      expect(deserialized).toBeNull()
    })

    test('should preserve complex layouts during serialization', () => {
      const layout = manager.createLayout('test-layout', 'session-1')
      const rootPaneId = layout.panes[0].id

      // Create complex layout
      let updated = manager.splitPane('test-layout', rootPaneId, 'horizontal', 0.5)
      const firstChildId = updated!.panes[0].children![0].id
      updated = manager.splitPane('test-layout', firstChildId, 'vertical', 0.3)

      // Serialize and deserialize
      const json = manager.serializeLayout('test-layout')
      const restored = manager.deserializeLayout('restored-layout', json!)

      expect(restored!.panes[0].children).toHaveLength(2)
      expect(restored!.panes[0].children![0].children).toHaveLength(2)
    })
  })

  describe('Edge Cases', () => {
    test('should generate unique pane IDs', () => {
      const layout1 = manager.createLayout('layout-1', 'session-1')
      const layout2 = manager.createLayout('layout-2', 'session-2')

      expect(layout1.panes[0].id).not.toBe(layout2.panes[0].id)
    })

    test('should handle multiple layouts independently', () => {
      const layout1 = manager.createLayout('layout-1', 'session-1')
      const layout2 = manager.createLayout('layout-2', 'session-2')

      manager.splitPane('layout-1', layout1.panes[0].id, 'horizontal', 0.5)

      const retrieved1 = manager.getLayout('layout-1')
      const retrieved2 = manager.getLayout('layout-2')

      expect(retrieved1!.type).toBe('split')
      expect(retrieved2!.type).toBe('single')
    })
  })
})
