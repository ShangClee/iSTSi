import { test, expect } from './setup';

test.describe('Bitcoin Deposit Flow', () => {
  test('complete bitcoin deposit workflow', async ({ authenticatedPage: page }) => {
    // Navigate to integration router
    await page.click('[data-testid="integration-nav"]');
    await expect(page.locator('h1')).toContainText('Integration Router');

    // Fill out Bitcoin deposit form
    await page.fill('[data-testid="btc-amount-input"]', '1.5');
    await page.fill(
      '[data-testid="btc-tx-hash-input"]',
      'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
    );
    await page.fill('[data-testid="confirmations-input"]', '6');

    // Submit the form
    await page.click('[data-testid="execute-deposit-button"]');

    // Wait for success message
    await expect(page.locator('[data-testid="success-message"]')).toContainText(
      'deposit operation initiated'
    );

    // Check that operation appears in history
    await page.click('[data-testid="operation-history-tab"]');
    await expect(page.locator('[data-testid="operation-list"]')).toContainText('Bitcoin Deposit');

    // Verify operation details
    const operationRow = page.locator('[data-testid="operation-row"]').first();
    await expect(operationRow).toContainText('1.50 BTC');
    await expect(operationRow).toContainText('Pending');
  });

  test('validates bitcoin deposit form inputs', async ({ authenticatedPage: page }) => {
    await page.click('[data-testid="integration-nav"]');

    // Try to submit empty form
    await page.click('[data-testid="execute-deposit-button"]');

    // Check validation errors
    await expect(page.locator('[data-testid="btc-amount-error"]')).toContainText(
      'Bitcoin amount is required'
    );
    await expect(page.locator('[data-testid="btc-tx-hash-error"]')).toContainText(
      'Transaction hash is required'
    );

    // Test invalid amount
    await page.fill('[data-testid="btc-amount-input"]', '-1');
    await page.click('[data-testid="execute-deposit-button"]');
    await expect(page.locator('[data-testid="btc-amount-error"]')).toContainText(
      'Amount must be positive'
    );

    // Test insufficient confirmations
    await page.fill('[data-testid="btc-amount-input"]', '1.0');
    await page.fill('[data-testid="confirmations-input"]', '2');
    await page.click('[data-testid="execute-deposit-button"]');
    await expect(page.locator('[data-testid="confirmations-error"]')).toContainText(
      'At least 3 confirmations required'
    );
  });

  test('handles API errors gracefully', async ({ authenticatedPage: page }) => {
    // Mock API to return error
    await page.route('/api/integration/bitcoin-deposit', (route) => {
      route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Insufficient confirmations' }),
      });
    });

    await page.click('[data-testid="integration-nav"]');

    // Fill valid form data
    await page.fill('[data-testid="btc-amount-input"]', '1.0');
    await page.fill(
      '[data-testid="btc-tx-hash-input"]',
      'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
    );
    await page.fill('[data-testid="confirmations-input"]', '6');

    await page.click('[data-testid="execute-deposit-button"]');

    // Check error message is displayed
    await expect(page.locator('[data-testid="error-message"]')).toContainText(
      'Insufficient confirmations'
    );
  });

  test('shows real-time operation status updates', async ({ authenticatedPage: page }) => {
    await page.click('[data-testid="integration-nav"]');

    // Submit a deposit
    await page.fill('[data-testid="btc-amount-input"]', '0.5');
    await page.fill(
      '[data-testid="btc-tx-hash-input"]',
      'realtime1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
    );
    await page.fill('[data-testid="confirmations-input"]', '6');
    await page.click('[data-testid="execute-deposit-button"]');

    // Wait for operation to appear
    await page.click('[data-testid="operation-history-tab"]');
    const operationRow = page.locator('[data-testid="operation-row"]').first();
    await expect(operationRow).toContainText('Pending');

    // Mock WebSocket update
    await page.evaluate(() => {
      // Simulate WebSocket message
      window.dispatchEvent(
        new CustomEvent('websocket-message', {
          detail: {
            type: 'operation_update',
            data: {
              operationId: 'test-op-id',
              status: 'completed',
            },
          },
        })
      );
    });

    // Check that status updates
    await expect(operationRow).toContainText('Completed', { timeout: 5000 });
  });
});

test.describe('Bitcoin Deposit Edge Cases', () => {
  test('handles network connectivity issues', async ({ authenticatedPage: page }) => {
    await page.click('[data-testid="integration-nav"]');

    // Simulate network failure
    await page.route('/api/integration/bitcoin-deposit', (route) => {
      route.abort('failed');
    });

    await page.fill('[data-testid="btc-amount-input"]', '1.0');
    await page.fill(
      '[data-testid="btc-tx-hash-input"]',
      'network1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
    );
    await page.fill('[data-testid="confirmations-input"]', '6');

    await page.click('[data-testid="execute-deposit-button"]');

    // Should show network error
    await expect(page.locator('[data-testid="error-message"]')).toContainText(
      'Network error'
    );

    // Should allow retry
    await expect(page.locator('[data-testid="retry-button"]')).toBeVisible();
  });

  test('prevents duplicate transaction submissions', async ({ authenticatedPage: page }) => {
    await page.click('[data-testid="integration-nav"]');

    const txHash = 'duplicate1234567890abcdef1234567890abcdef1234567890abcdef1234567890';

    // Submit first transaction
    await page.fill('[data-testid="btc-amount-input"]', '1.0');
    await page.fill('[data-testid="btc-tx-hash-input"]', txHash);
    await page.fill('[data-testid="confirmations-input"]', '6');
    await page.click('[data-testid="execute-deposit-button"]');

    await expect(page.locator('[data-testid="success-message"]')).toBeVisible();

    // Try to submit same transaction again
    await page.fill('[data-testid="btc-amount-input"]', '1.0');
    await page.fill('[data-testid="btc-tx-hash-input"]', txHash);
    await page.fill('[data-testid="confirmations-input"]', '6');
    await page.click('[data-testid="execute-deposit-button"]');

    // Should show duplicate error
    await expect(page.locator('[data-testid="error-message"]')).toContainText(
      'Transaction already processed'
    );
  });

  test('handles large amounts correctly', async ({ authenticatedPage: page }) => {
    await page.click('[data-testid="integration-nav"]');

    // Test with large amount
    await page.fill('[data-testid="btc-amount-input"]', '100.0'); // 100 BTC
    await page.fill(
      '[data-testid="btc-tx-hash-input"]',
      'large1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
    );
    await page.fill('[data-testid="confirmations-input"]', '6');

    await page.click('[data-testid="execute-deposit-button"]');

    // Should either succeed or show appropriate limit warning
    const successMessage = page.locator('[data-testid="success-message"]');
    const warningMessage = page.locator('[data-testid="warning-message"]');

    await expect(successMessage.or(warningMessage)).toBeVisible();

    // If warning, should mention tier limits
    const isWarning = await warningMessage.isVisible();
    if (isWarning) {
      await expect(warningMessage).toContainText('tier limit');
    }
  });
});