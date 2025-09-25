import { test as base, expect } from '@playwright/test';

// Extend the base test with custom fixtures
export const test = base.extend<{
  authenticatedPage: any;
}>({
  authenticatedPage: async ({ page }, use) => {
    // Navigate to login page
    await page.goto('/login');
    
    // Fill in login credentials
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    
    // Click login button
    await page.click('[data-testid="login-button"]');
    
    // Wait for navigation to dashboard
    await page.waitForURL('/dashboard');
    
    // Verify authentication
    await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
    
    await use(page);
  },
});

export { expect } from '@playwright/test';