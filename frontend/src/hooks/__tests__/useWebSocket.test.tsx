import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { Provider } from 'react-redux';
import { createTestStore } from '@/test/utils';
import { useWebSocket } from '../useWebSocket';

// Mock the websocket service
vi.mock('@/services/websocket', () => ({
  getWebSocketClient: vi.fn(() => ({
    isConnected: vi.fn(() => false),
    connect: vi.fn(),
    disconnect: vi.fn(),
    emit: vi.fn(),
    updateHandlers: vi.fn(),
    subscribeToSystemUpdates: vi.fn(),
    subscribeToUserUpdates: vi.fn(),
    subscribeToOperationUpdates: vi.fn(),
    subscribeToReserveUpdates: vi.fn(),
    subscribeToComplianceUpdates: vi.fn(),
    unsubscribe: vi.fn(),
  })),
}));

describe('useWebSocket Hook', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  const createWrapper = (initialState?: any) => {
    const store = createTestStore(initialState);
    return ({ children }: { children: React.ReactNode }) => (
      <Provider store={store}>{children}</Provider>
    );
  };

  it('initializes with default state', () => {
    const { result } = renderHook(() => useWebSocket({}, { autoConnect: false }), {
      wrapper: createWrapper(),
    });

    expect(result.current.isConnected).toBe(false);
    expect(result.current.connectionState).toBe('disconnected');
    expect(result.current.lastMessage).toBeNull();
  });

  it('provides connect function', () => {
    const { result } = renderHook(() => useWebSocket(), {
      wrapper: createWrapper(),
    });

    expect(typeof result.current.connect).toBe('function');
    
    act(() => {
      result.current.connect();
    });

    // Function should be callable without errors
  });

  it('provides disconnect function', () => {
    const { result } = renderHook(() => useWebSocket(), {
      wrapper: createWrapper(),
    });

    expect(typeof result.current.disconnect).toBe('function');
    
    act(() => {
      result.current.disconnect();
    });

    // Function should be callable without errors
  });

  it('provides emit function', () => {
    const { result } = renderHook(() => useWebSocket(), {
      wrapper: createWrapper(),
    });

    expect(typeof result.current.emit).toBe('function');
    
    act(() => {
      result.current.emit('test_event', { data: 'test' });
    });

    // Function should be callable without errors
  });

  it('provides subscribe function', () => {
    const { result } = renderHook(() => useWebSocket(), {
      wrapper: createWrapper(),
    });

    expect(typeof result.current.subscribe).toBe('function');
    
    act(() => {
      result.current.subscribe('system');
    });

    // Function should be callable without errors
  });

  it('provides unsubscribe function', () => {
    const { result } = renderHook(() => useWebSocket(), {
      wrapper: createWrapper(),
    });

    expect(typeof result.current.unsubscribe).toBe('function');
    
    act(() => {
      result.current.unsubscribe('system');
    });

    // Function should be callable without errors
  });

  it('handles message handlers', () => {
    const onMessage = vi.fn();
    const { result } = renderHook(() => useWebSocket({ onMessage }), {
      wrapper: createWrapper(),
    });

    // Hook should initialize without errors
    expect(result.current.isConnected).toBe(false);
  });

  it('handles connection options', () => {
    const { result } = renderHook(() => useWebSocket({}, { autoConnect: false }), {
      wrapper: createWrapper(),
    });

    // Hook should initialize without errors
    expect(result.current.isConnected).toBe(false);
  });
});