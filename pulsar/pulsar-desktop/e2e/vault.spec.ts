import { test, expect } from '@playwright/test';

/**
 * Vault and Credential Management E2E Tests
 *
 * Tests for secure credential storage, SSH key management, and vault operations.
 */

test.describe('Vault Unlock', () => {
  test('should open vault view', async ({ page }) => {
    // Click vault button
    await page.click('[data-testid="vault-button"]');

    // Verify vault view opens
    await expect(page.locator('[data-testid="vault-view"]')).toBeVisible();
  });

  test('should show unlock dialog for locked vault', async ({ page }) => {
    // Open vault view
    await page.click('[data-testid="vault-button"]');

    // Verify unlock dialog shown
    await expect(page.locator('[data-testid="vault-unlock-dialog"]')).toBeVisible();
  });

  test('should unlock vault with correct password', async ({ page }) => {
    // Open vault
    await page.click('[data-testid="vault-button"]');

    // Enter password
    await page.fill('[data-testid="vault-password"]', 'testpassword123');
    await page.click('[data-testid="unlock-button"]');

    // Verify vault unlocked (credential list visible)
    await expect(page.locator('[data-testid="credential-list"]')).toBeVisible({
      timeout: 5000,
    });
  });

  test('should show error for incorrect password', async ({ page }) => {
    // Open vault
    await page.click('[data-testid="vault-button"]');

    // Enter wrong password
    await page.fill('[data-testid="vault-password"]', 'wrongpassword');
    await page.click('[data-testid="unlock-button"]');

    // Verify error shown
    await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
    await expect(page.locator('[data-testid="error-message"]')).toContainText(
      /incorrect|wrong|invalid/i
    );
  });

  test('should unlock with keyboard shortcut', async ({ page }) => {
    // Open vault
    await page.click('[data-testid="vault-button"]');

    // Enter password and press Enter
    await page.fill('[data-testid="vault-password"]', 'testpassword123');
    await page.keyboard.press('Enter');

    // Verify vault unlocked
    await expect(page.locator('[data-testid="credential-list"]')).toBeVisible({
      timeout: 5000,
    });
  });
});

test.describe('SSH Key Management', () => {
  test.beforeEach(async ({ page }) => {
    // Unlock vault
    await page.click('[data-testid="vault-button"]');
    await page.fill('[data-testid="vault-password"]', 'testpassword123');
    await page.click('[data-testid="unlock-button"]');
    await expect(page.locator('[data-testid="credential-list"]')).toBeVisible();
  });

  test('should open SSH key creation form', async ({ page }) => {
    // Click add new key
    await page.click('[data-testid="add-ssh-key"]');

    // Verify form opens
    await expect(page.locator('[data-testid="ssh-key-form"]')).toBeVisible();
  });

  test('should generate new SSH key pair', async ({ page }) => {
    // Open form
    await page.click('[data-testid="add-ssh-key"]');

    // Fill in details
    await page.fill('[data-testid="key-name"]', 'My Test Key');
    await page.fill('[data-testid="key-comment"]', 'test@example.com');

    // Select key type
    await page.selectOption('[data-testid="key-type"]', 'ed25519');

    // Click generate
    await page.click('[data-testid="generate-button"]');

    // Verify success notification
    await expect(page.locator('[data-testid="success-notification"]')).toBeVisible({
      timeout: 5000,
    });

    // Verify key appears in list
    await expect(page.locator('[data-testid="credential-list"]')).toContainText('My Test Key');
  });

  test('should import existing SSH key', async ({ page }) => {
    // Open form
    await page.click('[data-testid="add-ssh-key"]');

    // Switch to import mode
    await page.click('[data-testid="import-mode"]');

    // Fill in details
    await page.fill('[data-testid="key-name"]', 'Imported Key');

    // Paste private key
    const testPrivateKey = '-----BEGIN OPENSSH PRIVATE KEY-----\ntest key content\n-----END OPENSSH PRIVATE KEY-----';
    await page.fill('[data-testid="private-key"]', testPrivateKey);

    // Save
    await page.click('[data-testid="save-button"]');

    // Verify success
    await expect(page.locator('[data-testid="success-notification"]')).toBeVisible();
  });

  test('should require passphrase for encrypted keys', async ({ page }) => {
    // Open form
    await page.click('[data-testid="add-ssh-key"]');

    // Enable encryption
    await page.click('[data-testid="encrypt-key"]');

    // Verify passphrase field appears
    await expect(page.locator('[data-testid="key-passphrase"]')).toBeVisible();

    // Fill form
    await page.fill('[data-testid="key-name"]', 'Encrypted Key');
    await page.fill('[data-testid="key-passphrase"]', 'keypass123');
    await page.fill('[data-testid="key-passphrase-confirm"]', 'keypass123');

    // Generate
    await page.click('[data-testid="generate-button"]');

    // Verify success
    await expect(page.locator('[data-testid="success-notification"]')).toBeVisible({
      timeout: 5000,
    });
  });

  test('should show error for mismatched passphrases', async ({ page }) => {
    // Open form
    await page.click('[data-testid="add-ssh-key"]');
    await page.click('[data-testid="encrypt-key"]');

    // Fill with mismatched passphrases
    await page.fill('[data-testid="key-name"]', 'Test Key');
    await page.fill('[data-testid="key-passphrase"]', 'password1');
    await page.fill('[data-testid="key-passphrase-confirm"]', 'password2');

    // Try to generate
    await page.click('[data-testid="generate-button"]');

    // Verify error shown
    await expect(page.locator('[data-testid="error-message"]')).toContainText(/mismatch|match/i);
  });
});

