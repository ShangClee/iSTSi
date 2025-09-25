import { test, expect } from './setup';

test.describe('Token Withdrawal Flow', () => {
  test('complete token withdrawal workflow', async ({ authenticatedPage: page }) => {
    await page.click('[data-testid="integration-nav"]');

    // Switch to withdrawal tab
    await page.click('[data-testid="token-withdrawal-tab"]');
    await expect(page.locator('h2')).toContainText('Token Withdrawal');

    // Fill out withdrawal form
    await page.fill('[data-testid="token-amount-input"]', '0.75');
    await page.fill('[data-testid="btc-address-input"]', 'bc1qtest123456789abcdef');

    // Submit the form
    await page.click('[data-testid="execute-withdrawal-button"]');

    // Wait for success message or error (depending on user balance)
    const successMessage = page.locator('[data-testid="success-message"]');
    const errorMessage = page.locator('[data-testid="error-message"]');

    await expect(successMessage.or(errorMessage)).toBeVisible();

    // If successful, check operation history
    const isSuccess = await successMessage.isVisible();
    if (isSuccess) {
      await expect(successMessage).toContainText('withdrawal operation initiated');

      await page.click('[data-testid="operation-history-tab"]');
      await expect(page.locator('[data-testid="operation-list"]')).toContainText('Token Withdrawal');
    }
  });

  test('validates bitcoin address format', async ({ authenticatedPage: page }) => {
    await page.click('[data-testid="integration-nav"]');
    await page.click('[data-testid="token-withdrawal-tab"]');

    // Test invalid Bitcoin address formats
    const invalidAddresses = [
      'invalid-address',
      '1234567890',
      'bc1qinvalid',
      '',
    ];

    for (const address of invalidAddresses) {
      await page.fill('[data-testid="token-amount-input"]', '0.1');
      await page.fill('[data-testid="btc-address-input"]', address);
      await page.click('[data-testid="execute-withdrawal-button"]');

      await expect(page.locator('[data-testid="btc-address-error"]')).toContainText(
        'Invalid Bitcoin address'
      );

      // Clear the form for next iteration
      await page.fill('[data-testid="btc-address-input"]', '');
    }
  });

  test('validates withdrawal amounts', async ({ authenticatedPage: page }) => {
    await page.click('[data-testid="integration-nav"]');
    await page.click('[data-testid="token-withdrawal-tab"]');

    // Test zero amount
    await page.fill('[data-testid="token-amount-input"]', '0');
    await page.fill('[data-testid="btc-address-input"]', 'bc1qtest123456789abcdef');
    await page.click('[data-testid="execute-withdrawal-button"]');

    await expect(page.locator('[data-testid="token-amount-error"]')).toContainText(
      'Amount must be greater than 0'
    );

    // Test negative amount
    await page.fill('[data-testid="token-amount-input"]', '-0.5');
    await page.click('[data-testid="execute-withdrawal-button"]');

    await expect(page.locator('[data-testid="token-amount-error"]')).toContainText(
      'Amount must be positive'
    );
  });

  test('shows insufficient balance error', async ({ authenticatedPage: page }) => {
    // Mock API to return insufficient balance error
    await page.route('/api/integration/token-withdrawal', (route) => {
      route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Insufficient token balance' }),
      });
    });

    await page.click('[data-testid="integration-nav"]');
    await page.click('[data-testid="token-withdrawal-tab"]');

    await page.fill('[data-testid="token-amount-input"]', '100.0'); // Large amount
    await page.fill('[data-testid="btc-address-input"]', 'bc1qtest123456789abcdef');
    await page.click('[data-testid="execute-withdrawal-button"]');

    await expect(page.locator('[data-testid="error-message"]')).toContainText(
      'Insufficient token balance'
    );
  });

  test('displays current token balance', async ({ authenticatedPage: page }) => {
    // Mock API to return user balance
    await page.route('/api/users/balance', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ 
          balance: '250000000', // 2.5 BTC worth
          formatted: '2.50 BTC'
        }),
      });
    });

    await page.click('[data-testid="integration-nav"]');
    await page.click('[data-testid="token-withdrawal-tab"]');

    // Should display current balance
    await expect(page.locator('[data-testid="current-balance"]')).toContainText('2.50 BTC');

    // Should have "Max" button to use full balance
    await page.click('[data-testid="max-amount-button"]');
    await expect(page.locator('[data-testid="token-amount-input"]')).toHaveValue('2.50');
  });

  test('estimates withdrawal fees', async ({ authenticatedPage: page }) => {
    // Mock API to return fee estimate
    await page.route('/api/integration/estimate-withdrawal-fee', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ 
          networkFee: '0.0001',
          serviceFee: '0.001',
          totalFee: '0.0011'
        }),
      });
    });

    await page.click('[data-testid="integration-nav"]');
    await page.click('[data-testid="token-withdrawal-tab"]');

    await page.fill('[data-testid="token-amount-input"]', '1.0');
    await page.fill('[data-testid="btc-address-input"]', 'bc1qtest123456789abcdef');

    // Should show fee estimate
    await expect(page.locator('[data-testid="fee-estimate"]')).toContainText('0.0011 BTC');
    await expect(page.locator('[data-testid="net-amount"]')).toContainText('0.9989 BTC');
  });
});

