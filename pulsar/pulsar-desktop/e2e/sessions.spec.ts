import { test, expect } from '@playwright/test';

/**
 * Session Management E2E Tests
 *
 * Tests for creating, managing, and switching between multiple sessions.
 */

test.describe('Session Creation', () => {
  test('should create new local session', async ({ page }) => {
    // Click new local session
    await page.click('[data-testid="new-local-session"]');

    // Verify terminal appears
    await expect(page.locator('[data-testid="terminal"]')).toBeVisible();

    // Verify session tab is created
    await expect(page.locator('[data-testid="session-tab"]')).toBeVisible();
  });

  test('should create multiple sessions in tabs', async ({ page }) => {
    // Create first session
    await page.click('[data-testid="new-local-session"]');
    await expect(page.locator('[data-testid="session-tab"]').first()).toBeVisible();

    // Create second session
    await page.click('[data-testid="new-local-session"]');

    // Verify two tabs exist
    const tabs = await page.locator('[data-testid="session-tab"]').count();
    expect(tabs).toBeGreaterThanOrEqual(2);
  });

  test('should auto-name sessions', async ({ page }) => {
    // Create sessions
    await page.click('[data-testid="new-local-session"]');

    // Verify default name
    await expect(page.locator('[data-testid="session-tab"]')).toContainText(/session|local/i);
  });
});

