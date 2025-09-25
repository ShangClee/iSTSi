import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { render, mockSystemOverview, createTestStore } from '@/test/utils';
import SystemOverview from '../SystemOverview';
import * as apiService from '@/services/api';

// Mock the API service
vi.mock('@/services/api', () => ({
  systemApi: {
    getOverview: vi.fn(),
    getReserveStatus: vi.fn(),
    getSystemHealth: vi.fn(),
  },
}));

describe('SystemOverview Component', () => {
  const mockApiService = apiService as any;
  
  beforeEach(() => {
    vi.clearAllMocks();
    mockApiService.systemApi.getOverview.mockResolvedValue({
      data: mockSystemOverview,
    });
  });

  it('renders system overview data correctly', async () => {
    render(<SystemOverview />);

    // Check loading state initially
    expect(screen.getByText(/loading/i)).toBeInTheDocument();

    // Wait for data to load
    await waitFor(() => {
      expect(screen.getByText('System Overview')).toBeInTheDocument();
    });

    // Check that system data is displayed
    expect(screen.getByText('10.00 BTC')).toBeInTheDocument(); // Total reserves
    expect(screen.getByText('9.50 BTC')).toBeInTheDocument(); // Total tokens issued
    expect(screen.getByText('105.26%')).toBeInTheDocument(); // Reserve ratio
    expect(screen.getByText('150')).toBeInTheDocument(); // Active users
    expect(screen.getByText('1,250')).toBeInTheDocument(); // Total transactions
  });

  it('displays correct system status indicator', async () => {
    render(<SystemOverview />);

    await waitFor(() => {
      expect(screen.getByText('Operational')).toBeInTheDocument();
    });

    // Check for status indicator color/styling
    const statusElement = screen.getByText('Operational');
    expect(statusElement).toHaveClass('text-green-600'); // Assuming green for operational
  });

  it('handles API error gracefully', async () => {
    mockApiService.systemApi.getOverview.mockRejectedValue(
      new Error('API Error')
    );

    render(<SystemOverview />);

    await waitFor(() => {
      expect(screen.getByText(/error loading system data/i)).toBeInTheDocument();
    });
  });

  it('refreshes data when refresh button is clicked', async () => {
    const user = userEvent.setup();
    render(<SystemOverview />);

    await waitFor(() => {
      expect(screen.getByText('System Overview')).toBeInTheDocument();
    });

    // Find and click refresh button
    const refreshButton = screen.getByRole('button', { name: /refresh/i });
    await user.click(refreshButton);

    // Verify API was called again
    expect(mockApiService.systemApi.getOverview).toHaveBeenCalledTimes(2);
  });

  it('shows warning when reserve ratio is low', async () => {
    const lowReserveData = {
      ...mockSystemOverview,
      reserveRatio: 95.5, // Below 100%
    };

    mockApiService.systemApi.getOverview.mockResolvedValue({
      data: lowReserveData,
    });

    render(<SystemOverview />);

    await waitFor(() => {
      expect(screen.getByText('95.50%')).toBeInTheDocument();
    });

    // Check for warning indicator
    expect(screen.getByText(/reserve ratio below 100%/i)).toBeInTheDocument();
  });

  it('updates data automatically at regular intervals', async () => {
    vi.useFakeTimers();
    
    render(<SystemOverview />);

    await waitFor(() => {
      expect(mockApiService.systemApi.getOverview).toHaveBeenCalledTimes(1);
    });

    // Fast-forward 30 seconds (assuming 30s refresh interval)
    vi.advanceTimersByTime(30000);

    await waitFor(() => {
      expect(mockApiService.systemApi.getOverview).toHaveBeenCalledTimes(2);
    });

    vi.useRealTimers();
  });

  it('displays formatted numbers correctly', async () => {
    const largeNumberData = {
      ...mockSystemOverview,
      totalReserves: '123456789000000', // 1,234,567.89 BTC
      activeUsers: 12345,
      totalTransactions: 987654,
    };

    mockApiService.systemApi.getOverview.mockResolvedValue({
      data: largeNumberData,
    });

    render(<SystemOverview />);

    await waitFor(() => {
      expect(screen.getByText('1,234,567.89 BTC')).toBeInTheDocument();
      expect(screen.getByText('12,345')).toBeInTheDocument();
      expect(screen.getByText('987,654')).toBeInTheDocument();
    });
  });

  it('handles real-time updates via WebSocket', async () => {
    const store = createTestStore();
    render(<SystemOverview />, { store });

    // Simulate WebSocket update
    const updatedData = {
      ...mockSystemOverview,
      totalReserves: '1100000000', // 11 BTC
      reserveRatio: 110.0,
    };

    // Dispatch WebSocket update action
    store.dispatch({
      type: 'system/updateOverview',
      payload: updatedData,
    });

    await waitFor(() => {
      expect(screen.getByText('11.00 BTC')).toBeInTheDocument();
      expect(screen.getByText('110.00%')).toBeInTheDocument();
    });
  });
});