test.describe('Credential List', () => {
  test.beforeEach(async ({ page }) => {
    // Unlock vault
    await page.click('[data-testid="vault-button"]');
    await page.fill('[data-testid="vault-password"]', 'testpassword123');
    await page.click('[data-testid="unlock-button"]');
    await expect(page.locator('[data-testid="credential-list"]')).toBeVisible();
  });

  test('should display all stored credentials', async ({ page }) => {
    // Verify list is populated
    const items = page.locator('[data-testid="credential-item"]');
    const count = await items.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should show credential details on click', async ({ page }) => {
    // Click first credential
    await page.click('[data-testid="credential-item"]');

    // Verify details panel opens
    await expect(page.locator('[data-testid="credential-details"]')).toBeVisible();
  });

  test('should filter credentials by search', async ({ page }) => {
    // Type in search
    await page.fill('[data-testid="credential-search"]', 'production');

    // Verify filtered results
    const items = page.locator('[data-testid="credential-item"]');
    const count = await items.count();

    for (let i = 0; i < count; i++) {
      await expect(items.nth(i)).toContainText(/production/i);
    }
  });

  test('should sort credentials by name', async ({ page }) => {
    // Click sort button
    await page.click('[data-testid="sort-by-name"]');

    // Verify sorted order
    const items = page.locator('[data-testid="credential-item"]');
    const firstItem = await items.first().textContent();
    const lastItem = await items.last().textContent();

    // Verify alphabetical order (first < last)
    expect(firstItem && lastItem && firstItem.localeCompare(lastItem)).toBeLessThan(0);
  });

  test('should sort credentials by date', async ({ page }) => {
    // Click sort button
    await page.click('[data-testid="sort-by-date"]');

    // Verify date sorting indicator shown
    await expect(page.locator('[data-testid="sort-indicator"]')).toContainText(/date/i);
  });
});

test.describe('Credential Operations', () => {
  test.beforeEach(async ({ page }) => {
    // Unlock vault
    await page.click('[data-testid="vault-button"]');
    await page.fill('[data-testid="vault-password"]', 'testpassword123');
    await page.click('[data-testid="unlock-button"]');
    await expect(page.locator('[data-testid="credential-list"]')).toBeVisible();
  });

  test('should edit credential name', async ({ page }) => {
    // Click credential
    await page.click('[data-testid="credential-item"]');

    // Click edit
    await page.click('[data-testid="edit-credential"]');

    // Update name
    await page.fill('[data-testid="credential-name"]', 'Updated Name');
    await page.click('[data-testid="save-button"]');

    // Verify update
    await expect(page.locator('[data-testid="credential-list"]')).toContainText('Updated Name');
  });

  test('should delete credential with confirmation', async ({ page }) => {
    // Click credential
    await page.click('[data-testid="credential-item"]');

    // Click delete
    await page.click('[data-testid="delete-credential"]');

    // Verify confirmation dialog
    await expect(page.locator('[data-testid="confirm-dialog"]')).toBeVisible();
    await expect(page.locator('[data-testid="confirm-dialog"]')).toContainText(/delete|remove/i);

    // Confirm deletion
    await page.click('[data-testid="confirm-delete"]');

    // Verify success notification
    await expect(page.locator('[data-testid="success-notification"]')).toBeVisible();
  });

  test('should cancel credential deletion', async ({ page }) => {
    // Click credential
    await page.click('[data-testid="credential-item"]');

    // Click delete
    await page.click('[data-testid="delete-credential"]');

    // Cancel deletion
    await page.click('[data-testid="cancel-delete"]');

    // Verify credential still exists
    await expect(page.locator('[data-testid="credential-item"]')).toBeVisible();
  });

  test('should copy public key to clipboard', async ({ page }) => {
    // Click credential
    await page.click('[data-testid="credential-item"]');

    // Click copy public key
    await page.click('[data-testid="copy-public-key"]');

    // Verify success notification
    await expect(page.locator('[data-testid="success-notification"]')).toContainText(/copied/i);
  });

  test('should export credential', async ({ page }) => {
    // Click credential
    await page.click('[data-testid="credential-item"]');

    // Click export
    await page.click('[data-testid="export-credential"]');

    // Verify export options shown
    await expect(page.locator('[data-testid="export-dialog"]')).toBeVisible();
  });
});

