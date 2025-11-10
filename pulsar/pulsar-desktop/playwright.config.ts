import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E Test Configuration for Pulsar Desktop
 *
 * This configuration is optimized for Tauri desktop applications
 * and supports both local development and CI/CD environments.
 */
export default defineConfig({
  // Test directory
  testDir: './e2e',

  // Maximum time one test can run
  timeout: 30000,

  // Global setup timeout
  globalTimeout: 120000,

  // Expect timeout for assertions
  expect: {
    timeout: 5000,
  },

  // Run tests in files in parallel
  fullyParallel: false, // Tauri app tests should run serially

  // Fail the build on CI if you accidentally left test.only in the source code
  forbidOnly: !!process.env.CI,

  // Retry on CI only
  retries: process.env.CI ? 2 : 0,

  // Opt out of parallel tests on CI
  workers: 1, // Tauri apps work best with serial execution

  // Reporter to use
  reporter: [
    ['list'],
    ['html', { open: 'never', outputFolder: 'playwright-report' }],
    ['json', { outputFile: 'playwright-report/results.json' }],
  ],

  // Shared settings for all projects
  use: {
    // Base URL for the app (Tauri uses custom protocol)
    // baseURL: 'tauri://localhost',

    // Collect trace when retrying the failed test
    trace: 'on-first-retry',

    // Screenshot on failure
    screenshot: 'only-on-failure',

    // Video on failure
    video: 'retain-on-failure',

    // Emulate viewport
    viewport: { width: 1280, height: 720 },
  },

  // Configure projects for different scenarios
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        headless: true, // Always headless for CI/CD
      },
    },

    // Uncomment when testing on multiple browsers
    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },

    // {
    //   name: 'webkit',
    //   use: { ...devices['Desktop Safari'] },
    // },
  ],

  // Web server configuration (for Tauri dev server)
  webServer: undefined, // Tauri manages its own server

  // Global setup (if needed)
  // globalSetup: require.resolve('./e2e/global-setup.ts'),

  // Global teardown (if needed)
  // globalTeardown: require.resolve('./e2e/global-teardown.ts'),
});
