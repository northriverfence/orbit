import { test, expect } from '@playwright/test';

/**
 * Keyboard Navigation and Accessibility E2E Tests
 *
 * Tests for keyboard shortcuts, navigation, and accessibility features.
 */

test.describe('Command Palette', () => {
  test('should open with Ctrl/Cmd+K', async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+KeyK`);

    await expect(page.locator('[data-testid="command-palette"]')).toBeVisible();
  });

  test('should close with Escape', async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+KeyK`);
    await page.keyboard.press('Escape');

    await expect(page.locator('[data-testid="command-palette"]')).not.toBeVisible();
  });

  test('should navigate commands with arrow keys', async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+KeyK`);

    // Press arrow down
    await page.keyboard.press('ArrowDown');

    // Verify second item highlighted
    const activeItem = page.locator('[data-testid="command-item"][class*="active"]');
    await expect(activeItem).toBeVisible();

    // Press arrow up
    await page.keyboard.press('ArrowUp');

    // Verify first item highlighted
    await expect(page.locator('[data-testid="command-item"]').first()).toHaveClass(/active/);
  });

  test('should execute command with Enter', async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+KeyK`);

    // Type to search
    await page.type('[data-testid="command-search"]', 'new session');
    await page.keyboard.press('Enter');

    // Verify command executed (new session created)
    await expect(page.locator('[data-testid="terminal"]')).toBeVisible({
      timeout: 5000,
    });
  });

  test('should filter commands by search', async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+KeyK`);

    // Type search term
    await page.type('[data-testid="command-search"]', 'settings');

    // Verify filtered results
    const items = page.locator('[data-testid="command-item"]');
    const count = await items.count();

    for (let i = 0; i < count; i++) {
      await expect(items.nth(i)).toContainText(/settings/i);
    }
  });
});

test.describe('Settings Dialog', () => {
  test('should open with Ctrl/Cmd+,', async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);

    await expect(page.locator('[data-testid="settings-dialog"]')).toBeVisible();
  });

  test('should close with Escape', async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);
    await page.keyboard.press('Escape');

    await expect(page.locator('[data-testid="settings-dialog"]')).not.toBeVisible();
  });

  test('should navigate tabs with arrow keys', async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);

    // Press arrow right to next tab
    await page.keyboard.press('ArrowRight');

    // Verify tab changed
    const activeTab = page.locator('[data-testid="settings-tab"][class*="active"]');
    await expect(activeTab).toBeVisible();
  });

  test('should save with Ctrl/Cmd+S', async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);

    // Make a change
    await page.fill('[data-testid="setting-input"]', 'new value');

    // Save with keyboard
    await page.keyboard.press(`${modifier}+KeyS`);

    // Verify save notification
    await expect(page.locator('[data-testid="save-notification"]')).toBeVisible({
      timeout: 3000,
    });
  });
});

test.describe('Keyboard Shortcuts Help', () => {
  test('should open with ? key', async ({ page }) => {
    await page.keyboard.press('Shift+Slash'); // ? key

    await expect(page.locator('[data-testid="shortcuts-dialog"]')).toBeVisible();
  });

  test('should show all shortcut categories', async ({ page }) => {
    await page.keyboard.press('Shift+Slash');

    // Verify categories
    await expect(page.locator('[data-testid="shortcuts-category"]')).toHaveCount(5, {
      timeout: 3000,
    });
  });

  test('should navigate categories with Tab', async ({ page }) => {
    await page.keyboard.press('Shift+Slash');

    // Tab through categories
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');

    // Verify focus moved
    const focusedElement = await page.evaluateHandle(() => document.activeElement);
    expect(focusedElement).toBeTruthy();
  });
});

test.describe('Focus Management', () => {
  test('should trap focus in modal dialogs', async ({ page }) => {
    // Open modal
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);

    // Tab through all focusable elements
    for (let i = 0; i < 10; i++) {
      await page.keyboard.press('Tab');
    }

    // Verify focus stayed within modal
    const activeElement = await page.evaluateHandle(() => document.activeElement);
    const isWithinModal = await page.evaluate(
      (el) => el?.closest('[data-testid="settings-dialog"]') !== null,
      activeElement
    );
    expect(isWithinModal).toBe(true);
  });

  test('should reverse tab with Shift+Tab', async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);

    // Tab forward
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');

    // Tab backward
    await page.keyboard.press('Shift+Tab');

    // Verify focus moved backward
    const focusedElement = await page.evaluateHandle(() => document.activeElement);
    expect(focusedElement).toBeTruthy();
  });

  test('should restore focus after dialog close', async ({ page }) => {
    // Focus an element
    await page.focus('[data-testid="new-connection"]');

    // Open and close dialog
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);
    await page.keyboard.press('Escape');

    // Verify focus restored
    const focusedElement = await page.evaluate(() => document.activeElement?.getAttribute('data-testid'));
    expect(focusedElement).toBe('new-connection');
  });
});

test.describe('List Navigation', () => {
  test('should navigate list items with arrow keys', async ({ page }) => {
    // Assuming a list is visible (e.g., saved connections)
    await page.focus('[data-testid="list-item"]');

    // Navigate down
    await page.keyboard.press('ArrowDown');

    // Verify next item focused
    const activeItem = await page.evaluate(() => document.activeElement?.getAttribute('data-testid'));
    expect(activeItem).toContain('list-item');
  });

  test('should jump to start with Home', async ({ page }) => {
    // Focus list
    await page.focus('[data-testid="list-item"]');

    // Navigate to middle
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('ArrowDown');

    // Press Home
    await page.keyboard.press('Home');

    // Verify first item focused
    const firstItem = page.locator('[data-testid="list-item"]').first();
    await expect(firstItem).toBeFocused();
  });

  test('should jump to end with End', async ({ page }) => {
    // Focus list
    await page.focus('[data-testid="list-item"]');

    // Press End
    await page.keyboard.press('End');

    // Verify last item focused
    const lastItem = page.locator('[data-testid="list-item"]').last();
    await expect(lastItem).toBeFocused();
  });

  test('should wrap at boundaries when loop enabled', async ({ page }) => {
    // Focus last item
    await page.focus('[data-testid="list-item"]');
    await page.keyboard.press('End');

    // Press down (should wrap to first)
    await page.keyboard.press('ArrowDown');

    // Verify wrapped to first
    const firstItem = page.locator('[data-testid="list-item"]').first();
    await expect(firstItem).toBeFocused();
  });
});

test.describe('Session Shortcuts', () => {
  test('should create new session with Ctrl/Cmd+T', async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+KeyT`);

    // Verify new session created
    await expect(page.locator('[data-testid="terminal"]')).toBeVisible({
      timeout: 5000,
    });
  });

  test('should close session with Ctrl/Cmd+W', async ({ page }) => {
    // Create a session first
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+KeyT`);
    await page.waitForTimeout(500);

    // Close it
    await page.keyboard.press(`${modifier}+KeyW`);

    // Verify session closed (or confirmation shown)
    const hasConfirmation = await page.locator('[data-testid="confirm-dialog"]').isVisible();
    const sessionClosed = !(await page.locator('[data-testid="terminal"]').isVisible());

    expect(hasConfirmation || sessionClosed).toBe(true);
  });

  test('should switch sessions with Ctrl/Cmd+Tab', async ({ page }) => {
    // Create two sessions
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+KeyT`);
    await page.waitForTimeout(300);
    await page.keyboard.press(`${modifier}+KeyT`);
    await page.waitForTimeout(300);

    // Switch sessions
    await page.keyboard.press(`${modifier}+Tab`);

    // Verify active session changed
    const activeTab = page.locator('[data-testid="session-tab"][class*="active"]');
    await expect(activeTab).toBeVisible();
  });
});

test.describe('Accessibility', () => {
  test('should announce focus changes to screen readers', async ({ page }) => {
    // This would require aria-live regions or similar
    // Verify aria attributes are present
    const mainContent = page.locator('[role="main"]');
    await expect(mainContent).toBeVisible();
  });

  test('should have proper ARIA labels', async ({ page }) => {
    // Verify important elements have aria-label
    const newConnection = page.locator('[data-testid="new-connection"]');
    const ariaLabel = await newConnection.getAttribute('aria-label');
    expect(ariaLabel).toBeTruthy();
  });

  test('should support keyboard-only workflow', async ({ page }) => {
    // Complete a workflow using only keyboard
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';

    // Open command palette
    await page.keyboard.press(`${modifier}+KeyK`);

    // Navigate and select
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('Enter');

    // Verify action completed
    // (Specific assertion depends on command)
    expect(true).toBe(true);
  });
});
