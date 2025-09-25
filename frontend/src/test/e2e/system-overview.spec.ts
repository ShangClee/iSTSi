import { test, expect } from './setup';

test.describe('System Overview Dashboard', () => {
  test('displays system metrics correctly', async ({ authenticatedPage: page }) => {
    // Mock system overview API
    await page.route('/api/system/overview', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          totalReserves: '1000000000', // 10 BTC
          totalTokensIssued: '950000000', // 9.5 BTC worth
          reserveRatio: 105.26,
          activeUsers: 150,
          totalTransactions: 1250,
          systemStatus: 'operational',
          lastUpdated: '2024-01-01T12:00:00Z',
        }),
      });
    });

    // Navigate to dashboard (should be default after login)
    await expect(page.locator('[data-testid="system-overview"]')).toBeVisible();

    // Check that all metrics are displayed
    await expect(page.locator('[data-testid="total-reserves"]')).toContainText('10.00 BTC');
    await expect(page.locator('[data-testid="total-tokens"]')).toContainText('9.50 BTC');
    await expect(page.locator('[data-testid="reserve-ratio"]')).toContainText('105.26%');
    await expect(page.locator('[data-testid="active-users"]')).toContainText('150');
    await expect(page.locator('[data-testid="total-transactions"]')).toContainText('1,250');

    // Check system status indicator
    await expect(page.locator('[data-testid="system-status"]')).toContainText('Operational');
    await expect(page.locator('[data-testid="status-indicator"]')).toHaveClass(/green/);
  });

  test('shows warning for low reserve ratio', async ({ authenticatedPage: page }) => {
    // Mock system with low reserve ratio
    await page.route('/api/system/overview', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          totalReserves: '950000000', // 9.5 BTC
          totalTokensIssued: '1000000000', // 10 BTC worth
          reserveRatio: 95.0, // Below 100%
          activeUsers: 150,
          totalTransactions: 1250,
          systemStatus: 'warning',
          lastUpdated: '2024-01-01T12:00:00Z',
        }),
      });
    });

    await page.reload();

    // Should show warning indicator
    await expect(page.locator('[data-testid="reserve-ratio"]')).toContainText('95.00%');
    await expect(page.locator('[data-testid="reserve-warning"]')).toBeVisible();
    await expect(page.locator('[data-testid="reserve-warning"]')).toContainText(
      'Reserve ratio below 100%'
    );

    // System status should reflect warning
    await expect(page.locator('[data-testid="system-status"]')).toContainText('Warning');
    await expect(page.locator('[data-testid="status-indicator"]')).toHaveClass(/yellow|orange/);
  });

  test('refreshes data automatically', async ({ authenticatedPage: page }) => {
    let requestCount = 0;
    
    await page.route('/api/system/overview', (route) => {
      requestCount++;
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          totalReserves: `${1000000000 + requestCount * 10000000}`, // Incrementing reserves
          totalTokensIssued: '950000000',
          reserveRatio: 105.26 + requestCount,
          activeUsers: 150 + requestCount,
          totalTransactions: 1250 + requestCount * 10,
          systemStatus: 'operational',
          lastUpdated: new Date().toISOString(),
        }),
      });
    });

    // Initial load
    await expect(page.locator('[data-testid="total-reserves"]')).toContainText('10.10 BTC');

    // Wait for auto-refresh (assuming 30 second interval)
    await page.waitForTimeout(31000);

    // Should have updated values
    await expect(page.locator('[data-testid="total-reserves"]')).toContainText('10.20 BTC');
    await expect(page.locator('[data-testid="active-users"]')).toContainText('152');
  });

  test('handles manual refresh', async ({ authenticatedPage: page }) => {
    let refreshCount = 0;
    
    await page.route('/api/system/overview', (route) => {
      refreshCount++;
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          totalReserves: '1000000000',
          totalTokensIssued: '950000000',
          reserveRatio: 105.26,
          activeUsers: 150,
          totalTransactions: 1250 + refreshCount * 100, // Changes with each refresh
          systemStatus: 'operational',
          lastUpdated: new Date().toISOString(),
        }),
      });
    });

    // Initial load (refreshCount = 1)
    await expect(page.locator('[data-testid="total-transactions"]')).toContainText('1,350');

    // Click refresh button
    await page.click('[data-testid="refresh-button"]');

    // Should show updated data (refreshCount = 2)
    await expect(page.locator('[data-testid="total-transactions"]')).toContainText('1,450');

    // Should show loading indicator during refresh
    await page.click('[data-testid="refresh-button"]');
    await expect(page.locator('[data-testid="loading-indicator"]')).toBeVisible();
  });

  test('displays real-time updates via WebSocket', async ({ authenticatedPage: page }) => {
    // Initial data
    await expect(page.locator('[data-testid="total-reserves"]')).toContainText('10.00 BTC');

    // Simulate WebSocket update
    await page.evaluate(() => {
      window.dispatchEvent(
        new CustomEvent('websocket-message', {
          detail: {
            type: 'system_update',
            data: {
              totalReserves: '1100000000', // 11 BTC
              reserveRatio: 115.79,
              activeUsers: 155,
            },
          },
        })
      );
    });

    // Should update without page refresh
    await expect(page.locator('[data-testid="total-reserves"]')).toContainText('11.00 BTC');
    await expect(page.locator('[data-testid="reserve-ratio"]')).toContainText('115.79%');
    await expect(page.locator('[data-testid="active-users"]')).toContainText('155');
  });

  test('handles API errors gracefully', async ({ authenticatedPage: page }) => {
    // Mock API error
    await page.route('/api/system/overview', (route) => {
      route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Internal server error' }),
      });
    });

    await page.reload();

    // Should show error message
    await expect(page.locator('[data-testid="error-message"]')).toContainText(
      'Error loading system data'
    );

    // Should have retry button
    await expect(page.locator('[data-testid="retry-button"]')).toBeVisible();

    // Mock successful response for retry
    await page.route('/api/system/overview', (route) => {
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

    // Click retry
    await page.click('[data-testid="retry-button"]');

    // Should load data successfully
    await expect(page.locator('[data-testid="total-reserves"]')).toContainText('10.00 BTC');
    await expect(page.locator('[data-testid="error-message"]')).not.toBeVisible();
  });
});

