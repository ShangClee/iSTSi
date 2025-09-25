import { test, expect } from '@playwright/test';

test.describe('Performance Tests', () => {
  test('dashboard loads within acceptable time', async ({ page }) => {
    // Start timing
    const startTime = Date.now();

    // Navigate to login and authenticate
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');

    // Wait for dashboard to load
    await page.waitForURL('/dashboard');
    await page.waitForLoadState('networkidle');

    const loadTime = Date.now() - startTime;

    // Dashboard should load within 3 seconds
    expect(loadTime).toBeLessThan(3000);

    // Check that critical elements are visible
    await expect(page.locator('[data-testid="system-overview"]')).toBeVisible();
    await expect(page.locator('[data-testid="total-reserves"]')).toBeVisible();
  });

  test('API responses are fast enough', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');
    await page.waitForURL('/dashboard');

    // Test system overview API performance
    const apiStartTime = Date.now();
    
    await page.route('/api/system/overview', (route) => {
      const responseTime = Date.now() - apiStartTime;
      
      // API should respond within 500ms
      expect(responseTime).toBeLessThan(500);
      
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          totalReserves: '1000000000',
          totalTokensIssued: '950000000',
          reserveRatio: 105.26,
          activeUsers: 150,
          totalTransactions: 1250,
          systemStatus: 'operational',
          lastUpdated: '2024-01-01T12:00:00Z',
        }),
      });
    });

    // Trigger API call
    await page.click('[data-testid="refresh-button"]');
    await page.waitForResponse('/api/system/overview');
  });

  test('handles multiple concurrent operations', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');
    await page.waitForURL('/dashboard');

    await page.goto('/integration');

    // Simulate multiple rapid form submissions
    const promises = [];
    
    for (let i = 0; i < 5; i++) {
      const promise = (async () => {
        await page.fill('[data-testid="btc-amount-input"]', '0.1');
        await page.fill('[data-testid="btc-tx-hash-input"]', 
          `concurrent${i}234567890abcdef1234567890abcdef1234567890abcdef1234567890`);
        await page.fill('[data-testid="confirmations-input"]', '6');
        await page.click('[data-testid="execute-deposit-button"]');
      })();
      
      promises.push(promise);
    }

    // All operations should complete without errors
    await Promise.all(promises);

    // UI should remain responsive
    await expect(page.locator('[data-testid="integration-router"]')).toBeVisible();
  });

  test('memory usage remains stable', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');
    await page.waitForURL('/dashboard');

    // Get initial memory usage
    const initialMemory = await page.evaluate(() => {
      return (performance as any).memory?.usedJSHeapSize || 0;
    });

    // Perform memory-intensive operations
    for (let i = 0; i < 10; i++) {
      // Navigate between pages
      await page.click('[data-testid="integration-nav"]');
      await page.click('[data-testid="dashboard-nav"]');
      
      // Trigger data refreshes
      await page.click('[data-testid="refresh-button"]');
      await page.waitForTimeout(100);
    }

    // Check final memory usage
    const finalMemory = await page.evaluate(() => {
      return (performance as any).memory?.usedJSHeapSize || 0;
    });

    // Memory growth should be reasonable (less than 50MB increase)
    const memoryGrowth = finalMemory - initialMemory;
    expect(memoryGrowth).toBeLessThan(50 * 1024 * 1024); // 50MB
  });

  test('WebSocket connection performance', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');
    await page.waitForURL('/dashboard');

    // Monitor WebSocket connection time
    const wsStartTime = Date.now();
    
    await page.evaluate(() => {
      // Simulate WebSocket connection
      const ws = new WebSocket('ws://localhost:8080/ws');
      
      ws.onopen = () => {
        const connectionTime = Date.now() - window.wsStartTime;
        window.wsConnectionTime = connectionTime;
      };
      
      window.wsStartTime = Date.now();
    });

    // Wait for connection
    await page.waitForFunction(() => window.wsConnectionTime !== undefined);

    const connectionTime = await page.evaluate(() => window.wsConnectionTime);
    
    // WebSocket should connect within 1 second
    expect(connectionTime).toBeLessThan(1000);
  });

  test('large data set rendering performance', async ({ page }) => {
    // Mock API with large operation history
    await page.route('/api/users/operations', (route) => {
      const operations = [];
      for (let i = 0; i < 1000; i++) {
        operations.push({
          id: `op-${i}`,
          type: i % 2 === 0 ? 'bitcoin_deposit' : 'token_withdrawal',
          status: 'completed',
          amount: '100000000',
          createdAt: new Date(Date.now() - i * 60000).toISOString(),
        });
      }
      
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(operations),
      });
    });

    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');
    await page.waitForURL('/dashboard');

    await page.click('[data-testid="integration-nav"]');
    await page.click('[data-testid="operation-history-tab"]');

    const renderStartTime = Date.now();
    
    // Wait for large list to render
    await page.waitForSelector('[data-testid="operation-list"]');
    await page.waitForFunction(() => 
      document.querySelectorAll('[data-testid="operation-row"]').length > 0
    );

    const renderTime = Date.now() - renderStartTime;

    // Large list should render within 2 seconds
    expect(renderTime).toBeLessThan(2000);

    // Should implement virtualization for large lists
    const visibleRows = await page.locator('[data-testid="operation-row"]').count();
    
    // Should not render all 1000 rows at once (virtualization)
    expect(visibleRows).toBeLessThan(100);
  });
});