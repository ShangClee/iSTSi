import { connectionUtils } from './api';
import { getWebSocketStatus, testWebSocketConnection } from './websocket';

export interface ConnectionStatus {
  api: {
    connected: boolean;
    latency?: number;
    baseUrl: string;
    version?: string;
  };
  websocket: {
    connected: boolean;
    url: string;
    reconnectAttempts: number;
  };
  overall: 'connected' | 'degraded' | 'disconnected';
}

export interface ConnectionTestResult {
  success: boolean;
  api: boolean;
  websocket: boolean;
  errors: string[];
  latency: {
    api?: number;
    websocket?: number;
  };
}

/**
 * Get current connection status for all services
 */
export const getConnectionStatus = async (): Promise<ConnectionStatus> => {
  // Test API connection
  const apiInfo = await connectionUtils.getConnectionInfo();
  
  // Get WebSocket status
  const wsStatus = getWebSocketStatus();
  
  // Determine overall status
  let overall: ConnectionStatus['overall'] = 'disconnected';
  if (apiInfo.connected && wsStatus.connected) {
    overall = 'connected';
  } else if (apiInfo.connected || wsStatus.connected) {
    overall = 'degraded';
  }
  
  return {
    api: {
      connected: apiInfo.connected,
      latency: apiInfo.latency,
      baseUrl: apiInfo.baseUrl,
      version: apiInfo.version,
    },
    websocket: {
      connected: wsStatus.connected,
      url: wsStatus.url,
      reconnectAttempts: wsStatus.reconnectAttempts || 0,
    },
    overall,
  };
};

/**
 * Test all connections and return detailed results
 */
export const testAllConnections = async (token?: string): Promise<ConnectionTestResult> => {
  const errors: string[] = [];
  const latency: ConnectionTestResult['latency'] = {};
  
  // Test API connection
  const apiTest = await connectionUtils.testConnection();
  const apiSuccess = apiTest.success;
  if (!apiSuccess && apiTest.error) {
    errors.push(`API: ${apiTest.error}`);
  } else if (apiTest.latency) {
    latency.api = apiTest.latency;
  }
  
  // Test WebSocket connection
  const wsTest = await testWebSocketConnection(token);
  const wsSuccess = wsTest.success;
  if (!wsSuccess && wsTest.error) {
    errors.push(`WebSocket: ${wsTest.error}`);
  } else if (wsTest.latency) {
    latency.websocket = wsTest.latency;
  }
  
  return {
    success: apiSuccess && wsSuccess,
    api: apiSuccess,
    websocket: wsSuccess,
    errors,
    latency,
  };
};

/**
 * Monitor connection status with periodic checks
 */
export class ConnectionMonitor {
  private interval: NodeJS.Timeout | null = null;
  private listeners: Array<(status: ConnectionStatus) => void> = [];
  private lastStatus: ConnectionStatus | null = null;
  
  constructor(private checkInterval: number = 30000) {}
  
  /**
   * Start monitoring connections
   */
  start(): void {
    if (this.interval) {
      this.stop();
    }
    
    // Initial check
    this.checkStatus();
    
    // Periodic checks
    this.interval = setInterval(() => {
      this.checkStatus();
    }, this.checkInterval);
  }
  
  /**
   * Stop monitoring
   */
  stop(): void {
    if (this.interval) {
      clearInterval(this.interval);
      this.interval = null;
    }
  }
  
  /**
   * Subscribe to status changes
   */
  subscribe(listener: (status: ConnectionStatus) => void): () => void {
    this.listeners.push(listener);
    
    // Send current status if available
    if (this.lastStatus) {
      listener(this.lastStatus);
    }
    
    return () => {
      const index = this.listeners.indexOf(listener);
      if (index > -1) {
        this.listeners.splice(index, 1);
      }
    };
  }
  
  /**
   * Get last known status
   */
  getLastStatus(): ConnectionStatus | null {
    return this.lastStatus;
  }
  
  /**
   * Force a status check
   */
  async checkStatus(): Promise<ConnectionStatus> {
    try {
      const status = await getConnectionStatus();
      
      // Check if status changed
      const statusChanged = !this.lastStatus || 
        this.lastStatus.overall !== status.overall ||
        this.lastStatus.api.connected !== status.api.connected ||
        this.lastStatus.websocket.connected !== status.websocket.connected;
      
      this.lastStatus = status;
      
      // Notify listeners if status changed
      if (statusChanged) {
        this.listeners.forEach(listener => {
          try {
            listener(status);
          } catch (error) {
            console.error('Error in connection status listener:', error);
          }
        });
      }
      
      return status;
    } catch (error) {
      console.error('Failed to check connection status:', error);
      
      // Return disconnected status on error
      const errorStatus: ConnectionStatus = {
        api: {
          connected: false,
          baseUrl: '',
        },
        websocket: {
          connected: false,
          url: '',
          reconnectAttempts: 0,
        },
        overall: 'disconnected',
      };
      
      this.lastStatus = errorStatus;
      return errorStatus;
    }
  }
}

// Create singleton monitor instance
let connectionMonitor: ConnectionMonitor | null = null;

/**
 * Get or create the connection monitor instance
 */
export const getConnectionMonitor = (): ConnectionMonitor => {
  if (!connectionMonitor) {
    connectionMonitor = new ConnectionMonitor();
  }
  return connectionMonitor;
};

/**
 * Start connection monitoring
 */
export const startConnectionMonitoring = (): ConnectionMonitor => {
  const monitor = getConnectionMonitor();
  monitor.start();
  return monitor;
};

/**
 * Stop connection monitoring
 */
export const stopConnectionMonitoring = (): void => {
  if (connectionMonitor) {
    connectionMonitor.stop();
  }
};

export default {
  getConnectionStatus,
  testAllConnections,
  ConnectionMonitor,
  getConnectionMonitor,
  startConnectionMonitoring,
  stopConnectionMonitoring,
};