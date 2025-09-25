import { test, expect } from '@playwright/test';

test.describe('Security Tests', () => {
  test('prevents XSS attacks in user inputs', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');
    await page.waitForURL('/dashboard');

    await page.click('[data-testid="integration-nav"]');

    // Try to inject XSS in Bitcoin transaction hash field
    const xssPayload = '<script>alert("XSS")</script>';
    await page.fill('[data-testid="btc-tx-hash-input"]', xssPayload);

    // Should not execute script
    page.on('dialog', (dialog) => {
      // If alert appears, XSS was successful (test should fail)
      expect(dialog.type()).not.toBe('alert');
      dialog.dismiss();
    });

    await page.click('[data-testid="execute-deposit-button"]');

    // Input should be sanitized
    const inputValue = await page.inputValue('[data-testid="btc-tx-hash-input"]');
    expect(inputValue).not.toContain('<script>');
  });

  test('validates CSRF protection', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');
    await page.waitForURL('/dashboard');

    // Intercept requests to check for CSRF tokens
    let hasCSRFToken = false;
    
    page.on('request', (request) => {
      if (request.method() === 'POST') {
        const headers = request.headers();
        // Check for CSRF token in headers or form data
        if (headers['x-csrf-token'] || headers['x-xsrf-token']) {
          hasCSRFToken = true;
        }
      }
    });

    await page.click('[data-testid="integration-nav"]');
    await page.fill('[data-testid="btc-amount-input"]', '1.0');
    await page.fill('[data-testid="btc-tx-hash-input"]', 
      'csrf1234567890abcdef1234567890abcdef1234567890abcdef1234567890');
    await page.fill('[data-testid="confirmations-input"]', '6');
    await page.click('[data-testid="execute-deposit-button"]');

    // Should include CSRF protection
    expect(hasCSRFToken).toBe(true);
  });

  test('enforces authentication on protected routes', async ({ page }) => {
    // Try to access protected route without authentication
    await page.goto('/dashboard');

    // Should redirect to login
    await expect(page).toHaveURL(/.*login.*/);

    // Try to access integration page
    await page.goto('/integration');
    await expect(page).toHaveURL(/.*login.*/);

    // Try to access API endpoints directly
    const response = await page.request.get('/api/users/operations');
    expect(response.status()).toBe(401);
  });

  test('validates JWT token expiration', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');
    await page.waitForURL('/dashboard');

    // Mock expired token response
    await page.route('/api/**', (route) => {
      if (route.request().headers()['authorization']) {
        route.fulfill({
          status: 401,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Token expired' }),
        });
      } else {
        route.continue();
      }
    });

    // Try to make authenticated request
    await page.click('[data-testid="refresh-button"]');

    // Should redirect to login on token expiration
    await expect(page).toHaveURL(/.*login.*/);
  });

  test('prevents SQL injection in search inputs', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');
    await page.waitForURL('/dashboard');

    await page.click('[data-testid="integration-nav"]');
    await page.click('[data-testid="operation-history-tab"]');

    // Try SQL injection in search field
    const sqlInjection = "'; DROP TABLE operations; --";
    
    if (await page.locator('[data-testid="search-input"]').isVisible()) {
      await page.fill('[data-testid="search-input"]', sqlInjection);
      await page.press('[data-testid="search-input"]', 'Enter');

      // Should not cause errors or expose database structure
      await expect(page.locator('[data-testid="error-message"]')).not.toContainText('SQL');
      await expect(page.locator('[data-testid="error-message"]')).not.toContainText('database');
    }
  });

  test('validates input length limits', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');
    await page.waitForURL('/dashboard');

    await page.click('[data-testid="integration-nav"]');

    // Try extremely long input
    const longInput = 'a'.repeat(10000);
    await page.fill('[data-testid="btc-tx-hash-input"]', longInput);

    // Should enforce length limits
    const actualValue = await page.inputValue('[data-testid="btc-tx-hash-input"]');
    expect(actualValue.length).toBeLessThan(1000); // Reasonable limit
  });

  test('prevents clickjacking attacks', async ({ page }) => {
    // Check for X-Frame-Options or CSP frame-ancestors
    const response = await page.goto('/dashboard');
    const headers = response?.headers() || {};

    const hasFrameProtection = 
      headers['x-frame-options'] === 'DENY' || 
      headers['x-frame-options'] === 'SAMEORIGIN' ||
      (headers['content-security-policy'] && 
       headers['content-security-policy'].includes('frame-ancestors'));

    expect(hasFrameProtection).toBe(true);
  });

  test('validates secure cookie settings', async ({ page, context }) => {
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');
    await page.waitForURL('/dashboard');

    // Check cookies for security flags
    const cookies = await context.cookies();
    
    for (const cookie of cookies) {
      if (cookie.name.includes('auth') || cookie.name.includes('session')) {
        // Authentication cookies should be secure
        expect(cookie.secure).toBe(true);
        expect(cookie.httpOnly).toBe(true);
        expect(cookie.sameSite).toBe('Strict');
      }
    }
  });

  test('validates content security policy', async ({ page }) => {
    const response = await page.goto('/dashboard');
    const headers = response?.headers() || {};

    const csp = headers['content-security-policy'];
    expect(csp).toBeDefined();

    if (csp) {
      // Should restrict script sources
      expect(csp).toContain("script-src 'self'");
      
      // Should restrict object sources
      expect(csp).toContain("object-src 'none'");
      
      // Should have base-uri restriction
      expect(csp).toContain("base-uri 'self'");
    }
  });

  test('prevents sensitive data exposure in client-side storage', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');
    await page.waitForURL('/dashboard');

    // Check localStorage for sensitive data
    const localStorage = await page.evaluate(() => {
      const items = {};
      for (let i = 0; i < window.localStorage.length; i++) {
        const key = window.localStorage.key(i);
        if (key) {
          items[key] = window.localStorage.getItem(key);
        }
      }
      return items;
    });

    // Should not store passwords or private keys
    for (const [key, value] of Object.entries(localStorage)) {
      expect(value).not.toMatch(/password/i);
      expect(value).not.toMatch(/private.*key/i);
      expect(value).not.toMatch(/secret/i);
    }

    // Check sessionStorage
    const sessionStorage = await page.evaluate(() => {
      const items = {};
      for (let i = 0; i < window.sessionStorage.length; i++) {
        const key = window.sessionStorage.key(i);
        if (key) {
          items[key] = window.sessionStorage.getItem(key);
        }
      }
      return items;
    });

    for (const [key, value] of Object.entries(sessionStorage)) {
      expect(value).not.toMatch(/password/i);
      expect(value).not.toMatch(/private.*key/i);
      expect(value).not.toMatch(/secret/i);
    }
  });

  test('validates rate limiting on authentication', async ({ page }) => {
    await page.goto('/login');

    // Attempt multiple failed logins
    for (let i = 0; i < 5; i++) {
      await page.fill('[data-testid="email-input"]', 'test@example.com');
      await page.fill('[data-testid="password-input"]', 'wrongpassword');
      await page.click('[data-testid="login-button"]');
      
      // Wait a bit between attempts
      await page.waitForTimeout(100);
    }

    // Should show rate limiting message
    const errorMessage = await page.locator('[data-testid="error-message"]').textContent();
    expect(errorMessage).toMatch(/rate limit|too many attempts|blocked/i);
  });

  test('validates secure headers', async ({ page }) => {
    const response = await page.goto('/dashboard');
    const headers = response?.headers() || {};

    // Check for security headers
    expect(headers['x-content-type-options']).toBe('nosniff');
    expect(headers['x-xss-protection']).toBe('1; mode=block');
    expect(headers['referrer-policy']).toBeDefined();
    expect(headers['strict-transport-security']).toBeDefined();
  });
});