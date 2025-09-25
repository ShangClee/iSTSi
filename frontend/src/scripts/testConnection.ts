#!/usr/bin/env node

/**
 * Frontend-Backend Connection Test Script
 * 
 * This script tests the connection between the frontend and Loco.rs backend
 * Run with: npm run test:connection
 */

import { logConnectionTest, runConnectionTests, quickHealthCheck } from '../utils/connectionTest';

async function main() {
  console.log('🚀 Starting Frontend-Backend Connection Test\n');
  
  try {
    // Quick health check first
    console.log('1️⃣ Running quick health check...');
    const healthCheck = await quickHealthCheck();
    
    if (healthCheck.healthy) {
      console.log('✅ Quick health check passed');
    } else {
      console.warn('⚠️ Quick health check failed:', healthCheck.message);
    }
    
    console.log('\n2️⃣ Running comprehensive connection tests...');
    
    // Run full connection test
    await logConnectionTest();
    
    console.log('\n3️⃣ Testing with detailed report...');
    
    // Get detailed report
    const report = await runConnectionTests();
    
    console.log('\n📋 Detailed Test Report:');
    console.log('========================');
    console.log(`Timestamp: ${report.timestamp}`);
    console.log(`Overall Status: ${report.overall}`);
    console.log('\nBackend:');
    console.log(`  - Reachable: ${report.backend.reachable}`);
    console.log(`  - Latency: ${report.backend.latency || 'N/A'}ms`);
    console.log(`  - Version: ${report.backend.version || 'Unknown'}`);
    console.log('\nWebSocket:');
    console.log(`  - Reachable: ${report.websocket.reachable}`);
    console.log(`  - Latency: ${report.websocket.latency || 'N/A'}ms`);
    console.log('\nIntegration:');
    console.log(`  - Status: ${report.integration.status || 'Unknown'}`);
    console.log(`  - Configured: ${report.integration.configured}`);
    
    if (report.errors.length > 0) {
      console.log('\n❌ Errors:');
      report.errors.forEach(error => console.log(`  - ${error}`));
    }
    
    console.log('\n🎯 Test Summary:');
    console.log('================');
    
    if (report.overall === 'healthy') {
      console.log('✅ All systems are operational!');
      console.log('🎉 Frontend can successfully communicate with the Loco.rs backend.');
    } else if (report.overall === 'degraded') {
      console.log('⚠️ Some services are unavailable.');
      console.log('🔧 Check the errors above and ensure the backend is running.');
    } else {
      console.log('❌ Connection failed.');
      console.log('🚨 Make sure the Loco.rs backend is running on http://localhost:8080');
    }
    
  } catch (error) {
    console.error('\n💥 Test script failed:', error);
    process.exit(1);
  }
}

// Run the test if this script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}

export default main;