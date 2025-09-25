import React, { ReactElement } from 'react';
import { render, RenderOptions } from '@testing-library/react';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import authSlice from '@/store/slices/authSlice';
import systemSlice from '@/store/slices/systemSlice';
import alertsSlice from '@/store/slices/alertsSlice';
import { apiSlice } from '@/store/slices/apiSlice';

// Create a test store factory
export const createTestStore = (preloadedState?: any) => {
  return configureStore({
    reducer: {
      auth: authSlice,
      system: systemSlice,
      alerts: alertsSlice,
      api: apiSlice.reducer,
    },
    preloadedState,
    middleware: (getDefaultMiddleware) =>
      getDefaultMiddleware({
        serializableCheck: {
          ignoredActions: ['persist/PERSIST'],
        },
      }),
  });
};

// Test wrapper component
interface AllTheProvidersProps {
  children: React.ReactNode;
  store?: ReturnType<typeof createTestStore>;
}

const AllTheProviders: React.FC<AllTheProvidersProps> = ({ 
  children, 
  store = createTestStore() 
}) => {
  return (
    <Provider store={store}>
      {children}
    </Provider>
  );
};

// Custom render function
interface CustomRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  preloadedState?: any;
  store?: ReturnType<typeof createTestStore>;
}

const customRender = (
  ui: ReactElement,
  {
    preloadedState,
    store = createTestStore(preloadedState),
    ...renderOptions
  }: CustomRenderOptions = {}
) => {
  const Wrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
    <AllTheProviders store={store}>{children}</AllTheProviders>
  );

  return {
    store,
    ...render(ui, { wrapper: Wrapper, ...renderOptions }),
  };
};

// Mock data factories
export const mockUser = {
  id: 'test-user-id',
  email: 'test@example.com',
  address: 'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA',
  kycStatus: 'approved' as const,
  tier: 1,
  createdAt: '2024-01-01T00:00:00Z',
};

export const mockSystemOverview = {
  totalReserves: '1000000000', // 10 BTC
  totalTokensIssued: '950000000', // 9.5 BTC worth
  reserveRatio: 105.26,
  activeUsers: 150,
  totalTransactions: 1250,
  systemStatus: 'operational' as const,
  lastUpdated: '2024-01-01T12:00:00Z',
};

export const mockOperation = {
  id: 'op-123',
  type: 'bitcoin_deposit' as const,
  status: 'completed' as const,
  userAddress: 'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA',
  amount: '100000000', // 1 BTC
  btcTxHash: 'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
  createdAt: '2024-01-01T10:00:00Z',
  completedAt: '2024-01-01T10:05:00Z',
};

export const mockAlert = {
  id: 'alert-123',
  type: 'warning' as const,
  title: 'Reserve Threshold Alert',
  message: 'Reserve ratio has dropped below 100%',
  timestamp: '2024-01-01T11:00:00Z',
  acknowledged: false,
};

// Re-export everything
export * from '@testing-library/react';
export { customRender as render };