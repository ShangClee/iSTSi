import type { WebSocketMessage } from '@/types';

// WebSocket Configuration for Loco.rs backend
const WS_URL = import.meta.env.VITE_WS_URL || 'ws://localhost:8080';
const WS_RECONNECT_INTERVAL = 5000;
const WS_MAX_RECONNECT_ATTEMPTS = 10;

export interface WebSocketEventHandlers {
  onConnect?: () => void;
  onDisconnect?: (reason: string) => void;
  onError?: (error: Event) => void;
  onSystemUpdate?: (data: any) => void;
  onOperationUpdate?: (data: any) => void;
  onAlert?: (data: any) => void;
  onReserveUpdate?: (data: any) => void;
  onComplianceUpdate?: (data: any) => void;
  onMessage?: (message: WebSocketMessage) => void;
}

export class WebSocketClient {
  private socket: WebSocket | null = null;
  private handlers: WebSocketEventHandlers = {};
  private reconnectAttempts = 0;
  private maxReconnectAttempts = WS_MAX_RECONNECT_ATTEMPTS;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private isConnecting = false;
  private isManuallyDisconnected = false;
  private subscriptions: Set<string> = new Set();
  private heartbeatInterval: NodeJS.Timeout | null = null;

  constructor(handlers: WebSocketEventHandlers = {}) {
    this.handlers = handlers;
  }

  /**
   * Connect to the WebSocket server
   */
  connect(token?: string): void {
    if ((this.socket?.readyState === WebSocket.OPEN) || this.isConnecting) {
      return;
    }

    this.isConnecting = true;
    this.isManuallyDisconnected = false;

    try {
      // Build WebSocket URL with authentication
      let wsUrl = WS_URL;
      if (token) {
        const url = new URL(wsUrl);
        url.searchParams.set('token', token);
        wsUrl = url.toString();
      }

      this.socket = new WebSocket(wsUrl);
      this.setupEventListeners();
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
      this.isConnecting = false;
      this.handlers.onError?.(error as Event);
    }
  }

  /**
   * Disconnect from the WebSocket server
   */
  disconnect(): void {
    this.isManuallyDisconnected = true;
    
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
    
    if (this.socket) {
      this.socket.close(1000, 'Manual disconnect');
      this.socket = null;
    }
    
    this.isConnecting = false;
    this.reconnectAttempts = 0;
    this.subscriptions.clear();
  }

  /**
   * Check if the WebSocket is connected
   */
  isConnected(): boolean {
    return this.socket?.readyState === WebSocket.OPEN;
  }

  /**
   * Send a message to the server
   */
  send(message: any): void {
    if (this.isConnected()) {
      try {
        const messageStr = typeof message === 'string' ? message : JSON.stringify(message);
        this.socket!.send(messageStr);
      } catch (error) {
        console.error('Failed to send WebSocket message:', error);
      }
    } else {
      console.warn('WebSocket not connected. Cannot send message:', message);
    }
  }

  /**
   * Send a structured event message
   */
  emit(event: string, data?: any): void {
    this.send({
      type: event,
      payload: data,
      timestamp: new Date().toISOString(),
    });
  }

  /**
   * Subscribe to system updates
   */
  subscribeToSystemUpdates(): void {
    const subscription = 'system';
    this.subscriptions.add(subscription);
    this.emit('subscribe', { channel: 'system' });
  }

  /**
   * Subscribe to user-specific updates
   */
  subscribeToUserUpdates(userId: string): void {
    const subscription = `user:${userId}`;
    this.subscriptions.add(subscription);
    this.emit('subscribe', { channel: 'user', userId });
  }

  /**
   * Subscribe to operation updates
   */
  subscribeToOperationUpdates(operationId?: string): void {
    const subscription = `operations:${operationId || 'all'}`;
    this.subscriptions.add(subscription);
    this.emit('subscribe', { 
      channel: 'operations', 
      operationId: operationId || 'all' 
    });
  }

  /**
   * Subscribe to reserve updates
   */
  subscribeToReserveUpdates(): void {
    const subscription = 'reserves';
    this.subscriptions.add(subscription);
    this.emit('subscribe', { channel: 'reserves' });
  }

  /**
   * Subscribe to compliance updates
   */
  subscribeToComplianceUpdates(): void {
    const subscription = 'compliance';
    this.subscriptions.add(subscription);
    this.emit('subscribe', { channel: 'compliance' });
  }

  /**
   * Subscribe to integration events
   */
  subscribeToIntegrationEvents(): void {
    const subscription = 'integration';
    this.subscriptions.add(subscription);
    this.emit('subscribe', { channel: 'integration' });
  }

  /**
   * Unsubscribe from a channel
   */
  unsubscribe(channel: string, identifier?: string): void {
    const subscription = identifier ? `${channel}:${identifier}` : channel;
    this.subscriptions.delete(subscription);
    this.emit('unsubscribe', { channel, identifier });
  }

  /**
   * Resubscribe to all active subscriptions (used after reconnection)
   */
  private resubscribeAll(): void {
    this.subscriptions.forEach(subscription => {
      const [channel, identifier] = subscription.split(':');
      if (identifier) {
        this.emit('subscribe', { channel, [channel === 'user' ? 'userId' : 'operationId']: identifier });
      } else {
        this.emit('subscribe', { channel });
      }
    });
  }

  /**
   * Update event handlers
   */
  updateHandlers(newHandlers: Partial<WebSocketEventHandlers>): void {
    this.handlers = { ...this.handlers, ...newHandlers };
  }

