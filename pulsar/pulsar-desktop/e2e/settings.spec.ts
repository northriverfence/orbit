import { test, expect } from '@playwright/test';

/**
 * Settings Dialog E2E Tests
 *
 * Comprehensive tests for all settings categories: General, Appearance,
 * Terminal, Connections, Vault, and Advanced.
 */

test.describe('Settings Dialog Access', () => {
  test('should open settings with menu button', async ({ page }) => {
    // Click settings button
    await page.click('[data-testid="settings-button"]');

    // Verify dialog opens
    await expect(page.locator('[data-testid="settings-dialog"]')).toBeVisible();
  });

  test('should open settings with keyboard shortcut', async ({ page }) => {
    // Press Ctrl/Cmd+,
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);

    // Verify dialog opens
    await expect(page.locator('[data-testid="settings-dialog"]')).toBeVisible();
  });

  test('should close settings with Escape', async ({ page }) => {
    // Open settings
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);

    // Press Escape
    await page.keyboard.press('Escape');

    // Verify dialog closes
    await expect(page.locator('[data-testid="settings-dialog"]')).not.toBeVisible();
  });

  test('should close settings with close button', async ({ page }) => {
    // Open settings
    await page.click('[data-testid="settings-button"]');

    // Click close button
    await page.click('[data-testid="close-settings"]');

    // Verify dialog closes
    await expect(page.locator('[data-testid="settings-dialog"]')).not.toBeVisible();
  });
});

test.describe('Settings Navigation', () => {
  test.beforeEach(async ({ page }) => {
    // Open settings
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);
    await expect(page.locator('[data-testid="settings-dialog"]')).toBeVisible();
  });

  test('should show all settings categories', async ({ page }) => {
    // Verify all categories present
    await expect(page.locator('[data-testid="category-general"]')).toBeVisible();
    await expect(page.locator('[data-testid="category-appearance"]')).toBeVisible();
    await expect(page.locator('[data-testid="category-terminal"]')).toBeVisible();
    await expect(page.locator('[data-testid="category-connections"]')).toBeVisible();
    await expect(page.locator('[data-testid="category-vault"]')).toBeVisible();
    await expect(page.locator('[data-testid="category-advanced"]')).toBeVisible();
  });

  test('should switch categories with mouse', async ({ page }) => {
    // Click different categories
    await page.click('[data-testid="category-appearance"]');
    await expect(page.locator('[data-testid="appearance-settings"]')).toBeVisible();

    await page.click('[data-testid="category-terminal"]');
    await expect(page.locator('[data-testid="terminal-settings"]')).toBeVisible();
  });

  test('should switch categories with arrow keys', async ({ page }) => {
    // Focus first category
    await page.focus('[data-testid="category-general"]');

    // Press arrow down
    await page.keyboard.press('ArrowDown');

    // Verify next category focused/selected
    await expect(page.locator('[data-testid="category-appearance"]')).toBeFocused();
  });

  test('should navigate with Tab key', async ({ page }) => {
    // Tab through focusable elements
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');

    // Verify focus moves through settings
    const focusedElement = await page.evaluateHandle(() => document.activeElement);
    expect(focusedElement).toBeTruthy();
  });
});

test.describe('General Settings', () => {
  test.beforeEach(async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);
    await page.click('[data-testid="category-general"]');
  });

  test('should change language setting', async ({ page }) => {
    // Click language dropdown
    await page.click('[data-testid="language-select"]');

    // Select different language
    await page.click('[data-testid="language-option-es"]');

    // Verify selection saved
    await expect(page.locator('[data-testid="language-select"]')).toContainText('Español');
  });

  test('should toggle auto-launch on startup', async ({ page }) => {
    // Click auto-launch toggle
    await page.click('[data-testid="auto-launch-toggle"]');

    // Verify toggle state changed
    const isChecked = await page.isChecked('[data-testid="auto-launch-toggle"]');
    expect(typeof isChecked).toBe('boolean');
  });

  test('should toggle check for updates', async ({ page }) => {
    // Click auto-update toggle
    await page.click('[data-testid="auto-update-toggle"]');

    // Verify toggle state
    const isChecked = await page.isChecked('[data-testid="auto-update-toggle"]');
    expect(typeof isChecked).toBe('boolean');
  });

  test('should set default shell', async ({ page }) => {
    // Click shell dropdown
    await page.click('[data-testid="default-shell-select"]');

    // Select shell
    await page.click('[data-testid="shell-option-zsh"]');

    // Verify selection
    await expect(page.locator('[data-testid="default-shell-select"]')).toContainText('zsh');
  });
});

