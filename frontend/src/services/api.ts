import axios, { AxiosInstance, AxiosResponse, AxiosError, AxiosRequestConfig } from 'axios';
import type { 
  ApiResponse, 
  BitcoinDepositRequest, 
  TokenWithdrawalRequest,
  SystemState,
  User 
} from '@/types';

// API Configuration
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

// Security Configuration
interface SecurityConfig {
  enableCSRFProtection: boolean;
  enableRequestSigning: boolean;
  maxRetries: number;
  timeoutMs: number;
  rateLimitPerMinute: number;
}

const SECURITY_CONFIG: SecurityConfig = {
  enableCSRFProtection: true,
  enableRequestSigning: false, // Enable in production
  maxRetries: 3,
  timeoutMs: 15000,
  rateLimitPerMinute: 60,
};

// Retry configuration
interface RetryConfig {
  retries: number;
  retryDelay: number;
  retryCondition?: (error: AxiosError) => boolean;
}

const DEFAULT_RETRY_CONFIG: RetryConfig = {
  retries: 3,
  retryDelay: 1000,
  retryCondition: (error: AxiosError) => {
    // Retry on network errors or 5xx server errors
    return !error.response || (error.response.status >= 500 && error.response.status < 600);
  },
};

// Security utilities
class SecurityUtils {
  private static requestCounts = new Map<string, { count: number; resetTime: number }>();

  static checkRateLimit(endpoint: string): boolean {
    const now = Date.now();
    const key = endpoint;
    const current = this.requestCounts.get(key);

    if (!current || now > current.resetTime) {
      this.requestCounts.set(key, {
        count: 1,
        resetTime: now + 60000, // 1 minute
      });
      return true;
    }

    if (current.count >= SECURITY_CONFIG.rateLimitPerMinute) {
      return false;
    }

    current.count++;
    return true;
  }

  static sanitizeInput(input: any): any {
    if (typeof input === 'string') {
      // Basic XSS prevention
      return input
        .replace(/[<>]/g, '')
        .replace(/javascript:/gi, '')
        .replace(/on\w+=/gi, '');
    }
    
    if (Array.isArray(input)) {
      return input.map(item => this.sanitizeInput(item));
    }
    
    if (typeof input === 'object' && input !== null) {
      const sanitized: any = {};
      for (const [key, value] of Object.entries(input)) {
        sanitized[key] = this.sanitizeInput(value);
      }
      return sanitized;
    }
    
    return input;
  }

