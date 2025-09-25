import { authApi } from './api';
import { initializeWebSocket, cleanupWebSocket, getWebSocketClient } from './websocket';
import type { User, AuthState } from '@/types';

// Storage keys
const TOKEN_KEY = 'auth_token';
const USER_KEY = 'user';
const REFRESH_TOKEN_KEY = 'refresh_token';

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface RegisterData {
  email: string;
  password: string;
  username: string;
}

export class AuthService {
  private static instance: AuthService;
  private authState: AuthState = {
    user: null,
    token: null,
    isAuthenticated: false,
    isLoading: false,
  };
  private listeners: Array<(state: AuthState) => void> = [];
  private refreshTimer: NodeJS.Timeout | null = null;

  private constructor() {
    this.initializeFromStorage();
  }

  /**
   * Get singleton instance
   */
  static getInstance(): AuthService {
    if (!AuthService.instance) {
      AuthService.instance = new AuthService();
    }
    return AuthService.instance;
  }

  /**
   * Get current authentication state
   */
  getAuthState(): AuthState {
    return { ...this.authState };
  }

  /**
   * Subscribe to authentication state changes
   */
  subscribe(listener: (state: AuthState) => void): () => void {
    this.listeners.push(listener);
    
    // Return unsubscribe function
    return () => {
      const index = this.listeners.indexOf(listener);
      if (index > -1) {
        this.listeners.splice(index, 1);
      }
    };
  }

  /**
   * Login with email and password
   */
  async login(credentials: LoginCredentials): Promise<{ success: boolean; error?: string }> {
    try {
      this.setLoading(true);

      const response = await authApi.login(credentials.email, credentials.password);
      
      if (response.success && response.data) {
        const { user, token } = response.data;
        
        // Store authentication data
        this.setAuthData(user, token);
        
        // Initialize WebSocket connection with enhanced error handling
        try {
          const wsClient = initializeWebSocket(token, {
            onConnect: () => {
              console.log('WebSocket connected after login');
              // Subscribe to user-specific updates
              wsClient.subscribeToUserUpdates(user.id);
              wsClient.subscribeToSystemUpdates();
            },
            onError: (error) => {
              console.warn('WebSocket connection failed after login:', error);
              // Don't fail login if WebSocket fails
            },
          });
        } catch (wsError) {
          console.warn('Failed to initialize WebSocket after login:', wsError);
          // Continue with login even if WebSocket fails
        }
        
        // Setup token refresh
        this.setupTokenRefresh(token);
        
        return { success: true };
      } else {
        return { success: false, error: response.error || 'Login failed' };
      }
    } catch (error: any) {
      console.error('Login error:', error);
      return { 
        success: false, 
        error: error.message || 'Login failed' 
      };
    } finally {
      this.setLoading(false);
    }
  }

  /**
   * Register new user
   */
  async register(userData: RegisterData): Promise<{ success: boolean; error?: string }> {
    try {
      this.setLoading(true);

      const response = await authApi.register(userData);
      
      if (response.success && response.data) {
        const { user, token } = response.data;
        
        // Store authentication data
        this.setAuthData(user, token);
        
        // Initialize WebSocket connection with enhanced error handling
        try {
          const wsClient = initializeWebSocket(token, {
            onConnect: () => {
              console.log('WebSocket connected after registration');
              // Subscribe to user-specific updates
              wsClient.subscribeToUserUpdates(user.id);
              wsClient.subscribeToSystemUpdates();
            },
            onError: (error) => {
              console.warn('WebSocket connection failed after registration:', error);
              // Don't fail registration if WebSocket fails
            },
          });
        } catch (wsError) {
          console.warn('Failed to initialize WebSocket after registration:', wsError);
          // Continue with registration even if WebSocket fails
        }
        
        // Setup token refresh
        this.setupTokenRefresh(token);
        
        return { success: true };
      } else {
        return { success: false, error: response.error || 'Registration failed' };
      }
    } catch (error: any) {
      console.error('Registration error:', error);
      return { 
        success: false, 
        error: error.message || 'Registration failed' 
      };
    } finally {
      this.setLoading(false);
    }
  }

