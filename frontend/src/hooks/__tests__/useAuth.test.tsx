import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { Provider } from 'react-redux';
import { createTestStore, mockUser } from '@/test/utils';
import { useAuth } from '../useAuth';
import { authService } from '@/services/auth';

// Mock the auth service
vi.mock('@/services/auth', () => ({
  authService: {
    login: vi.fn(),
    logout: vi.fn(),
    register: vi.fn(),
    refreshToken: vi.fn(),
    getCurrentUser: vi.fn(),
    getAuthState: vi.fn(),
    subscribe: vi.fn(),
  },
}));

describe('useAuth Hook', () => {
  const mockAuthService = authService as any;
  
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    
    // Setup default mock implementations
    mockAuthService.getAuthState.mockReturnValue({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,
    });
    mockAuthService.subscribe.mockReturnValue(() => {});
  });

  const createWrapper = (initialState?: any) => {
    const store = createTestStore(initialState);
    return ({ children }: { children: React.ReactNode }) => (
      <Provider store={store}>{children}</Provider>
    );
  };

  it('returns initial unauthenticated state', () => {
    const { result } = renderHook(() => useAuth(), {
      wrapper: createWrapper(),
    });

    expect(result.current.user).toBeNull();
    expect(result.current.isAuthenticated).toBe(false);
    expect(result.current.isLoading).toBe(false);
  });

  it('handles successful login', async () => {
    const loginResponse = { success: true };

    mockAuthService.login.mockResolvedValue(loginResponse);

    const { result } = renderHook(() => useAuth(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.login({ email: 'test@example.com', password: 'password' });
    });

    expect(mockAuthService.login).toHaveBeenCalledWith({ 
      email: 'test@example.com', 
      password: 'password' 
    });
  });

  it('handles login failure', async () => {
    const loginResponse = { success: false, error: 'Invalid credentials' };
    mockAuthService.login.mockResolvedValue(loginResponse);

    const { result } = renderHook(() => useAuth(), {
      wrapper: createWrapper(),
    });

    const response = await act(async () => {
      return await result.current.login({ email: 'test@example.com', password: 'wrong-password' });
    });

    expect(response.success).toBe(false);
    expect(response.error).toBe('Invalid credentials');
  });

  it('handles logout', async () => {
    mockAuthService.logout.mockResolvedValue(undefined);

    const { result } = renderHook(() => useAuth(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.logout();
    });

    expect(mockAuthService.logout).toHaveBeenCalled();
  });

  it('handles user registration', async () => {
    const registrationData = {
      email: 'newuser@example.com',
      password: 'password123',
      username: 'newuser',
    };

    const registrationResponse = { success: true };

    mockAuthService.register.mockResolvedValue(registrationResponse);

    const { result } = renderHook(() => useAuth(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.register(registrationData);
    });

    expect(mockAuthService.register).toHaveBeenCalledWith(registrationData);
  });

  it('handles token refresh', async () => {
    mockAuthService.refreshToken.mockResolvedValue(true);

    const { result } = renderHook(() => useAuth(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.refreshToken();
    });

    expect(mockAuthService.refreshToken).toHaveBeenCalled();
  });

  it('gets current user', async () => {
    mockAuthService.getCurrentUser.mockResolvedValue(mockUser);

    const { result } = renderHook(() => useAuth(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.getCurrentUser();
    });

    expect(mockAuthService.getCurrentUser).toHaveBeenCalled();
  });

  it('handles loading states correctly', async () => {
    // Mock loading state
    mockAuthService.getAuthState.mockReturnValue({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: true,
    });

    const { result } = renderHook(() => useAuth(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);
  });
});