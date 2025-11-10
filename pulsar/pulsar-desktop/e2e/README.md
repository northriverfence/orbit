# Pulsar Desktop - E2E Testing Guide

This directory contains end-to-end (E2E) tests for Pulsar Desktop using Playwright.

## Overview

E2E tests validate complete user workflows from start to finish, simulating real user interactions with the application.

## Structure

```
e2e/
├── fixtures/          # Test fixtures and helpers
├── utils/            # Utility functions for tests
├── app.spec.ts       # Application launch and basic functionality
├── connection.spec.ts # SSH connection workflows
├── sessions.spec.ts  # Session management tests
├── file-transfer.spec.ts # File transfer tests
├── vault.spec.ts     # Vault and credential management
├── settings.spec.ts  # Settings dialog tests
├── keyboard.spec.ts  # Keyboard navigation and shortcuts
└── README.md         # This file
```

## Running Tests

### Run all tests
```bash
npm run test:e2e
```

### Run specific test file
```bash
npx playwright test e2e/connection.spec.ts
```

### Run in headed mode (with visible browser)
```bash
npm run test:e2e:headed
```

### Run in debug mode
```bash
npm run test:e2e:debug
```

### Run with UI mode (interactive)
```bash
npm run test:e2e:ui
```

### View test report
```bash
npm run test:e2e:report
```

### Run specific test by name
```bash
npx playwright test -g "should create connection with password"
```

## Test Categories

### 1. App Launch Tests (`app.spec.ts`)
- Application launches successfully
- Main window renders
- Initial UI state is correct

### 2. Connection Tests (`connection.spec.ts`)
- Create new SSH connection
- Connect with password
- Connect with SSH key
- Connection error handling
- Retry failed connections

### 3. Session Management (`sessions.spec.ts`)
- Create multiple sessions
- Switch between sessions
- Close sessions
- Session restoration on app restart
- Tab management

### 4. File Transfer (`file-transfer.spec.ts`)
- Upload files via WebTransport
- Download files
- Progress tracking
- Transfer cancellation
- Resume failed transfers

### 5. Vault Management (`vault.spec.ts`)
- Unlock vault
- Save credentials
- Retrieve credentials
- SSH key management
- Auto-unlock functionality

### 6. Settings (`settings.spec.ts`)
- Open settings dialog
- Update settings
- Settings persistence
- Tab navigation
- Keyboard shortcuts

### 7. Keyboard Navigation (`keyboard.spec.ts`)
- Command palette (Ctrl/Cmd+K)
- Settings shortcuts (Ctrl/Cmd+,)
- Tab navigation
- Arrow key navigation
- Focus management

## Writing New Tests

### Test Template

```typescript
import { test, expect } from '@playwright/test';

test.describe('Feature Name', () => {
  test.beforeEach(async ({ page }) => {
    // Setup before each test
  });

  test('should do something', async ({ page }) => {
    // Arrange
    // Act
    // Assert
    expect(true).toBe(true);
  });

  test.afterEach(async ({ page }) => {
    // Cleanup after each test
  });
});
```

### Best Practices

1. **Use descriptive test names** - Name should describe what the test does
2. **Follow AAA pattern** - Arrange, Act, Assert
3. **Use page objects** - Abstract common interactions
4. **Wait for elements** - Always wait for elements before interaction
5. **Clean up** - Reset state after tests
6. **Make tests independent** - Each test should be runnable in isolation

### Example Test

```typescript
test('should create new SSH connection', async ({ page }) => {
  // Arrange
  await page.goto('/');

  // Act
  await page.click('[data-testid="new-connection"]');
  await page.fill('[data-testid="host"]', 'example.com');
  await page.fill('[data-testid="username"]', 'testuser');
  await page.click('[data-testid="connect"]');

  // Assert
  await expect(page.locator('[data-testid="terminal"]')).toBeVisible();
});
```

## Tauri-Specific Considerations

### Application Launch

For Tauri apps, you need to launch the actual application binary:

```typescript
// Example using Tauri's test helpers
import { spawn } from 'child_process';

// Launch Tauri app
const app = spawn('cargo', ['tauri', 'dev']);
```

### Custom Protocol

Tauri uses custom protocols (`tauri://localhost`), so standard web testing may need adaptation.

### IPC Communication

For testing Tauri commands, use the Tauri API:

```typescript
// Example
await page.evaluate(() => {
  return window.__TAURI__.invoke('command_name', { arg: 'value' });
});
```

## CI/CD Integration

### GitHub Actions Workflow

A complete GitHub Actions workflow is configured in `.github/workflows/e2e-tests.yml`:

**Features:**
- Runs on Ubuntu, macOS, and Windows
- Installs Playwright browsers with dependencies
- Builds Tauri application
- Runs E2E tests in headless mode
- Uploads test reports, screenshots, and videos on failure
- Generates test summary for each platform

**Workflow runs on:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches

**Artifacts generated:**
- Test results (JSON reports)
- HTML reports
- Screenshots (on failure)
- Videos (on failure)

### Local CI Testing

To test locally as CI would run:

```bash
CI=true npm run test:e2e
```

## Debugging Tests

### Visual Debugging

```bash
npx playwright test --debug
```

### Generate Test Code

Playwright can record your actions and generate test code:

```bash
npx playwright codegen tauri://localhost
```

### View Trace Files

```bash
npx playwright show-trace trace.zip
```

## Coverage

E2E tests should cover:

- ✅ Critical user paths (connection, file transfer)
- ✅ Error scenarios (network failures, invalid inputs)
- ✅ Keyboard navigation (accessibility)
- ✅ Multi-session workflows
- ✅ Settings and configuration

## Resources

- [Playwright Documentation](https://playwright.dev/)
- [Tauri Testing Guide](https://tauri.app/v1/guides/testing/)
- [Best Practices](https://playwright.dev/docs/best-practices)

---

**Last Updated**: November 10, 2025
**Test Framework**: Playwright v1.x
**Application**: Pulsar Desktop (Orbit)
