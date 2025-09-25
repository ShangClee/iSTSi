// Core system types
export interface SystemState {
  status: 'operational' | 'degraded' | 'emergency' | 'paused';
  totalOperations: number;
  successRate: number;
  reserveRatio: number;
  activeUsers: number;
  lastUpdate: string;
}

export interface Alert {
  id: string;
  type: 'info' | 'warning' | 'error';
  message: string;
  timestamp: string;
  severity: 'low' | 'medium' | 'high';
}

// API types - Updated for Loco.rs backend
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

// Enhanced API error type
export interface ApiError {
  code?: string;
  message: string;
  details?: Record<string, any>;
  status?: number;
}

// User and authentication types (to be expanded in task 2.3)
export interface User {
  id: string;
  username: string;
  email: string;
  role: string;
  createdAt: string;
}

export interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

// Bitcoin and integration types - Updated for Loco.rs backend
export interface BitcoinDepositRequest {
  amount: number;
  address: string;
  userId: string;
}

export interface TokenWithdrawalRequest {
  amount: number;
  address: string;
  userId: string;
}

// Integration operation result
export interface IntegrationOperationResult {
  operation_id: string;
  transaction_hash?: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  message?: string;
  timestamp: string;
}

// System overview from backend
export interface SystemOverview {
  system_status: string;
  total_operations: number;
  success_rate: number;
  reserve_ratio: number;
  active_users: number;
  last_update: string;
  contracts: {
    integration_router: string;
    kyc_registry: string;
    istsi_token: string;
    reserve_manager: string;
  };
}

// Integration status
export interface IntegrationStatus {
  status: string;
  soroban_network: string;
  contracts_configured: boolean;
  event_monitoring_active: boolean;
  last_health_check: number;
}

// WebSocket message types - Updated for Loco.rs backend
export interface WebSocketMessage {
  type: string;
  payload: any;
  timestamp: string;
}

// WebSocket connection status
export interface WebSocketStatus {
  connected: boolean;
  url: string;
  reconnectAttempts: number;
  lastError?: string;
}

// Connection status types
export interface ConnectionInfo {
  api: {
    connected: boolean;
    latency?: number;
    baseUrl: string;
    version?: string;
  };
  websocket: WebSocketStatus;
  overall: 'connected' | 'degraded' | 'disconnected';
}

// Operation and transaction types
export interface Operation {
  id: string;
  type: 'bitcoin_deposit' | 'token_withdrawal' | 'compliance_check';
  status: 'pending' | 'processing' | 'completed' | 'failed';
  userId: string;
  amount: number;
  currency: string;
  createdAt: string;
  updatedAt: string;
  metadata?: Record<string, any>;
}

export interface Transaction {
  id: string;
  operationId: string;
  hash?: string;
  blockHeight?: number;
  confirmations?: number;
  fee?: number;
  createdAt: string;
}

// KYC and compliance types
export interface KycRecord {
  id: string;
  userId: string;
  status: 'pending' | 'approved' | 'rejected' | 'expired';
  tier: 'basic' | 'standard' | 'premium';
  documents: KycDocument[];
  createdAt: string;
  updatedAt: string;
  expiresAt?: string;
}

export interface KycDocument {
  id: string;
  type: 'identity' | 'address' | 'income' | 'other';
  status: 'pending' | 'approved' | 'rejected';
  url?: string;
  metadata?: Record<string, any>;
}

export interface ComplianceCheck {
  id: string;
  transactionId: string;
  status: 'pending' | 'approved' | 'rejected';
  rules: string[];
  reason?: string;
  createdAt: string;
}

// Reserve and token types
export interface ReserveStatus {
  totalReserves: number;
  totalSupply: number;
  reserveRatio: number;
  lastAudit: string;
  proofHash?: string;
}

export interface TokenInfo {
  name: string;
  symbol: string;
  decimals: number;
  totalSupply: number;
  contractAddress: string;
  network: string;
}

// Legacy error type (kept for compatibility)
export interface LegacyApiError {
  code: string;
  message: string;
  details?: Record<string, any>;
}

// Pagination types
export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

// Form types
export interface FormField {
  value: string;
  error?: string;
  touched?: boolean;
}

export interface FormState<T> {
  fields: Record<keyof T, FormField>;
  isValid: boolean;
  isSubmitting: boolean;
}