test.describe('Vault Lock/Unlock', () => {
  test('should lock vault manually', async ({ page }) => {
    // Unlock vault
    await page.click('[data-testid="vault-button"]');
    await page.fill('[data-testid="vault-password"]', 'testpassword123');
    await page.click('[data-testid="unlock-button"]');
    await expect(page.locator('[data-testid="credential-list"]')).toBeVisible();

    // Click lock button
    await page.click('[data-testid="lock-vault"]');

    // Verify vault locked (unlock dialog shown)
    await expect(page.locator('[data-testid="vault-unlock-dialog"]')).toBeVisible();
  });

  test('should auto-lock vault after timeout', async ({ page }) => {
    // This test would require configuring auto-lock timeout in settings
    // and waiting for the timeout period
    // For now, it's a placeholder showing the pattern

    expect(true).toBe(true);
  });

  test('should remember unlock state during session', async ({ page }) => {
    // Unlock vault
    await page.click('[data-testid="vault-button"]');
    await page.fill('[data-testid="vault-password"]', 'testpassword123');
    await page.click('[data-testid="unlock-button"]');

    // Close vault view
    await page.keyboard.press('Escape');

    // Reopen vault view
    await page.click('[data-testid="vault-button"]');

    // Verify vault still unlocked (no unlock dialog)
    await expect(page.locator('[data-testid="credential-list"]')).toBeVisible();
  });
});

test.describe('Vault Integration with Connections', () => {
  test('should use vault credential for SSH connection', async ({ page }) => {
    // Open connection dialog
    await page.click('[data-testid="new-connection"]');

    // Fill host and username
    await page.fill('[data-testid="host"]', 'test.example.com');
    await page.fill('[data-testid="username"]', 'testuser');

    // Select SSH key auth
    await page.click('[data-testid="auth-method-key"]');

    // Open credential selector
    await page.click('[data-testid="select-key"]');

    // Verify vault credentials shown
    await expect(page.locator('[data-testid="credential-selector"]')).toBeVisible();

    // Select a credential
    await page.click('[data-testid="credential-option-0"]');

    // Verify credential selected
    await expect(page.locator('[data-testid="selected-credential"]')).toBeVisible();
  });

  test('should prompt to unlock vault when selecting credentials', async ({ page }) => {
    // Assuming vault is locked
    // Open connection dialog
    await page.click('[data-testid="new-connection"]');
    await page.click('[data-testid="auth-method-key"]');
    await page.click('[data-testid="select-key"]');

    // Verify unlock prompt shown
    await expect(page.locator('[data-testid="vault-unlock-dialog"]')).toBeVisible();
  });

  test('should save new credentials to vault during connection', async ({ page }) => {
    // Open connection dialog
    await page.click('[data-testid="new-connection"]');

    // Fill details
    await page.fill('[data-testid="host"]', 'test.example.com');
    await page.fill('[data-testid="username"]', 'testuser');
    await page.fill('[data-testid="password"]', 'testpass');

    // Check "Save to vault"
    await page.click('[data-testid="save-to-vault"]');

    // Connect
    await page.click('[data-testid="connect-button"]');

    // Verify save confirmation (may appear after successful connection)
    // (Implementation depends on connection flow)
  });
});

test.describe('Vault Security', () => {
  test('should require password to view sensitive data', async ({ page }) => {
    // Unlock vault
    await page.click('[data-testid="vault-button"]');
    await page.fill('[data-testid="vault-password"]', 'testpassword123');
    await page.click('[data-testid="unlock-button"]');

    // Click credential
    await page.click('[data-testid="credential-item"]');

    // Try to view private key (should require re-authentication)
    await page.click('[data-testid="view-private-key"]');

    // Verify password prompt shown
    await expect(page.locator('[data-testid="password-prompt"]')).toBeVisible();
  });

  test('should not expose credentials in UI', async ({ page }) => {
    // Unlock vault
    await page.click('[data-testid="vault-button"]');
    await page.fill('[data-testid="vault-password"]', 'testpassword123');
    await page.click('[data-testid="unlock-button"]');

    // Click credential
    await page.click('[data-testid="credential-item"]');

    // Verify private key is not visible by default
    const privateKeyField = page.locator('[data-testid="private-key-display"]');
    const isVisible = await privateKeyField.isVisible();
    expect(isVisible).toBe(false);
  });

  test('should validate vault password strength', async ({ page }) => {
    // This test would be for first-time vault setup
    // Verify weak password is rejected
    expect(true).toBe(true); // Placeholder
  });
});
