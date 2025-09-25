import { createApi, fetchBaseQuery } from '@reduxjs/toolkit/query/react';
import type { RootState } from '../store';
import type { 
  ApiResponse, 
  SystemState, 
  User, 
  BitcoinDepositRequest, 
  TokenWithdrawalRequest 
} from '@/types';

// Base query with authentication
const baseQuery = fetchBaseQuery({
  baseUrl: `${import.meta.env.VITE_API_URL || 'http://localhost:8080'}/api`,
  prepareHeaders: (headers, { getState }) => {
    const token = (getState() as RootState).auth.token;
    if (token) {
      headers.set('authorization', `Bearer ${token}`);
    }
    return headers;
  },
});

// RTK Query API slice
export const apiSlice = createApi({
  reducerPath: 'api',
  baseQuery,
  tagTypes: [
    'User', 
    'SystemState', 
    'Operation', 
    'Alert', 
    'KYC', 
    'Token', 
    'Reserve', 
    'Compliance'
  ],
  endpoints: (builder) => ({
    // System endpoints
    getSystemOverview: builder.query<SystemState, void>({
      query: () => '/system/overview',
      transformResponse: (response: ApiResponse<SystemState>) => response.data!,
      providesTags: ['SystemState'],
    }),

    pauseSystem: builder.mutation<void, void>({
      query: () => ({
        url: '/system/pause',
        method: 'POST',
      }),
      invalidatesTags: ['SystemState'],
    }),

    resumeSystem: builder.mutation<void, void>({
      query: () => ({
        url: '/system/resume',
        method: 'POST',
      }),
      invalidatesTags: ['SystemState'],
    }),

    // User endpoints
    getCurrentUser: builder.query<User, void>({
      query: () => '/auth/me',
      transformResponse: (response: ApiResponse<User>) => response.data!,
      providesTags: ['User'],
    }),

    // Integration endpoints
    executeBitcoinDeposit: builder.mutation<{ transactionId: string }, BitcoinDepositRequest>({
      query: (params) => ({
        url: '/integration/bitcoin-deposit',
        method: 'POST',
        body: params,
      }),
      transformResponse: (response: ApiResponse<{ transactionId: string }>) => response.data!,
      invalidatesTags: ['Operation', 'SystemState'],
    }),

    executeTokenWithdrawal: builder.mutation<{ transactionId: string }, TokenWithdrawalRequest>({
      query: (params) => ({
        url: '/integration/token-withdrawal',
        method: 'POST',
        body: params,
      }),
      transformResponse: (response: ApiResponse<{ transactionId: string }>) => response.data!,
      invalidatesTags: ['Operation', 'SystemState'],
    }),

    getOperationHistory: builder.query<any[], string | void>({
      query: (userId) => ({
        url: '/integration/operations',
        params: userId ? { userId } : {},
      }),
      transformResponse: (response: ApiResponse<any[]>) => response.data || [],
      providesTags: ['Operation'],
    }),

    getOperationStatus: builder.query<any, string>({
      query: (operationId) => `/integration/operations/${operationId}`,
      transformResponse: (response: ApiResponse<any>) => response.data!,
      providesTags: (result, error, operationId) => [{ type: 'Operation', id: operationId }],
    }),

    // KYC and Compliance endpoints
    getKycStatus: builder.query<any, string>({
      query: (userId) => `/compliance/kyc/${userId}`,
      transformResponse: (response: ApiResponse<any>) => response.data!,
      providesTags: (result, error, userId) => [{ type: 'KYC', id: userId }],
    }),

    submitKycData: builder.mutation<void, { userId: string; kycData: any }>({
      query: ({ userId, kycData }) => ({
        url: `/compliance/kyc/${userId}`,
        method: 'POST',
        body: kycData,
      }),
      invalidatesTags: (result, error, { userId }) => [{ type: 'KYC', id: userId }],
    }),

    getComplianceReport: builder.query<any, { from?: string; to?: string } | void>({
      query: (dateRange) => ({
        url: '/compliance/report',
        params: dateRange,
      }),
      transformResponse: (response: ApiResponse<any>) => response.data!,
      providesTags: ['Compliance'],
    }),

    checkTransactionCompliance: builder.mutation<{ approved: boolean; reason?: string }, any>({
      query: (transactionData) => ({
        url: '/compliance/check',
        method: 'POST',
        body: transactionData,
      }),
      transformResponse: (response: ApiResponse<{ approved: boolean; reason?: string }>) => response.data!,
    }),

    // Token endpoints
    getTokenBalance: builder.query<{ balance: number; currency: string }, string>({
      query: (userId) => `/tokens/balance/${userId}`,
      transformResponse: (response: ApiResponse<{ balance: number; currency: string }>) => response.data!,
      providesTags: (result, error, userId) => [{ type: 'Token', id: userId }],
    }),

    getTokenInfo: builder.query<any, void>({
      query: () => '/tokens/info',
      transformResponse: (response: ApiResponse<any>) => response.data!,
      providesTags: ['Token'],
    }),

    getTransactionHistory: builder.query<any[], string>({
      query: (userId) => `/tokens/transactions/${userId}`,
      transformResponse: (response: ApiResponse<any[]>) => response.data || [],
      providesTags: (result, error, userId) => [{ type: 'Token', id: `${userId}-transactions` }],
    }),

    // Reserve endpoints
    getReserveStatus: builder.query<any, void>({
      query: () => '/reserves/status',
      transformResponse: (response: ApiResponse<any>) => response.data!,
      providesTags: ['Reserve'],
    }),

    getProofOfReserves: builder.query<any, void>({
      query: () => '/reserves/proof',
      transformResponse: (response: ApiResponse<any>) => response.data!,
      providesTags: ['Reserve'],
    }),

    getReserveHistory: builder.query<any[], { from?: string; to?: string } | void>({
      query: (dateRange) => ({
        url: '/reserves/history',
        params: dateRange,
      }),
      transformResponse: (response: ApiResponse<any[]>) => response.data || [],
      providesTags: ['Reserve'],
    }),

    // Alerts endpoints
    getAlerts: builder.query<any[], string | void>({
      query: (userId) => ({
        url: '/alerts',
        params: userId ? { userId } : {},
      }),
      transformResponse: (response: ApiResponse<any[]>) => response.data || [],
      providesTags: ['Alert'],
    }),

    markAlertAsRead: builder.mutation<void, string>({
      query: (alertId) => ({
        url: `/alerts/${alertId}/read`,
        method: 'PATCH',
      }),
      invalidatesTags: ['Alert'],
    }),

    dismissAlert: builder.mutation<void, string>({
      query: (alertId) => ({
        url: `/alerts/${alertId}`,
        method: 'DELETE',
      }),
      invalidatesTags: ['Alert'],
    }),
  }),
});

// Export hooks for usage in functional components
export const {
  // System hooks
  useGetSystemOverviewQuery,
  usePauseSystemMutation,
  useResumeSystemMutation,
  
  // User hooks
  useGetCurrentUserQuery,
  
  // Integration hooks
  useExecuteBitcoinDepositMutation,
  useExecuteTokenWithdrawalMutation,
  useGetOperationHistoryQuery,
  useGetOperationStatusQuery,
  
  // KYC and Compliance hooks
  useGetKycStatusQuery,
  useSubmitKycDataMutation,
  useGetComplianceReportQuery,
  useCheckTransactionComplianceMutation,
  
  // Token hooks
  useGetTokenBalanceQuery,
  useGetTokenInfoQuery,
  useGetTransactionHistoryQuery,
  
  // Reserve hooks
  useGetReserveStatusQuery,
  useGetProofOfReservesQuery,
  useGetReserveHistoryQuery,
  
  // Alert hooks
  useGetAlertsQuery,
  useMarkAlertAsReadMutation,
  useDismissAlertMutation,
} = apiSlice;