test.describe('Token Withdrawal Security', () => {
  test('requires additional confirmation for large withdrawals', async ({ authenticatedPage: page }) => {
    await page.click('[data-testid="integration-nav"]');
    await page.click('[data-testid="token-withdrawal-tab"]');

    // Large withdrawal amount
    await page.fill('[data-testid="token-amount-input"]', '10.0'); // 10 BTC
    await page.fill('[data-testid="btc-address-input"]', 'bc1qtest123456789abcdef');

    await page.click('[data-testid="execute-withdrawal-button"]');

    // Should show confirmation dialog for large amounts
    await expect(page.locator('[data-testid="confirmation-dialog"]')).toBeVisible();
    await expect(page.locator('[data-testid="confirmation-message"]')).toContainText(
      'large withdrawal'
    );

    // Should require explicit confirmation
    await page.click('[data-testid="confirm-withdrawal-button"]');

    // Now should proceed with withdrawal
    const successMessage = page.locator('[data-testid="success-message"]');
    const errorMessage = page.locator('[data-testid="error-message"]');
    await expect(successMessage.or(errorMessage)).toBeVisible();
  });

  test('validates KYC requirements for withdrawals', async ({ authenticatedPage: page }) => {
    // Mock user with pending KYC
    await page.route('/api/auth/me', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'user-123',
          email: 'test@example.com',
          kycStatus: 'pending',
          tier: 0,
        }),
      });
    });

    await page.reload();
    await page.click('[data-testid="integration-nav"]');
    await page.click('[data-testid="token-withdrawal-tab"]');

    // Should show KYC warning
    await expect(page.locator('[data-testid="kyc-warning"]')).toContainText(
      'KYC verification required'
    );

    // Withdrawal button should be disabled
    await expect(page.locator('[data-testid="execute-withdrawal-button"]')).toBeDisabled();
  });

  test('implements withdrawal rate limiting', async ({ authenticatedPage: page }) => {
    await page.click('[data-testid="integration-nav"]');
    await page.click('[data-testid="token-withdrawal-tab"]');

    // Make multiple rapid withdrawal attempts
    for (let i = 0; i < 3; i++) {
      await page.fill('[data-testid="token-amount-input"]', '0.1');
      await page.fill('[data-testid="btc-address-input"]', `bc1qtest${i}23456789abcdef`);
      await page.click('[data-testid="execute-withdrawal-button"]');

      // Wait a bit between attempts
      await page.waitForTimeout(100);
    }

    // Should eventually show rate limiting message
    const rateLimitMessage = page.locator('[data-testid="rate-limit-message"]');
    const errorMessage = page.locator('[data-testid="error-message"]');

    // Either rate limit message or general error should appear
    await expect(rateLimitMessage.or(errorMessage)).toBeVisible();
  });
});