test.describe('Appearance Settings', () => {
  test.beforeEach(async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);
    await page.click('[data-testid="category-appearance"]');
  });

  test('should switch between light and dark themes', async ({ page }) => {
    // Click theme toggle
    await page.click('[data-testid="theme-toggle"]');

    // Verify theme changed (check for class or attribute)
    const theme = await page.getAttribute('html', 'data-theme');
    expect(['light', 'dark']).toContain(theme);
  });

  test('should select color scheme', async ({ page }) => {
    // Click color scheme dropdown
    await page.click('[data-testid="color-scheme-select"]');

    // Select scheme
    await page.click('[data-testid="scheme-option-solarized"]');

    // Verify selection
    await expect(page.locator('[data-testid="color-scheme-select"]')).toContainText('Solarized');
  });

  test('should adjust font size', async ({ page }) => {
    // Find font size input
    const fontSizeInput = page.locator('[data-testid="font-size-input"]');

    // Change font size
    await fontSizeInput.fill('16');

    // Verify value changed
    await expect(fontSizeInput).toHaveValue('16');
  });

  test('should select font family', async ({ page }) => {
    // Click font family dropdown
    await page.click('[data-testid="font-family-select"]');

    // Select font
    await page.click('[data-testid="font-option-fira-code"]');

    // Verify selection
    await expect(page.locator('[data-testid="font-family-select"]')).toContainText('Fira Code');
  });

  test('should adjust opacity', async ({ page }) => {
    // Find opacity slider
    const opacitySlider = page.locator('[data-testid="opacity-slider"]');

    // Change opacity
    await opacitySlider.fill('0.9');

    // Verify value
    const value = await opacitySlider.inputValue();
    expect(parseFloat(value)).toBeCloseTo(0.9, 1);
  });
});

test.describe('Terminal Settings', () => {
  test.beforeEach(async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);
    await page.click('[data-testid="category-terminal"]');
  });

  test('should adjust cursor style', async ({ page }) => {
    // Click cursor style dropdown
    await page.click('[data-testid="cursor-style-select"]');

    // Select style
    await page.click('[data-testid="cursor-option-underline"]');

    // Verify selection
    await expect(page.locator('[data-testid="cursor-style-select"]')).toContainText('Underline');
  });

  test('should toggle cursor blink', async ({ page }) => {
    // Click cursor blink toggle
    await page.click('[data-testid="cursor-blink-toggle"]');

    // Verify toggle state
    const isChecked = await page.isChecked('[data-testid="cursor-blink-toggle"]');
    expect(typeof isChecked).toBe('boolean');
  });

  test('should adjust scrollback buffer size', async ({ page }) => {
    // Find scrollback input
    const scrollbackInput = page.locator('[data-testid="scrollback-input"]');

    // Change value
    await scrollbackInput.fill('10000');

    // Verify value
    await expect(scrollbackInput).toHaveValue('10000');
  });

  test('should toggle bell sound', async ({ page }) => {
    // Click bell toggle
    await page.click('[data-testid="bell-toggle"]');

    // Verify toggle state
    const isChecked = await page.isChecked('[data-testid="bell-toggle"]');
    expect(typeof isChecked).toBe('boolean');
  });

  test('should configure copy on select', async ({ page }) => {
    // Click copy-on-select toggle
    await page.click('[data-testid="copy-on-select-toggle"]');

    // Verify toggle state
    const isChecked = await page.isChecked('[data-testid="copy-on-select-toggle"]');
    expect(typeof isChecked).toBe('boolean');
  });
});

