import { test, expect } from '@playwright/test';

/**
 * SSH Connection E2E Tests
 *
 * Tests for creating, managing, and using SSH connections.
 * These tests cover the complete connection workflow.
 */

test.describe('SSH Connection Creation', () => {
  test.beforeEach(async ({ page }) => {
    // Open connection dialog
    await page.click('[data-testid="new-connection"]');
    await expect(page.locator('[data-testid="connection-dialog"]')).toBeVisible();
  });

  test('should create connection with password', async ({ page }) => {
    // Fill in connection details
    await page.fill('[data-testid="host"]', 'test.example.com');
    await page.fill('[data-testid="port"]', '22');
    await page.fill('[data-testid="username"]', 'testuser');

    // Select password authentication
    await page.click('[data-testid="auth-method-password"]');
    await page.fill('[data-testid="password"]', 'testpass123');

    // Click connect
    await page.click('[data-testid="connect-button"]');

    // Verify loading state
    await expect(page.locator('[data-testid="loading-spinner"]')).toBeVisible();

    // Note: Actual connection would require mock SSH server
    // This tests the UI flow only
  });

  test('should create connection with SSH key', async ({ page }) => {
    // Fill in connection details
    await page.fill('[data-testid="host"]', 'test.example.com');
    await page.fill('[data-testid="username"]', 'testuser');

    // Select SSH key authentication
    await page.click('[data-testid="auth-method-key"]');

    // Select key from vault
    await page.click('[data-testid="select-key"]');
    await expect(page.locator('[data-testid="key-selector"]')).toBeVisible();

    // Choose a key
    await page.click('[data-testid="key-option-0"]');

    // Click connect
    await page.click('[data-testid="connect-button"]');

    // Verify loading state
    await expect(page.locator('[data-testid="loading-spinner"]')).toBeVisible();
  });

  test('should validate required fields', async ({ page }) => {
    // Click connect without filling fields
    await page.click('[data-testid="connect-button"]');

    // Verify validation errors
    await expect(page.locator('[data-testid="error-host"]')).toBeVisible();
    await expect(page.locator('[data-testid="error-username"]')).toBeVisible();
  });

  test('should show error for invalid hostname', async ({ page }) => {
    // Fill with invalid hostname
    await page.fill('[data-testid="host"]', 'invalid host name with spaces');
    await page.fill('[data-testid="username"]', 'testuser');

    // Click connect
    await page.click('[data-testid="connect-button"]');

    // Verify validation error
    await expect(page.locator('[data-testid="error-host"]')).toContainText(/invalid/i);
  });

  test('should connect with keyboard shortcut Ctrl+Enter', async ({ page }) => {
    // Fill in minimum required fields
    await page.fill('[data-testid="host"]', 'test.example.com');
    await page.fill('[data-testid="username"]', 'testuser');
    await page.fill('[data-testid="password"]', 'testpass');

    // Press Ctrl+Enter
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+Enter`);

    // Verify connection attempt
    await expect(page.locator('[data-testid="loading-spinner"]')).toBeVisible();
  });
});

test.describe('Connection Error Handling', () => {
  test('should show error for connection refused', async ({ page }) => {
    // This test would simulate a connection refusal
    // Requires mock SSH server or controlled environment

    // Open dialog and fill details
    await page.click('[data-testid="new-connection"]');
    await page.fill('[data-testid="host"]', 'localhost');
    await page.fill('[data-testid="port"]', '9999'); // Non-existent port
    await page.fill('[data-testid="username"]', 'test');

    // Attempt connect
    await page.click('[data-testid="connect-button"]');

    // Verify error shown
    await expect(page.locator('[data-testid="error-alert"]')).toBeVisible({
      timeout: 15000,
    });
    await expect(page.locator('[data-testid="error-alert"]')).toContainText(
      /connection.*failed|refused/i
    );
  });

  test('should show retry button after connection failure', async ({ page }) => {
    // After connection failure, verify retry option
    await page.click('[data-testid="new-connection"]');
    await page.fill('[data-testid="host"]', 'invalid.host.invalid');
    await page.fill('[data-testid="username"]', 'test');
    await page.click('[data-testid="connect-button"]');

    // Wait for error
    await expect(page.locator('[data-testid="error-alert"]')).toBeVisible({
      timeout: 15000,
    });

    // Verify retry button
    await expect(page.locator('[data-testid="retry-button"]')).toBeVisible();
  });

  test('should retry connection on button click', async ({ page }) => {
    // After connection failure, click retry
    await page.click('[data-testid="new-connection"]');
    await page.fill('[data-testid="host"]', 'invalid.host');
    await page.fill('[data-testid="username"]', 'test');
    await page.click('[data-testid="connect-button"]');

    // Wait for error
    await expect(page.locator('[data-testid="error-alert"]')).toBeVisible({
      timeout: 15000,
    });

    // Click retry
    await page.click('[data-testid="retry-button"]');

    // Verify loading state again
    await expect(page.locator('[data-testid="loading-spinner"]')).toBeVisible();
  });
});

test.describe('Connection Management', () => {
  test('should save connection for reuse', async ({ page }) => {
    // Create connection with "Save" option
    await page.click('[data-testid="new-connection"]');
    await page.fill('[data-testid="host"]', 'test.example.com');
    await page.fill('[data-testid="username"]', 'testuser');

    // Check save checkbox
    await page.click('[data-testid="save-connection"]');
    await page.fill('[data-testid="connection-name"]', 'My Test Server');

    // Connect
    await page.click('[data-testid="connect-button"]');

    // Verify connection appears in saved list
    await expect(page.locator('[data-testid="saved-connections"]')).toContainText(
      'My Test Server'
    );
  });

  test('should edit saved connection', async ({ page }) => {
    // Assuming a saved connection exists
    // Click edit on saved connection
    await page.click('[data-testid="edit-connection-0"]');

    // Verify dialog opens with pre-filled data
    await expect(page.locator('[data-testid="connection-dialog"]')).toBeVisible();
    await expect(page.locator('[data-testid="host"]')).not.toBeEmpty();

    // Modify and save
    await page.fill('[data-testid="host"]', 'updated.example.com');
    await page.click('[data-testid="save-button"]');

    // Verify update
    await expect(page.locator('[data-testid="saved-connections"]')).toContainText(
      'updated.example.com'
    );
  });

  test('should delete saved connection', async ({ page }) => {
    // Assuming a saved connection exists
    // Click delete
    await page.click('[data-testid="delete-connection-0"]');

    // Confirm deletion
    await page.click('[data-testid="confirm-delete"]');

    // Verify connection removed
    // (Implementation depends on how empty state is shown)
  });
});

test.describe('Connection Quick Access', () => {
  test('should connect to recent connection from command palette', async ({ page }) => {
    // Open command palette
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control';
    await page.keyboard.press(`${modifier}+KeyK`);

    // Type "connect"
    await page.fill('[data-testid="command-search"]', 'connect');

    // Select recent connection
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('Enter');

    // Verify connection starts
    await expect(page.locator('[data-testid="terminal"]')).toBeVisible({
      timeout: 10000,
    });
  });
});
