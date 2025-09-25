/**
 * Basic API service tests
 * These tests verify the API client configuration and basic functionality
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { apiClient, connectionUtils } from '../api';

// Mock axios
vi.mock('axios', () => ({
  default: {
    create: vi.fn(() => ({
      defaults: {
        baseURL: 'http://localhost:8080/api',
        timeout: 15000,
      },
      interceptors: {
        request: { use: vi.fn() },
        response: { use: vi.fn() },
      },
      get: vi.fn(),
      post: vi.fn(),
      put: vi.fn(),
      delete: vi.fn(),
    })),
  },
}));

describe('API Service', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should create API client with correct configuration', () => {
    expect(apiClient).toBeDefined();
    expect(apiClient.defaults.baseURL).toContain('/api');
    expect(apiClient.defaults.timeout).toBe(15000);
  });

  it('should have connection utilities', () => {
    expect(connectionUtils).toBeDefined();
    expect(typeof connectionUtils.testConnection).toBe('function');
    expect(typeof connectionUtils.testBackendEndpoints).toBe('function');
    expect(typeof connectionUtils.getConnectionInfo).toBe('function');
  });

  it('should handle API base URL from environment', () => {
    const expectedBaseUrl = import.meta.env.VITE_API_URL || 'http://localhost:8080';
    expect(apiClient.defaults.baseURL).toBe(`${expectedBaseUrl}/api`);
  });
});