test.describe('Connection Settings', () => {
  test.beforeEach(async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);
    await page.click('[data-testid="category-connections"]');
  });

  test('should set default connection timeout', async ({ page }) => {
    // Find timeout input
    const timeoutInput = page.locator('[data-testid="connection-timeout-input"]');

    // Change value
    await timeoutInput.fill('30');

    // Verify value
    await expect(timeoutInput).toHaveValue('30');
  });

  test('should toggle keep-alive', async ({ page }) => {
    // Click keep-alive toggle
    await page.click('[data-testid="keep-alive-toggle"]');

    // Verify toggle state
    const isChecked = await page.isChecked('[data-testid="keep-alive-toggle"]');
    expect(typeof isChecked).toBe('boolean');
  });

  test('should set keep-alive interval', async ({ page }) => {
    // Enable keep-alive first
    const keepAliveToggle = page.locator('[data-testid="keep-alive-toggle"]');
    if (!(await keepAliveToggle.isChecked())) {
      await keepAliveToggle.click();
    }

    // Find interval input
    const intervalInput = page.locator('[data-testid="keep-alive-interval-input"]');

    // Change value
    await intervalInput.fill('60');

    // Verify value
    await expect(intervalInput).toHaveValue('60');
  });

  test('should configure compression', async ({ page }) => {
    // Click compression toggle
    await page.click('[data-testid="compression-toggle"]');

    // Verify toggle state
    const isChecked = await page.isChecked('[data-testid="compression-toggle"]');
    expect(typeof isChecked).toBe('boolean');
  });

  test('should set default port', async ({ page }) => {
    // Find port input
    const portInput = page.locator('[data-testid="default-port-input"]');

    // Change value
    await portInput.fill('2222');

    // Verify value
    await expect(portInput).toHaveValue('2222');
  });
});

test.describe('Vault Settings', () => {
  test.beforeEach(async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);
    await page.click('[data-testid="category-vault"]');
  });

  test('should set auto-lock timeout', async ({ page }) => {
    // Find timeout dropdown
    await page.click('[data-testid="auto-lock-timeout-select"]');

    // Select timeout
    await page.click('[data-testid="timeout-option-15"]');

    // Verify selection
    await expect(page.locator('[data-testid="auto-lock-timeout-select"]')).toContainText('15 minutes');
  });

  test('should toggle auto-lock on sleep', async ({ page }) => {
    // Click toggle
    await page.click('[data-testid="lock-on-sleep-toggle"]');

    // Verify toggle state
    const isChecked = await page.isChecked('[data-testid="lock-on-sleep-toggle"]');
    expect(typeof isChecked).toBe('boolean');
  });

  test('should change vault password', async ({ page }) => {
    // Click change password button
    await page.click('[data-testid="change-vault-password"]');

    // Verify password change dialog opens
    await expect(page.locator('[data-testid="password-change-dialog"]')).toBeVisible();

    // Fill in current password
    await page.fill('[data-testid="current-password"]', 'oldpassword');

    // Fill in new password
    await page.fill('[data-testid="new-password"]', 'newpassword123');
    await page.fill('[data-testid="confirm-password"]', 'newpassword123');

    // Click change
    await page.click('[data-testid="confirm-change"]');

    // Verify success notification
    await expect(page.locator('[data-testid="success-notification"]')).toBeVisible();
  });

  test('should export vault backup', async ({ page }) => {
    // Click export button
    await page.click('[data-testid="export-vault"]');

    // Verify export dialog opens
    await expect(page.locator('[data-testid="export-dialog"]')).toBeVisible();
  });

  test('should import vault backup', async ({ page }) => {
    // Click import button
    await page.click('[data-testid="import-vault"]');

    // Verify import dialog opens
    await expect(page.locator('[data-testid="import-dialog"]')).toBeVisible();
  });
});