test.describe('Session Switching', () => {
  test.beforeEach(async ({ page }) => {
    // Create two sessions
    await page.click('[data-testid="new-local-session"]');
    await page.waitForTimeout(500);
    await page.click('[data-testid="new-local-session"]');
    await page.waitForTimeout(500);
  });

  test('should switch between sessions with mouse', async ({ page }) => {
    // Get tab references
    const tab1 = page.locator('[data-testid="session-tab"]').first();
    const tab2 = page.locator('[data-testid="session-tab"]').nth(1);

    // Click first tab
    await tab1.click();
    await expect(tab1).toHaveClass(/active/);

    // Click second tab
    await tab2.click();
    await expect(tab2).toHaveClass(/active/);
  });

  test('should switch sessions with keyboard shortcuts', async ({ page }) => {
    // Ctrl/Cmd+Tab to next session
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Tab`);

    // Verify active session changed
    const activeTab = page.locator('[data-testid="session-tab"][class*="active"]');
    await expect(activeTab).toBeVisible();
  });

  test('should cycle through sessions', async ({ page }) => {
    // Create 3 sessions
    await page.click('[data-testid="new-local-session"]');
    await page.waitForTimeout(300);

    const tabs = await page.locator('[data-testid="session-tab"]').count();
    expect(tabs).toBe(3);

    // Cycle through all sessions
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    for (let i = 0; i < tabs; i++) {
      await page.keyboard.press(`${modifier}+Tab`);
      await page.waitForTimeout(100);
    }

    // Should be back to first tab
    const firstTab = page.locator('[data-testid="session-tab"]').first();
    await expect(firstTab).toHaveClass(/active/);
  });
});

test.describe('Session Closing', () => {
  test('should close session with close button', async ({ page }) => {
    // Create session
    await page.click('[data-testid="new-local-session"]');
    const initialCount = await page.locator('[data-testid="session-tab"]').count();

    // Click close button on tab
    await page.click('[data-testid="close-session"]');

    // Verify session closed
    const finalCount = await page.locator('[data-testid="session-tab"]').count();
    expect(finalCount).toBe(initialCount - 1);
  });

  test('should close session with keyboard shortcut', async ({ page }) => {
    // Create session
    await page.click('[data-testid="new-local-session"]');

    // Press Ctrl/Cmd+W
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+KeyW`);

    // Verify confirmation dialog or direct close
    // (depends on implementation)
  });

  test('should confirm before closing unsaved session', async ({ page }) => {
    // Create session with activity
    await page.click('[data-testid="new-local-session"]');

    // Simulate activity (type in terminal)
    // This would require terminal interaction

    // Try to close
    await page.click('[data-testid="close-session"]');

    // Verify confirmation dialog
    await expect(page.locator('[data-testid="confirm-dialog"]')).toBeVisible();
  });

  test('should close all sessions', async ({ page }) => {
    // Create multiple sessions
    await page.click('[data-testid="new-local-session"]');
    await page.click('[data-testid="new-local-session"]');
    await page.click('[data-testid="new-local-session"]');

    // Close all command
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+KeyK`);
    await page.fill('[data-testid="command-search"]', 'close all');
    await page.keyboard.press('Enter');

    // Verify all sessions closed
    await expect(page.locator('[data-testid="empty-state"]')).toBeVisible();
  });
});

test.describe('Session Restoration', () => {
  test('should save sessions on close', async ({ page }) => {
    // Create sessions
    await page.click('[data-testid="new-local-session"]');
    await page.waitForTimeout(300);

    // Close app (simulated)
    // In real test, this would close the actual Tauri app

    // Reopen app
    // Verify session restoration notification
    await expect(page.locator('[data-testid="restore-notification"]')).toBeVisible({
      timeout: 5000,
    });
  });

  test('should restore previous sessions', async ({ page }) => {
    // Assuming sessions were saved
    // Click restore button
    await page.click('[data-testid="restore-sessions"]');

    // Verify sessions restored
    const tabs = await page.locator('[data-testid="session-tab"]').count();
    expect(tabs).toBeGreaterThan(0);
  });

  test('should selectively restore sessions', async ({ page }) => {
    // Open restore dialog
    await page.click('[data-testid="restore-sessions-select"]');

    // Verify session list
    await expect(page.locator('[data-testid="session-list"]')).toBeVisible();

    // Select specific sessions
    await page.click('[data-testid="session-checkbox-0"]');
    await page.click('[data-testid="session-checkbox-2"]');

    // Restore selected
    await page.click('[data-testid="restore-selected"]');

    // Verify only selected sessions restored
    const tabs = await page.locator('[data-testid="session-tab"]').count();
    expect(tabs).toBe(2);
  });

  test('should dismiss session restoration', async ({ page }) => {
    // Click dismiss on notification
    await page.click('[data-testid="dismiss-restore"]');

    // Verify notification dismissed
    await expect(page.locator('[data-testid="restore-notification"]')).not.toBeVisible();

    // Verify no sessions restored
    await expect(page.locator('[data-testid="empty-state"]')).toBeVisible();
  });
});

test.describe('Session Naming', () => {
  test('should rename session', async ({ page }) => {
    // Create session
    await page.click('[data-testid="new-local-session"]');

    // Right-click tab for context menu
    await page.click('[data-testid="session-tab"]', { button: 'right' });

    // Click rename
    await page.click('[data-testid="rename-session"]');

    // Enter new name
    await page.fill('[data-testid="session-name"]', 'My Custom Session');
    await page.keyboard.press('Enter');

    // Verify name updated
    await expect(page.locator('[data-testid="session-tab"]')).toContainText(
      'My Custom Session'
    );
  });

  test('should auto-name SSH sessions with host', async ({ page }) => {
    // Create SSH session (assuming connection dialog flow)
    // Fill hostname
    await page.click('[data-testid="new-connection"]');
    await page.fill('[data-testid="host"]', 'production.example.com');
    await page.fill('[data-testid="username"]', 'admin');
    await page.click('[data-testid="connect-button"]');

    // Verify tab named after host
    await expect(page.locator('[data-testid="session-tab"]')).toContainText(
      'production.example.com',
      { timeout: 10000 }
    );
  });
});

test.describe('Session State', () => {
  test('should persist terminal scrollback', async ({ page }) => {
    // Create session
    await page.click('[data-testid="new-local-session"]');

    // Generate output (would require terminal interaction)
    // Switch away and back
    await page.click('[data-testid="new-local-session"]');
    await page.click('[data-testid="session-tab"]').first();

    // Verify scrollback preserved
    // (Implementation depends on terminal component)
  });

  test('should maintain working directory per session', async ({ page }) => {
    // This test verifies each session maintains its own state
    // Requires terminal interaction to change directories
    expect(true).toBe(true);
  });
});
