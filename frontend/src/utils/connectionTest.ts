import { api } from '@/services';
import { testAllConnections, getConnectionStatus } from '@/services/connection';

export interface ConnectionTestReport {
  timestamp: string;
  backend: {
    reachable: boolean;
    latency?: number;
    version?: string;
    endpoints: Record<string, boolean>;
  };
  websocket: {
    reachable: boolean;
    latency?: number;
  };
  integration: {
    status?: string;
    configured: boolean;
  };
  overall: 'healthy' | 'degraded' | 'failed';
  errors: string[];
}

/**
 * Run comprehensive connection tests
 */
export const runConnectionTests = async (token?: string): Promise<ConnectionTestReport> => {
  const timestamp = new Date().toISOString();
  const errors: string[] = [];
  
  // Test basic connections
  const connectionTest = await testAllConnections(token);
  
  // Test backend endpoints
  const endpointTests = await api.connection.testBackendEndpoints();
  
  // Get backend version
  let version: string | undefined;
  try {
    const versionResponse = await api.system.getVersion();
    if (versionResponse.success) {
      version = versionResponse.data?.version;
    }
  } catch (error: any) {
    errors.push(`Version check failed: ${error.message}`);
  }
  
  // Test integration status
  let integrationStatus: string | undefined;
  let integrationConfigured = false;
  try {
    const statusResponse = await api.system.getIntegrationStatus();
    if (statusResponse.success) {
      integrationStatus = statusResponse.data?.status;
      integrationConfigured = statusResponse.data?.contracts_configured || false;
    }
  } catch (error: any) {
    errors.push(`Integration status check failed: ${error.message}`);
  }
  
  // Determine overall health
  let overall: ConnectionTestReport['overall'] = 'failed';
  if (connectionTest.success && integrationConfigured) {
    overall = 'healthy';
  } else if (connectionTest.api || connectionTest.websocket) {
    overall = 'degraded';
  }
  
  // Add connection test errors
  errors.push(...connectionTest.errors);
  
  return {
    timestamp,
    backend: {
      reachable: connectionTest.api,
      latency: connectionTest.latency.api,
      version,
      endpoints: endpointTests,
    },
    websocket: {
      reachable: connectionTest.websocket,
      latency: connectionTest.latency.websocket,
    },
    integration: {
      status: integrationStatus,
      configured: integrationConfigured,
    },
    overall,
    errors,
  };
};

/**
 * Quick health check
 */
export const quickHealthCheck = async (): Promise<{
  healthy: boolean;
  message: string;
  details?: any;
}> => {
  try {
    const status = await getConnectionStatus();
    
    if (status.overall === 'connected') {
      return {
        healthy: true,
        message: 'All systems operational',
        details: status,
      };
    } else if (status.overall === 'degraded') {
      return {
        healthy: false,
        message: 'Some services are unavailable',
        details: status,
      };
    } else {
      return {
        healthy: false,
        message: 'Backend services are not reachable',
        details: status,
      };
    }
  } catch (error: any) {
    return {
      healthy: false,
      message: `Health check failed: ${error.message}`,
    };
  }
};

/**
 * Test authentication flow
 */
export const testAuthFlow = async (): Promise<{
  success: boolean;
  steps: Record<string, boolean>;
  errors: string[];
}> => {
  const steps: Record<string, boolean> = {};
  const errors: string[] = [];
  
  try {
    // Test login endpoint
    try {
      const loginResponse = await api.auth.login('test@example.com', 'password');
      steps.login = loginResponse.success;
      if (!loginResponse.success && loginResponse.error) {
        errors.push(`Login: ${loginResponse.error}`);
      }
    } catch (error: any) {
      steps.login = false;
      errors.push(`Login: ${error.message}`);
    }
    
    // Test current user endpoint
    try {
      const userResponse = await api.auth.getCurrentUser();
      steps.getCurrentUser = userResponse.success;
      if (!userResponse.success && userResponse.error) {
        errors.push(`Get User: ${userResponse.error}`);
      }
    } catch (error: any) {
      steps.getCurrentUser = false;
      errors.push(`Get User: ${error.message}`);
    }
    
    // Test logout endpoint
    try {
      const logoutResponse = await api.auth.logout();
      steps.logout = logoutResponse.success;
      if (!logoutResponse.success && logoutResponse.error) {
        errors.push(`Logout: ${logoutResponse.error}`);
      }
    } catch (error: any) {
      steps.logout = false;
      errors.push(`Logout: ${error.message}`);
    }
    
    const success = Object.values(steps).some(step => step); // At least one step should work
    
    return { success, steps, errors };
  } catch (error: any) {
    return {
      success: false,
      steps,
      errors: [`Auth flow test failed: ${error.message}`],
    };
  }
};

/**
 * Log connection test results
 */
export const logConnectionTest = async (token?: string): Promise<void> => {
  console.group('🔍 Frontend-Backend Connection Test');
  
  try {
    const report = await runConnectionTests(token);
    
    console.log('📊 Test Report:', {
      timestamp: report.timestamp,
      overall: report.overall,
    });
    
    console.log('🌐 Backend:', report.backend);
    console.log('🔌 WebSocket:', report.websocket);
    console.log('🔗 Integration:', report.integration);
    
    if (report.errors.length > 0) {
      console.warn('⚠️ Errors:', report.errors);
    }
    
    if (report.overall === 'healthy') {
      console.log('✅ All systems operational');
    } else if (report.overall === 'degraded') {
      console.warn('⚠️ Some services unavailable');
    } else {
      console.error('❌ Connection failed');
    }
  } catch (error) {
    console.error('❌ Connection test failed:', error);
  }
  
  console.groupEnd();
};

export default {
  runConnectionTests,
  quickHealthCheck,
  testAuthFlow,
  logConnectionTest,
};