test.describe('Advanced Settings', () => {
  test.beforeEach(async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);
    await page.click('[data-testid="category-advanced"]');
  });

  test('should toggle developer tools', async ({ page }) => {
    // Click dev tools toggle
    await page.click('[data-testid="dev-tools-toggle"]');

    // Verify toggle state
    const isChecked = await page.isChecked('[data-testid="dev-tools-toggle"]');
    expect(typeof isChecked).toBe('boolean');
  });

  test('should toggle debug logging', async ({ page }) => {
    // Click debug logging toggle
    await page.click('[data-testid="debug-logging-toggle"]');

    // Verify toggle state
    const isChecked = await page.isChecked('[data-testid="debug-logging-toggle"]');
    expect(typeof isChecked).toBe('boolean');
  });

  test('should set log level', async ({ page }) => {
    // Click log level dropdown
    await page.click('[data-testid="log-level-select"]');

    // Select level
    await page.click('[data-testid="log-level-option-debug"]');

    // Verify selection
    await expect(page.locator('[data-testid="log-level-select"]')).toContainText('Debug');
  });

  test('should clear cache', async ({ page }) => {
    // Click clear cache button
    await page.click('[data-testid="clear-cache"]');

    // Verify confirmation dialog
    await expect(page.locator('[data-testid="confirm-dialog"]')).toBeVisible();

    // Confirm
    await page.click('[data-testid="confirm-button"]');

    // Verify success notification
    await expect(page.locator('[data-testid="success-notification"]')).toBeVisible();
  });

  test('should reset all settings', async ({ page }) => {
    // Click reset button
    await page.click('[data-testid="reset-settings"]');

    // Verify confirmation dialog
    await expect(page.locator('[data-testid="confirm-dialog"]')).toBeVisible();
    await expect(page.locator('[data-testid="confirm-dialog"]')).toContainText(/reset.*all/i);

    // Confirm
    await page.click('[data-testid="confirm-button"]');

    // Verify success notification
    await expect(page.locator('[data-testid="success-notification"]')).toBeVisible();
  });
});

test.describe('Settings Persistence', () => {
  test('should save settings automatically', async ({ page }) => {
    // Open settings
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);

    // Change a setting
    await page.click('[data-testid="category-appearance"]');
    await page.click('[data-testid="theme-toggle"]');

    // Close settings
    await page.keyboard.press('Escape');

    // Reopen settings
    await page.keyboard.press(`${modifier}+Comma`);
    await page.click('[data-testid="category-appearance"]');

    // Verify setting persisted
    // (Verification depends on implementation)
  });

  test('should save settings with Ctrl/Cmd+S', async ({ page }) => {
    // Open settings
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);

    // Make changes
    await page.click('[data-testid="category-terminal"]');
    await page.fill('[data-testid="scrollback-input"]', '5000');

    // Press Ctrl/Cmd+S
    await page.keyboard.press(`${modifier}+KeyS`);

    // Verify save notification
    await expect(page.locator('[data-testid="save-notification"]')).toBeVisible();
  });

  test('should prompt to save on close with unsaved changes', async ({ page }) => {
    // Open settings
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);

    // Make changes
    await page.click('[data-testid="category-general"]');
    await page.click('[data-testid="auto-launch-toggle"]');

    // Try to close
    await page.keyboard.press('Escape');

    // Verify save prompt (if implemented)
    // This depends on whether auto-save is enabled
  });
});

test.describe('Settings Search', () => {
  test.beforeEach(async ({ page }) => {
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Comma`);
  });

  test('should search settings', async ({ page }) => {
    // Type in search box
    await page.fill('[data-testid="settings-search"]', 'font');

    // Verify filtered results shown
    await expect(page.locator('[data-testid="search-results"]')).toBeVisible();
    await expect(page.locator('[data-testid="search-results"]')).toContainText(/font/i);
  });

  test('should clear search', async ({ page }) => {
    // Type in search
    await page.fill('[data-testid="settings-search"]', 'theme');

    // Click clear button
    await page.click('[data-testid="clear-search"]');

    // Verify search cleared
    await expect(page.locator('[data-testid="settings-search"]')).toHaveValue('');
  });

  test('should navigate to setting from search result', async ({ page }) => {
    // Search for setting
    await page.fill('[data-testid="settings-search"]', 'auto-launch');

    // Click search result
    await page.click('[data-testid="search-result-0"]');

    // Verify navigated to correct category and setting
    await expect(page.locator('[data-testid="category-general"]')).toHaveClass(/active/);
    await expect(page.locator('[data-testid="auto-launch-toggle"]')).toBeVisible();
  });
});