  static generateRequestId(): string {
    return `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  static validateResponse(response: any): boolean {
    // Basic response validation
    if (!response || typeof response !== 'object') {
      return false;
    }
    
    // Check for suspicious content
    const responseStr = JSON.stringify(response);
    const suspiciousPatterns = [
      /<script/i,
      /javascript:/i,
      /on\w+=/i,
      /eval\(/i,
    ];
    
    return !suspiciousPatterns.some(pattern => pattern.test(responseStr));
  }
}

// Create axios instance with enhanced security configuration
export const apiClient: AxiosInstance = axios.create({
  baseURL: `${API_BASE_URL}/api`,
  timeout: SECURITY_CONFIG.timeoutMs,
  headers: {
    'Content-Type': 'application/json',
    'X-Requested-With': 'XMLHttpRequest', // CSRF protection
  },
  withCredentials: false, // Disable credentials for security
  maxRedirects: 0, // Prevent redirect attacks
});

// Add retry functionality
const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

const axiosRetry = async (
  config: AxiosRequestConfig & { _retry?: number },
  retryConfig: RetryConfig = DEFAULT_RETRY_CONFIG
): Promise<AxiosResponse> => {
  try {
    return await axios(config);
  } catch (error) {
    const axiosError = error as AxiosError;
    const currentRetry = config._retry || 0;
    
    if (
      currentRetry < retryConfig.retries &&
      retryConfig.retryCondition &&
      retryConfig.retryCondition(axiosError)
    ) {
      console.warn(`Request failed, retrying (${currentRetry + 1}/${retryConfig.retries})...`);
      
      // Exponential backoff
      const delay = retryConfig.retryDelay * Math.pow(2, currentRetry);
      await sleep(delay);
      
      return axiosRetry({ ...config, _retry: currentRetry + 1 }, retryConfig);
    }
    
    throw error;
  }
};

// Request interceptor with enhanced security
apiClient.interceptors.request.use(
  (config) => {
    // Rate limiting check
    const endpoint = config.url || '';
    if (!SecurityUtils.checkRateLimit(endpoint)) {
      throw new Error('Rate limit exceeded. Please try again later.');
    }

    // Add authentication token
    const token = localStorage.getItem('auth_token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }

    // Add security headers
    config.headers['X-Request-ID'] = SecurityUtils.generateRequestId();
    config.headers['X-Client-Version'] = import.meta.env.VITE_APP_VERSION || '1.0.0';
    
    // CSRF protection
    if (SECURITY_CONFIG.enableCSRFProtection) {
      const csrfToken = document.querySelector('meta[name="csrf-token"]')?.getAttribute('content');
      if (csrfToken) {
        config.headers['X-CSRF-Token'] = csrfToken;
      }
    }

    // Sanitize request data
    if (config.data) {
      config.data = SecurityUtils.sanitizeInput(config.data);
    }

    // Add request metadata
    (config as any).metadata = { 
      startTime: new Date(),
      requestId: config.headers['X-Request-ID'],
    };
    
    return config;
  },
  (error) => {
    console.error('Request interceptor error:', error);
    return Promise.reject(error);
  }
);

// Response interceptor with enhanced security and logging
apiClient.interceptors.response.use(
  (response: AxiosResponse) => {
    // Validate response security
    if (!SecurityUtils.validateResponse(response.data)) {
      console.warn('Potentially unsafe response detected');
      // In production, you might want to reject the response
    }

    // Log successful requests in development
    if (import.meta.env.DEV) {
      const metadata = (response.config as any).metadata;
      const duration = metadata?.startTime ? new Date().getTime() - metadata.startTime.getTime() : 0;
      const requestId = metadata?.requestId || 'unknown';
      console.log(`✅ [${requestId}] ${response.config.method?.toUpperCase()} ${response.config.url} - ${response.status} (${duration}ms)`);
    }

    // Security headers validation
    const securityHeaders = [
      'x-content-type-options',
      'x-frame-options',
      'x-xss-protection',
    ];

    if (import.meta.env.DEV) {
      securityHeaders.forEach(header => {
        if (!response.headers[header]) {
          console.warn(`Missing security header: ${header}`);
        }
      });
    }
    
    return response;
  },
  async (error: AxiosError) => {
    // Log errors
    if (import.meta.env.DEV) {
      const metadata = (error.config as any)?.metadata;
      const duration = metadata?.startTime 
        ? new Date().getTime() - metadata.startTime.getTime()
        : 0;
      console.error(`❌ ${error.config?.method?.toUpperCase()} ${error.config?.url} - ${error.response?.status || 'Network Error'} (${duration}ms)`);
    }
    
    // Handle 401 Unauthorized
    if (error.response?.status === 401) {
      localStorage.removeItem('auth_token');
      localStorage.removeItem('user');
      
      // Dispatch custom event for auth state change
      window.dispatchEvent(new CustomEvent('auth:logout'));
      
      // Only redirect if not already on login page
      if (!window.location.pathname.includes('/login')) {
        window.location.href = '/login';
      }
    }
    
    // Handle network errors with retry
    if (!error.response && error.config && !error.config._retry) {
      try {
        return await axiosRetry(error.config);
      } catch (retryError) {
        console.error('Request failed after retries:', retryError);
      }
    }
    
    // Enhanced error information
    const enhancedError = {
      ...error,
      message: error.response?.data?.message || error.message,
      code: error.response?.data?.code || error.code,
      status: error.response?.status,
    };
    
    return Promise.reject(enhancedError);
  }
);

// Authentication API - Updated for Loco.rs backend
export const authApi = {
  login: async (email: string, password: string): Promise<ApiResponse<{ user: User; token: string }>> => {
    try {
      const response = await apiClient.post('/auth/login', { email, password });
      
      // Handle Loco.rs response format
      if (response.data.token && response.data.user) {
        return {
          success: true,
          data: {
            user: response.data.user,
            token: response.data.token,
          },
        };
      }
      
      return {
        success: false,
        error: response.data.message || 'Login failed',
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Login failed',
      };
    }
  },

  register: async (userData: {
    email: string;
    password: string;
    username: string;
  }): Promise<ApiResponse<{ user: User; token: string }>> => {
    try {
      const response = await apiClient.post('/auth/register', userData);
      
      // Handle Loco.rs response format
      if (response.data.token && response.data.user) {
        return {
          success: true,
          data: {
            user: response.data.user,
            token: response.data.token,
          },
        };
      }
      
      return {
        success: false,
        error: response.data.message || 'Registration failed',
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Registration failed',
      };
    }
  },

  logout: async (): Promise<ApiResponse> => {
    try {
      const response = await apiClient.post('/auth/logout');
      return {
        success: true,
        message: response.data.message || 'Logged out successfully',
      };
    } catch (error: any) {
      // Even if logout fails on server, we should clear local state
      return {
        success: true,
        message: 'Logged out locally',
      };
    }
  },

  refreshToken: async (): Promise<ApiResponse<{ token: string }>> => {
    try {
      const response = await apiClient.post('/auth/refresh');
      
      if (response.data.token) {
        return {
          success: true,
          data: { token: response.data.token },
        };
      }
      
      return {
        success: false,
        error: 'Token refresh failed',
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Token refresh failed',
      };
    }
  },

  getCurrentUser: async (): Promise<ApiResponse<User>> => {
    try {
      const response = await apiClient.get('/auth/me');
      
      if (response.data.user) {
        return {
          success: true,
          data: response.data.user,
        };
      }
      
      return {
        success: false,
        error: 'User data not found',
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get user data',
      };
    }
  },
};

// System API - Updated for Loco.rs backend
export const systemApi = {
  getOverview: async (): Promise<ApiResponse<SystemState>> => {
    try {
      const response = await apiClient.get('/integration/system-overview');
      
      // Transform backend response to frontend format
      const systemState: SystemState = {
        status: response.data.system_status || 'operational',
        totalOperations: response.data.total_operations || 0,
        successRate: response.data.success_rate || 0,
        reserveRatio: response.data.reserve_ratio || 0,
        activeUsers: response.data.active_users || 0,
        lastUpdate: new Date().toISOString(),
      };
      
      return {
        success: true,
        data: systemState,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get system overview',
      };
    }
  },

  getHealth: async (): Promise<ApiResponse<{ status: string; timestamp: string }>> => {
    try {
      const response = await apiClient.get('/system/health');
      return {
        success: true,
        data: {
          status: 'healthy',
          timestamp: new Date().toISOString(),
        },
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Health check failed',
      };
    }
  },

  getVersion: async (): Promise<ApiResponse<{ version: string; build: string }>> => {
    try {
      const response = await apiClient.get('/system/version');
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get version info',
      };
    }
  },

  getIntegrationStatus: async (): Promise<ApiResponse<any>> => {
    try {
      const response = await apiClient.get('/integration/status');
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get integration status',
      };
    }
  },
};

// Integration API - Updated for Loco.rs backend
export const integrationApi = {
  executeBitcoinDeposit: async (params: BitcoinDepositRequest): Promise<ApiResponse<{ transactionId: string }>> => {
    try {
      // Transform frontend request to backend format
      const payload = {
        user_address: params.address,
        btc_amount: Math.floor(params.amount * 100000000), // Convert to satoshis
        btc_tx_hash: `mock_tx_${Date.now()}`, // This would come from Bitcoin network
        confirmations: 6, // Default confirmations
      };
      
      const response = await apiClient.post('/integration/bitcoin-deposit', payload);
      
      return {
        success: true,
        data: {
          transactionId: response.data.transaction_hash || response.data.operation_id,
        },
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Bitcoin deposit failed',
      };
    }
  },

  executeTokenWithdrawal: async (params: TokenWithdrawalRequest): Promise<ApiResponse<{ transactionId: string }>> => {
    try {
      // Transform frontend request to backend format
      const payload = {
        user_address: params.userId, // User's Stellar address
        token_amount: Math.floor(params.amount * 100000000), // Convert to smallest unit
        btc_address: params.address, // Bitcoin withdrawal address
      };
      
      const response = await apiClient.post('/integration/token-withdrawal', payload);
      
      return {
        success: true,
        data: {
          transactionId: response.data.transaction_hash || response.data.operation_id,
        },
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Token withdrawal failed',
      };
    }
  },

  executeCrossTokenExchange: async (params: {
    userAddress: string;
    fromToken: string;
    toToken: string;
    amount: number;
  }): Promise<ApiResponse<{ transactionId: string }>> => {
    try {
      const payload = {
        user_address: params.userAddress,
        from_token: params.fromToken,
        to_token: params.toToken,
        amount: Math.floor(params.amount * 100000000),
      };
      
      const response = await apiClient.post('/integration/cross-token-exchange', payload);
      
      return {
        success: true,
        data: {
          transactionId: response.data.transaction_hash || response.data.operation_id,
        },
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Cross-token exchange failed',
      };
    }
  },

  getEvents: async (params?: {
    contract?: string;
    eventType?: string;
    limit?: number;
  }): Promise<ApiResponse<any[]>> => {
    try {
      const response = await apiClient.get('/integration/events', { params });
      
      return {
        success: true,
        data: response.data || [],
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get events',
      };
    }
  },

  getTransactionStatus: async (transactionHash: string): Promise<ApiResponse<any>> => {
    try {
      const response = await apiClient.post('/integration/transaction-status', {
        transaction_hash: transactionHash,
      });
      
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get transaction status',
      };
    }
  },

  getEventStatistics: async (): Promise<ApiResponse<any>> => {
    try {
      const response = await apiClient.get('/integration/event-statistics');
      
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get event statistics',
      };
    }
  },

  configureIntegration: async (config: {
    enableSigning: boolean;
    secretKey?: string;
    enableEventMonitoring: boolean;
  }): Promise<ApiResponse<any>> => {
    try {
      const payload = {
        enable_signing: config.enableSigning,
        secret_key: config.secretKey,
        enable_event_monitoring: config.enableEventMonitoring,
      };
      
      const response = await apiClient.post('/integration/configure', payload);
      
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Integration configuration failed',
      };
    }
  },
};

// KYC and Compliance API - Updated for Loco.rs backend
export const complianceApi = {
  getKycStatus: async (userId: string): Promise<ApiResponse<any>> => {
    try {
      const response = await apiClient.get(`/kyc/status/${userId}`);
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get KYC status',
      };
    }
  },

  submitKycData: async (userId: string, kycData: any): Promise<ApiResponse> => {
    try {
      const response = await apiClient.post(`/kyc/submit/${userId}`, kycData);
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to submit KYC data',
      };
    }
  },

  getComplianceReport: async (dateRange?: { from: string; to: string }): Promise<ApiResponse<any>> => {
    try {
      const response = await apiClient.get('/kyc/compliance-report', {
        params: dateRange,
      });
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get compliance report',
      };
    }
  },

  checkTransactionCompliance: async (transactionData: any): Promise<ApiResponse<{ approved: boolean; reason?: string }>> => {
    try {
      const response = await apiClient.post('/kyc/check-compliance', transactionData);
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Compliance check failed',
      };
    }
  },
};

// Token Management API - Updated for Loco.rs backend
export const tokenApi = {
  getBalance: async (userId: string): Promise<ApiResponse<{ balance: number; currency: string }>> => {
    try {
      const response = await apiClient.get(`/tokens/balance/${userId}`);
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get token balance',
      };
    }
  },

  getTokenInfo: async (): Promise<ApiResponse<any>> => {
    try {
      const response = await apiClient.get('/tokens/info');
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get token info',
      };
    }
  },

  getTransactionHistory: async (userId: string): Promise<ApiResponse<any[]>> => {
    try {
      const response = await apiClient.get(`/tokens/transactions/${userId}`);
      return {
        success: true,
        data: response.data || [],
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get transaction history',
      };
    }
  },
};

// Reserve Management API - Updated for Loco.rs backend
export const reserveApi = {
  getReserveStatus: async (): Promise<ApiResponse<any>> => {
    try {
      const response = await apiClient.get('/reserves/status');
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get reserve status',
      };
    }
  },

  getProofOfReserves: async (): Promise<ApiResponse<any>> => {
    try {
      const response = await apiClient.get('/reserves/proof');
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get proof of reserves',
      };
    }
  },

  getReserveHistory: async (dateRange?: { from: string; to: string }): Promise<ApiResponse<any[]>> => {
    try {
      const response = await apiClient.get('/reserves/history', {
        params: dateRange,
      });
      return {
        success: true,
        data: response.data || [],
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get reserve history',
      };
    }
  },
};

// User Management API - New for Loco.rs backend
export const userApi = {
  getProfile: async (userId: string): Promise<ApiResponse<User>> => {
    try {
      const response = await apiClient.get(`/users/${userId}`);
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get user profile',
      };
    }
  },

  updateProfile: async (userId: string, userData: Partial<User>): Promise<ApiResponse<User>> => {
    try {
      const response = await apiClient.put(`/users/${userId}`, userData);
      return {
        success: true,
        data: response.data,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to update user profile',
      };
    }
  },

  getUserOperations: async (userId: string): Promise<ApiResponse<any[]>> => {
    try {
      const response = await apiClient.get(`/users/${userId}/operations`);
      return {
        success: true,
        data: response.data || [],
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get user operations',
      };
    }
  },
};

// Health Check and Monitoring API
export const monitoringApi = {
  healthCheck: async (): Promise<ApiResponse<{ status: string; timestamp: string; services: Record<string, string> }>> => {
    try {
      const [systemHealth, integrationStatus] = await Promise.allSettled([
        apiClient.get('/system/health'),
        apiClient.get('/integration/status'),
      ]);
      
      const services: Record<string, string> = {};
      
      if (systemHealth.status === 'fulfilled') {
        services.system = 'healthy';
      } else {
        services.system = 'unhealthy';
      }
      
      if (integrationStatus.status === 'fulfilled') {
        services.integration = integrationStatus.value.data.status || 'unknown';
      } else {
        services.integration = 'unhealthy';
      }
      
      const overallStatus = Object.values(services).every(status => status === 'healthy') ? 'healthy' : 'degraded';
      
      return {
        success: true,
        data: {
          status: overallStatus,
          timestamp: new Date().toISOString(),
          services,
        },
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Health check failed',
      };
    }
  },

  getMetrics: async (): Promise<ApiResponse<any>> => {
    try {
      const [systemOverview, eventStats] = await Promise.allSettled([
        apiClient.get('/integration/system-overview'),
        apiClient.get('/integration/event-statistics'),
      ]);
      
      const metrics: any = {
        timestamp: new Date().toISOString(),
      };
      
      if (systemOverview.status === 'fulfilled') {
        metrics.system = systemOverview.value.data;
      }
      
      if (eventStats.status === 'fulfilled') {
        metrics.events = eventStats.value.data;
      }
      
      return {
        success: true,
        data: metrics,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.message || error.message || 'Failed to get metrics',
      };
    }
  },
};

// Connection testing utilities
export const connectionUtils = {
  testConnection: async (): Promise<{ success: boolean; latency?: number; error?: string }> => {
    const startTime = Date.now();
    
    try {
      await apiClient.get('/system/health', { timeout: 5000 });
      const latency = Date.now() - startTime;
      
      return {
        success: true,
        latency,
      };
    } catch (error: any) {
      return {
        success: false,
        error: error.message || 'Connection test failed',
      };
    }
  },

  testBackendEndpoints: async (): Promise<Record<string, boolean>> => {
    const endpoints = [
      '/system/health',
      '/system/version',
      '/integration/status',
    ];
    
    const results: Record<string, boolean> = {};
    
    await Promise.allSettled(
      endpoints.map(async (endpoint) => {
        try {
          await apiClient.get(endpoint, { timeout: 3000 });
          results[endpoint] = true;
        } catch {
          results[endpoint] = false;
        }
      })
    );
    
    return results;
  },

  getConnectionInfo: async (): Promise<{
    baseUrl: string;
    connected: boolean;
    latency?: number;
    version?: string;
  }> => {
    const baseUrl = API_BASE_URL;
    const connectionTest = await connectionUtils.testConnection();
    
    let version: string | undefined;
    try {
      const versionResponse = await systemApi.getVersion();
      if (versionResponse.success) {
        version = versionResponse.data?.version;
      }
    } catch {
      // Version check failed, but that's okay
    }
    
    return {
      baseUrl,
      connected: connectionTest.success,
      latency: connectionTest.latency,
      version,
    };
  },
};

// Export all APIs as a single object
export const api = {
  auth: authApi,
  system: systemApi,
  integration: integrationApi,
  compliance: complianceApi,
  tokens: tokenApi,
  reserves: reserveApi,
  users: userApi,
  monitoring: monitoringApi,
  connection: connectionUtils,
};

// Export individual APIs for convenience
export {
  authApi,
  systemApi,
  integrationApi,
  complianceApi,
  tokenApi,
  reserveApi,
  userApi,
  monitoringApi,
};

export default api;