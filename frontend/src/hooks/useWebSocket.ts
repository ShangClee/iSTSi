import { useState, useEffect, useCallback, useRef } from 'react';
import { getWebSocketClient, WebSocketEventHandlers } from '@/services/websocket';
import type { WebSocketMessage } from '@/types';

export interface UseWebSocketOptions {
  autoConnect?: boolean;
  reconnectOnMount?: boolean;
}

export interface UseWebSocketReturn {
  isConnected: boolean;
  lastMessage: WebSocketMessage | null;
  connectionState: 'disconnected' | 'connecting' | 'connected' | 'error';
  connect: () => void;
  disconnect: () => void;
  emit: (event: string, data?: any) => void;
  subscribe: (channel: string, identifier?: string) => void;
  unsubscribe: (channel: string, identifier?: string) => void;
}

/**
 * Hook for managing WebSocket connection and real-time updates
 */
export function useWebSocket(
  handlers: Partial<WebSocketEventHandlers> = {},
  options: UseWebSocketOptions = {}
): UseWebSocketReturn {
  const { autoConnect = true, reconnectOnMount = true } = options;
  
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null);
  const [connectionState, setConnectionState] = useState<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected');
  
  const wsClient = useRef(getWebSocketClient());
  const handlersRef = useRef(handlers);

  // Update handlers ref when handlers change
  useEffect(() => {
    handlersRef.current = handlers;
  }, [handlers]);

  // Setup WebSocket event handlers
  useEffect(() => {
    const client = wsClient.current;
    
    const wsHandlers: WebSocketEventHandlers = {
      onConnect: () => {
        setIsConnected(true);
        setConnectionState('connected');
        handlersRef.current.onConnect?.();
      },
      
      onDisconnect: (reason: string) => {
        setIsConnected(false);
        setConnectionState('disconnected');
        handlersRef.current.onDisconnect?.(reason);
      },
      
      onError: (error: Event) => {
        setConnectionState('error');
        handlersRef.current.onError?.(error);
      },
      
      onMessage: (message: WebSocketMessage) => {
        setLastMessage(message);
        handlersRef.current.onMessage?.(message);
      },
      
      onSystemUpdate: (data: any) => {
        handlersRef.current.onSystemUpdate?.(data);
      },
      
      onOperationUpdate: (data: any) => {
        handlersRef.current.onOperationUpdate?.(data);
      },
      
      onAlert: (data: any) => {
        handlersRef.current.onAlert?.(data);
      },
      
      onReserveUpdate: (data: any) => {
        handlersRef.current.onReserveUpdate?.(data);
      },
      
      onComplianceUpdate: (data: any) => {
        handlersRef.current.onComplianceUpdate?.(data);
      },
    };

    client.updateHandlers(wsHandlers);

    // Auto-connect if requested
    if (autoConnect && !client.isConnected()) {
      setConnectionState('connecting');
      const token = localStorage.getItem('auth_token');
      client.connect(token || undefined);
    }

    return () => {
      if (!reconnectOnMount) {
        client.disconnect();
      }
    };
  }, [autoConnect, reconnectOnMount]);

  const connect = useCallback(() => {
    const client = wsClient.current;
    if (!client.isConnected()) {
      setConnectionState('connecting');
      const token = localStorage.getItem('auth_token');
      client.connect(token || undefined);
    }
  }, []);

  const disconnect = useCallback(() => {
    const client = wsClient.current;
    client.disconnect();
    setIsConnected(false);
    setConnectionState('disconnected');
  }, []);

  const emit = useCallback((event: string, data?: any) => {
    const client = wsClient.current;
    client.emit(event, data);
  }, []);

  const subscribe = useCallback((channel: string, identifier?: string) => {
    const client = wsClient.current;
    
    switch (channel) {
      case 'system':
        client.subscribeToSystemUpdates();
        break;
      case 'user':
        if (identifier) {
          client.subscribeToUserUpdates(identifier);
        }
        break;
      case 'operations':
        client.subscribeToOperationUpdates(identifier);
        break;
      case 'reserves':
        client.subscribeToReserveUpdates();
        break;
      case 'compliance':
        client.subscribeToComplianceUpdates();
        break;
      default:
        client.emit('subscribe', { channel, identifier });
    }
  }, []);

  const unsubscribe = useCallback((channel: string, identifier?: string) => {
    const client = wsClient.current;
    client.unsubscribe(channel, identifier);
  }, []);

  return {
    isConnected,
    lastMessage,
    connectionState,
    connect,
    disconnect,
    emit,
    subscribe,
    unsubscribe,
  };
}

/**
 * Hook for subscribing to specific WebSocket channels
 */
export function useWebSocketSubscription(
  channel: string,
  identifier?: string,
  onMessage?: (data: any) => void
) {
  const { subscribe, unsubscribe, lastMessage } = useWebSocket({
    onMessage: (message) => {
      if (message.type === channel || message.type.startsWith(channel)) {
        onMessage?.(message.payload);
      }
    },
  });

  useEffect(() => {
    subscribe(channel, identifier);
    
    return () => {
      unsubscribe(channel, identifier);
    };
  }, [channel, identifier, subscribe, unsubscribe]);

  return { lastMessage };
}

/**
 * Hook for system-wide updates
 */
export function useSystemUpdates(onUpdate?: (data: any) => void) {
  return useWebSocket({
    onSystemUpdate: onUpdate,
  });
}

/**
 * Hook for operation updates
 */
export function useOperationUpdates(operationId?: string, onUpdate?: (data: any) => void) {
  const { subscribe, unsubscribe } = useWebSocket({
    onOperationUpdate: onUpdate,
  });

  useEffect(() => {
    subscribe('operations', operationId);
    
    return () => {
      unsubscribe('operations', operationId);
    };
  }, [operationId, subscribe, unsubscribe]);
}

/**
 * Hook for real-time alerts
 */
export function useAlerts(onAlert?: (data: any) => void) {
  const [alerts, setAlerts] = useState<any[]>([]);

  const { lastMessage } = useWebSocket({
    onAlert: (data) => {
      setAlerts(prev => [data, ...prev]);
      onAlert?.(data);
    },
  });

  const clearAlert = useCallback((alertId: string) => {
    setAlerts(prev => prev.filter(alert => alert.id !== alertId));
  }, []);

  const clearAllAlerts = useCallback(() => {
    setAlerts([]);
  }, []);

  return {
    alerts,
    clearAlert,
    clearAllAlerts,
    lastMessage,
  };
}

export default useWebSocket;