  /**
   * Setup event listeners for the native WebSocket
   */
  private setupEventListeners(): void {
    if (!this.socket) return;

    // Connection opened
    this.socket.onopen = () => {
      console.log('WebSocket connected to Loco.rs backend');
      this.isConnecting = false;
      this.reconnectAttempts = 0;
      
      // Start heartbeat
      this.startHeartbeat();
      
      // Resubscribe to channels
      this.resubscribeAll();
      
      this.handlers.onConnect?.();
    };

    // Connection closed
    this.socket.onclose = (event) => {
      console.log('WebSocket disconnected:', event.code, event.reason);
      this.isConnecting = false;
      
      // Stop heartbeat
      if (this.heartbeatInterval) {
        clearInterval(this.heartbeatInterval);
        this.heartbeatInterval = null;
      }
      
      this.handlers.onDisconnect?.(event.reason || `Code: ${event.code}`);
      
      // Attempt to reconnect if not manually disconnected
      if (!this.isManuallyDisconnected && event.code !== 1000) {
        this.attemptReconnect();
      }
    };

    // Connection error
    this.socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      this.isConnecting = false;
      this.handlers.onError?.(error);
    };

    // Message received
    this.socket.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        this.handleMessage(message);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
        // Handle plain text messages
        this.handlers.onMessage?.({
          type: 'raw_message',
          payload: event.data,
          timestamp: new Date().toISOString(),
        });
      }
    };
  }

  /**
   * Handle incoming WebSocket messages
   */
  private handleMessage(message: any): void {
    const { type, payload, timestamp } = message;
    
    // Create standardized message object
    const wsMessage: WebSocketMessage = {
      type,
      payload,
      timestamp: timestamp || new Date().toISOString(),
    };
    
    // Route to specific handlers based on message type
    switch (type) {
      case 'system_update':
        this.handlers.onSystemUpdate?.(payload);
        break;
      case 'operation_update':
        this.handlers.onOperationUpdate?.(payload);
        break;
      case 'alert':
        this.handlers.onAlert?.(payload);
        break;
      case 'reserve_update':
        this.handlers.onReserveUpdate?.(payload);
        break;
      case 'compliance_update':
        this.handlers.onComplianceUpdate?.(payload);
        break;
      case 'pong':
        // Heartbeat response - no action needed
        break;
      default:
        console.log('Received WebSocket message:', type, payload);
    }
    
    // Always call the generic message handler
    this.handlers.onMessage?.(wsMessage);
  }

  /**
   * Start heartbeat to keep connection alive
   */
  private startHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
    }
    
    this.heartbeatInterval = setInterval(() => {
      if (this.isConnected()) {
        this.emit('ping', { timestamp: Date.now() });
      }
    }, 30000); // Send ping every 30 seconds
  }

  /**
   * Attempt to reconnect with exponential backoff
   */
  private attemptReconnect(): void {
    if (this.isManuallyDisconnected || this.reconnectAttempts >= this.maxReconnectAttempts) {
      if (this.reconnectAttempts >= this.maxReconnectAttempts) {
        console.error('Max WebSocket reconnection attempts reached');
      }
      return;
    }

    this.reconnectAttempts++;
    const delay = Math.min(WS_RECONNECT_INTERVAL * Math.pow(2, this.reconnectAttempts - 1), 30000);

    console.log(`Attempting WebSocket reconnection in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`);

    this.reconnectTimer = setTimeout(() => {
      if (!this.isConnected() && !this.isManuallyDisconnected) {
        // Get token from localStorage for reconnection
        const token = localStorage.getItem('auth_token');
        this.connect(token || undefined);
      }
    }, delay);
  }
}

// Create a singleton instance
let wsClient: WebSocketClient | null = null;

/**
 * Get or create the WebSocket client instance
 */
export const getWebSocketClient = (handlers?: WebSocketEventHandlers): WebSocketClient => {
  if (!wsClient) {
    wsClient = new WebSocketClient(handlers);
  } else if (handlers) {
    wsClient.updateHandlers(handlers);
  }
  return wsClient;
};

/**
 * Initialize WebSocket connection with authentication for Loco.rs backend
 */
export const initializeWebSocket = (token?: string, handlers?: WebSocketEventHandlers): WebSocketClient => {
  const client = getWebSocketClient(handlers);
  
  // Add connection status logging
  if (import.meta.env.DEV) {
    console.log('Initializing WebSocket connection to Loco.rs backend:', WS_URL);
  }
  
  client.connect(token);
  return client;
};

/**
 * Cleanup WebSocket connection
 */
export const cleanupWebSocket = (): void => {
  if (wsClient) {
    wsClient.disconnect();
    wsClient = null;
  }
};

/**
 * Get connection status
 */
export const getWebSocketStatus = (): {
  connected: boolean;
  url: string;
  reconnectAttempts?: number;
} => {
  return {
    connected: wsClient?.isConnected() || false,
    url: WS_URL,
    reconnectAttempts: wsClient ? (wsClient as any).reconnectAttempts : 0,
  };
};

/**
 * Test WebSocket connection
 */
export const testWebSocketConnection = async (token?: string): Promise<{
  success: boolean;
  error?: string;
  latency?: number;
}> => {
  return new Promise((resolve) => {
    const testClient = new WebSocketClient({
      onConnect: () => {
        const latency = Date.now() - startTime;
        testClient.disconnect();
        resolve({ success: true, latency });
      },
      onError: (error) => {
        testClient.disconnect();
        resolve({ success: false, error: error.toString() });
      },
    });
    
    const startTime = Date.now();
    testClient.connect(token);
    
    // Timeout after 10 seconds
    setTimeout(() => {
      if (!testClient.isConnected()) {
        testClient.disconnect();
        resolve({ success: false, error: 'Connection timeout' });
      }
    }, 10000);
  });
};

export default WebSocketClient;