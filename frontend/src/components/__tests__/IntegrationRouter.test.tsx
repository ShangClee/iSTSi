import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { render, mockUser, createTestStore } from '@/test/utils';
import IntegrationRouter from '../IntegrationRouter';
import * as apiService from '@/services/api';

// Mock the API service
vi.mock('@/services/api', () => ({
  integrationApi: {
    executeBitcoinDeposit: vi.fn(),
    executeTokenWithdrawal: vi.fn(),
    getOperationStatus: vi.fn(),
    getOperationHistory: vi.fn(),
  },
}));

describe('IntegrationRouter Component', () => {
  const mockApiService = apiService as any;
  
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders integration router interface', () => {
    const store = createTestStore({
      auth: { user: mockUser, isAuthenticated: true },
    });

    render(<IntegrationRouter />, { store });

    expect(screen.getByText('Integration Router')).toBeInTheDocument();
    expect(screen.getByText('Bitcoin Deposit')).toBeInTheDocument();
    expect(screen.getByText('Token Withdrawal')).toBeInTheDocument();
  });

  it('handles Bitcoin deposit form submission', async () => {
    const user = userEvent.setup();
    const store = createTestStore({
      auth: { user: mockUser, isAuthenticated: true },
    });

    mockApiService.integrationApi.executeBitcoinDeposit.mockResolvedValue({
      data: {
        operationId: 'op-123',
        status: 'pending',
        txHash: 'stellar-tx-hash',
      },
    });

    render(<IntegrationRouter />, { store });

    // Fill out Bitcoin deposit form
    const btcAmountInput = screen.getByLabelText(/bitcoin amount/i);
    const btcTxHashInput = screen.getByLabelText(/bitcoin transaction hash/i);
    const confirmationsInput = screen.getByLabelText(/confirmations/i);

    await user.type(btcAmountInput, '1.5');
    await user.type(btcTxHashInput, 'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890');
    await user.type(confirmationsInput, '6');

    // Submit form
    const submitButton = screen.getByRole('button', { name: /execute deposit/i });
    await user.click(submitButton);

    // Verify API call
    await waitFor(() => {
      expect(mockApiService.integrationApi.executeBitcoinDeposit).toHaveBeenCalledWith({
        userAddress: mockUser.address,
        btcAmount: 150000000, // 1.5 BTC in satoshis
        btcTxHash: 'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
        confirmations: 6,
      });
    });

    // Check success message
    expect(screen.getByText(/deposit operation initiated/i)).toBeInTheDocument();
  });

  it('validates Bitcoin deposit form inputs', async () => {
    const user = userEvent.setup();
    const store = createTestStore({
      auth: { user: mockUser, isAuthenticated: true },
    });

    render(<IntegrationRouter />, { store });

    // Try to submit empty form
    const submitButton = screen.getByRole('button', { name: /execute deposit/i });
    await user.click(submitButton);

    // Check validation errors
    expect(screen.getByText(/bitcoin amount is required/i)).toBeInTheDocument();
    expect(screen.getByText(/transaction hash is required/i)).toBeInTheDocument();
    expect(screen.getByText(/confirmations must be at least 3/i)).toBeInTheDocument();
  });

  it('handles token withdrawal form submission', async () => {
    const user = userEvent.setup();
    const store = createTestStore({
      auth: { user: mockUser, isAuthenticated: true },
    });

    mockApiService.integrationApi.executeTokenWithdrawal.mockResolvedValue({
      data: {
        operationId: 'op-456',
        status: 'pending',
        btcTxId: 'btc-tx-id',
      },
    });

    render(<IntegrationRouter />, { store });

    // Switch to withdrawal tab
    const withdrawalTab = screen.getByRole('tab', { name: /token withdrawal/i });
    await user.click(withdrawalTab);

    // Fill out withdrawal form
    const tokenAmountInput = screen.getByLabelText(/token amount/i);
    const btcAddressInput = screen.getByLabelText(/bitcoin address/i);

    await user.type(tokenAmountInput, '0.75');
    await user.type(btcAddressInput, 'bc1qtest123456789abcdef');

    // Submit form
    const submitButton = screen.getByRole('button', { name: /execute withdrawal/i });
    await user.click(submitButton);

    // Verify API call
    await waitFor(() => {
      expect(mockApiService.integrationApi.executeTokenWithdrawal).toHaveBeenCalledWith({
        userAddress: mockUser.address,
        tokenAmount: 75000000, // 0.75 BTC worth in smallest units
        btcAddress: 'bc1qtest123456789abcdef',
      });
    });

    // Check success message
    expect(screen.getByText(/withdrawal operation initiated/i)).toBeInTheDocument();
  });

  it('validates Bitcoin address format', async () => {
    const user = userEvent.setup();
    const store = createTestStore({
      auth: { user: mockUser, isAuthenticated: true },
    });

    render(<IntegrationRouter />, { store });

    // Switch to withdrawal tab
    const withdrawalTab = screen.getByRole('tab', { name: /token withdrawal/i });
    await user.click(withdrawalTab);

    // Enter invalid Bitcoin address
    const btcAddressInput = screen.getByLabelText(/bitcoin address/i);
    await user.type(btcAddressInput, 'invalid-address');

    // Try to submit
    const submitButton = screen.getByRole('button', { name: /execute withdrawal/i });
    await user.click(submitButton);

    // Check validation error
    expect(screen.getByText(/invalid bitcoin address format/i)).toBeInTheDocument();
  });

  it('handles API errors gracefully', async () => {
    const user = userEvent.setup();
    const store = createTestStore({
      auth: { user: mockUser, isAuthenticated: true },
    });

    mockApiService.integrationApi.executeBitcoinDeposit.mockRejectedValue(
      new Error('Insufficient confirmations')
    );

    render(<IntegrationRouter />, { store });

    // Fill and submit form
    const btcAmountInput = screen.getByLabelText(/bitcoin amount/i);
    const btcTxHashInput = screen.getByLabelText(/bitcoin transaction hash/i);
    const confirmationsInput = screen.getByLabelText(/confirmations/i);

    await user.type(btcAmountInput, '1.0');
    await user.type(btcTxHashInput, 'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890');
    await user.type(confirmationsInput, '2');

    const submitButton = screen.getByRole('button', { name: /execute deposit/i });
    await user.click(submitButton);

    // Check error message
    await waitFor(() => {
      expect(screen.getByText(/insufficient confirmations/i)).toBeInTheDocument();
    });
  });

  it('shows operation status updates', async () => {
    const store = createTestStore({
      auth: { user: mockUser, isAuthenticated: true },
    });

    render(<IntegrationRouter />, { store });

    // Simulate operation status update via WebSocket
    store.dispatch({
      type: 'system/updateOperationStatus',
      payload: {
        operationId: 'op-123',
        status: 'completed',
        txHash: 'stellar-tx-hash',
      },
    });

    await waitFor(() => {
      expect(screen.getByText(/operation op-123 completed/i)).toBeInTheDocument();
    });
  });

  it('requires user authentication', () => {
    const store = createTestStore({
      auth: { user: null, isAuthenticated: false },
    });

    render(<IntegrationRouter />, { store });

    expect(screen.getByText(/please log in to access integration features/i)).toBeInTheDocument();
  });

  it('checks KYC status before allowing operations', async () => {
    const unverifiedUser = {
      ...mockUser,
      kycStatus: 'pending' as const,
    };

    const store = createTestStore({
      auth: { user: unverifiedUser, isAuthenticated: true },
    });

    render(<IntegrationRouter />, { store });

    expect(screen.getByText(/kyc verification required/i)).toBeInTheDocument();
    
    // Form should be disabled
    const submitButton = screen.getByRole('button', { name: /execute deposit/i });
    expect(submitButton).toBeDisabled();
  });

  it('displays operation history', async () => {
    const mockOperations = [
      {
        id: 'op-1',
        type: 'bitcoin_deposit',
        status: 'completed',
        amount: '100000000',
        createdAt: '2024-01-01T10:00:00Z',
      },
      {
        id: 'op-2',
        type: 'token_withdrawal',
        status: 'pending',
        amount: '50000000',
        createdAt: '2024-01-01T11:00:00Z',
      },
    ];

    mockApiService.integrationApi.getOperationHistory.mockResolvedValue({
      data: mockOperations,
    });

    const store = createTestStore({
      auth: { user: mockUser, isAuthenticated: true },
    });

    render(<IntegrationRouter />, { store });

    // Switch to history tab
    const historyTab = screen.getByRole('tab', { name: /operation history/i });
    await userEvent.setup().click(historyTab);

    await waitFor(() => {
      expect(screen.getByText('op-1')).toBeInTheDocument();
      expect(screen.getByText('op-2')).toBeInTheDocument();
      expect(screen.getByText('Bitcoin Deposit')).toBeInTheDocument();
      expect(screen.getByText('Token Withdrawal')).toBeInTheDocument();
    });
  });
});