test.describe('System Overview Charts and Visualizations', () => {
  test('displays reserve ratio chart', async ({ authenticatedPage: page }) => {
    // Mock historical data API
    await page.route('/api/system/reserve-history', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          data: [
            { timestamp: '2024-01-01T00:00:00Z', ratio: 105.0 },
            { timestamp: '2024-01-01T06:00:00Z', ratio: 103.5 },
            { timestamp: '2024-01-01T12:00:00Z', ratio: 105.26 },
          ],
        }),
      });
    });

    // Navigate to charts section
    await page.click('[data-testid="charts-tab"]');

    // Should display reserve ratio chart
    await expect(page.locator('[data-testid="reserve-chart"]')).toBeVisible();
    await expect(page.locator('[data-testid="chart-title"]')).toContainText('Reserve Ratio');

    // Should show data points
    await expect(page.locator('[data-testid="chart-data-point"]')).toHaveCount(3);
  });

  test('displays transaction volume chart', async ({ authenticatedPage: page }) => {
    // Mock transaction volume data
    await page.route('/api/system/transaction-volume', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          daily: [
            { date: '2024-01-01', deposits: 5, withdrawals: 2, volume: '750000000' },
            { date: '2024-01-02', deposits: 8, withdrawals: 3, volume: '1200000000' },
          ],
        }),
      });
    });

    await page.click('[data-testid="charts-tab"]');
    await page.click('[data-testid="volume-chart-tab"]');

    // Should display transaction volume chart
    await expect(page.locator('[data-testid="volume-chart"]')).toBeVisible();
    await expect(page.locator('[data-testid="chart-legend"]')).toContainText('Deposits');
    await expect(page.locator('[data-testid="chart-legend"]')).toContainText('Withdrawals');
  });

  test('allows chart time range selection', async ({ authenticatedPage: page }) => {
    await page.click('[data-testid="charts-tab"]');

    // Should have time range selector
    await expect(page.locator('[data-testid="time-range-selector"]')).toBeVisible();

    // Test different time ranges
    const timeRanges = ['24h', '7d', '30d', '90d'];
    
    for (const range of timeRanges) {
      await page.click(`[data-testid="time-range-${range}"]`);
      
      // Should update chart data (mock would need to handle different ranges)
      await expect(page.locator('[data-testid="chart-time-label"]')).toContainText(range);
    }
  });
});