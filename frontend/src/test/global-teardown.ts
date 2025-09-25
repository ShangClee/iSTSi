import { chromium, FullConfig } from '@playwright/test';

async function globalTeardown(config: FullConfig) {
  console.log('🧹 Starting global test teardown...');

  const browser = await chromium.launch();
  const page = await browser.newPage();

  try {
    // Cleanup test data
    console.log('🗑️ Cleaning up test data...');
    
    await cleanupTestData(page);
    
    console.log('✅ Global teardown completed successfully');
  } catch (error) {
    console.error('❌ Global teardown failed:', error);
    // Don't throw here as it might mask test failures
  } finally {
    await browser.close();
  }
}

async function cleanupTestData(page: any) {
  // Clean up test users, operations, and other test data
  // This ensures tests don't interfere with each other
  
  console.log('🧽 Removing test users and data...');
  
  // Example cleanup (commented out):
  /*
  await page.request.delete('/api/test/cleanup', {
    data: {
      cleanupTypes: ['users', 'operations', 'kyc_records'],
      testPrefix: 'test-',
    },
  });
  */
  
  // Reset any global state
  console.log('🔄 Resetting global state...');
}

export default globalTeardown;