  /**
   * Logout user
   */
  async logout(): Promise<void> {
    try {
      // Call logout API
      await authApi.logout();
    } catch (error) {
      console.error('Logout API error:', error);
      // Continue with local logout even if API fails
    }

    // Clear local state and storage
    this.clearAuthData();
    
    // Cleanup WebSocket connection
    cleanupWebSocket();
    
    // Clear token refresh timer
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }
  }

  /**
   * Refresh authentication token
   */
  async refreshToken(): Promise<boolean> {
    try {
      const response = await authApi.refreshToken();
      
      if (response.success && response.data) {
        const { token } = response.data;
        
        // Update stored token
        localStorage.setItem(TOKEN_KEY, token);
        this.updateAuthState({ token });
        
        // Setup next refresh
        this.setupTokenRefresh(token);
        
        return true;
      }
      
      return false;
    } catch (error) {
      console.error('Token refresh error:', error);
      
      // If refresh fails, logout user
      await this.logout();
      
      return false;
    }
  }

  /**
   * Get current user information
   */
  async getCurrentUser(): Promise<User | null> {
    try {
      const response = await authApi.getCurrentUser();
      
      if (response.success && response.data) {
        const user = response.data;
        
        // Update stored user data
        localStorage.setItem(USER_KEY, JSON.stringify(user));
        this.updateAuthState({ user });
        
        return user;
      }
      
      return null;
    } catch (error) {
      console.error('Get current user error:', error);
      return null;
    }
  }

  /**
   * Check if user is authenticated
   */
  isAuthenticated(): boolean {
    return this.authState.isAuthenticated && !!this.authState.token;
  }

  /**
   * Get current user
   */
  getCurrentUserSync(): User | null {
    return this.authState.user;
  }

  /**
   * Get current token
   */
  getToken(): string | null {
    return this.authState.token;
  }

  /**
   * Initialize authentication state from localStorage
   */
  private initializeFromStorage(): void {
    try {
      const token = localStorage.getItem(TOKEN_KEY);
      const userStr = localStorage.getItem(USER_KEY);
      
      if (token && userStr) {
        const user = JSON.parse(userStr);
        
        // Verify token is not expired (basic check)
        if (this.isTokenValid(token)) {
          this.authState = {
            user,
            token,
            isAuthenticated: true,
            isLoading: false,
          };
          
          // Initialize WebSocket connection with error handling
          try {
            const wsClient = initializeWebSocket(token, {
              onConnect: () => {
                console.log('WebSocket reconnected on app initialization');
                // Subscribe to user-specific updates
                wsClient.subscribeToUserUpdates(user.id);
                wsClient.subscribeToSystemUpdates();
              },
              onError: (error) => {
                console.warn('WebSocket connection failed on initialization:', error);
              },
            });
          } catch (wsError) {
            console.warn('Failed to initialize WebSocket on app start:', wsError);
          }
          
          // Setup token refresh
          this.setupTokenRefresh(token);
          
          // Verify user data is still valid in background
          this.getCurrentUser().catch(error => {
            console.warn('Failed to verify user data on initialization:', error);
          });
        } else {
          this.clearAuthData();
        }
      }
    } catch (error) {
      console.error('Error initializing auth from storage:', error);
      this.clearAuthData();
    }
  }

  /**
   * Set authentication data
   */
  private setAuthData(user: User, token: string): void {
    // Store in localStorage
    localStorage.setItem(TOKEN_KEY, token);
    localStorage.setItem(USER_KEY, JSON.stringify(user));
    
    // Update state
    this.authState = {
      user,
      token,
      isAuthenticated: true,
      isLoading: false,
    };
    
    this.notifyListeners();
  }

  /**
   * Clear authentication data
   */
  private clearAuthData(): void {
    // Clear localStorage
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(USER_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    
    // Reset state
    this.authState = {
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,
    };
    
    this.notifyListeners();
  }

  /**
   * Update authentication state
   */
  private updateAuthState(updates: Partial<AuthState>): void {
    this.authState = { ...this.authState, ...updates };
    this.notifyListeners();
  }

  /**
   * Set loading state
   */
  private setLoading(isLoading: boolean): void {
    this.updateAuthState({ isLoading });
  }

  /**
   * Notify all listeners of state changes
   */
  private notifyListeners(): void {
    this.listeners.forEach(listener => {
      try {
        listener(this.getAuthState());
      } catch (error) {
        console.error('Error in auth state listener:', error);
      }
    });
  }

  /**
   * Basic token validation (check if it's not expired)
   */
  private isTokenValid(token: string): boolean {
    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      const currentTime = Date.now() / 1000;
      
      return payload.exp > currentTime;
    } catch (error) {
      return false;
    }
  }

  /**
   * Setup automatic token refresh
   */
  private setupTokenRefresh(token: string): void {
    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      const expirationTime = payload.exp * 1000; // Convert to milliseconds
      const currentTime = Date.now();
      const refreshTime = expirationTime - currentTime - (5 * 60 * 1000); // Refresh 5 minutes before expiry
      
      if (refreshTime > 0) {
        this.refreshTimer = setTimeout(() => {
          this.refreshToken();
        }, refreshTime);
      }
    } catch (error) {
      console.error('Error setting up token refresh:', error);
    }
  }
}

// Export singleton instance
export const authService = AuthService.getInstance();

// Export convenience functions
export const login = (credentials: LoginCredentials) => authService.login(credentials);
export const register = (userData: RegisterData) => authService.register(userData);
export const logout = () => authService.logout();
export const isAuthenticated = () => authService.isAuthenticated();
export const getCurrentUser = () => authService.getCurrentUserSync();
export const getToken = () => authService.getToken();
export const subscribeToAuth = (listener: (state: AuthState) => void) => authService.subscribe(listener);

export default authService;