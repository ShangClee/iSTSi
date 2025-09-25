import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit';
import { alertsApi } from '@/services/api';
import type { Alert } from '@/types';

interface AlertsSliceState {
  alerts: Alert[];
  isLoading: boolean;
  error: string | null;
  unreadCount: number;
}

const initialState: AlertsSliceState = {
  alerts: [],
  isLoading: false,
  error: null,
  unreadCount: 0,
};

// Async thunks
export const fetchAlerts = createAsyncThunk(
  'alerts/fetchAlerts',
  async (userId?: string, { rejectWithValue }) => {
    try {
      const response = await alertsApi.getAlerts(userId);
      if (response.success) {
        return response.data || [];
      } else {
        return rejectWithValue(response.error || 'Failed to fetch alerts');
      }
    } catch (error: any) {
      return rejectWithValue(error.message || 'Failed to fetch alerts');
    }
  }
);

export const markAlertAsRead = createAsyncThunk(
  'alerts/markAsRead',
  async (alertId: string, { rejectWithValue }) => {
    try {
      const response = await alertsApi.markAlertAsRead(alertId);
      if (response.success) {
        return alertId;
      } else {
        return rejectWithValue(response.error || 'Failed to mark alert as read');
      }
    } catch (error: any) {
      return rejectWithValue(error.message || 'Failed to mark alert as read');
    }
  }
);

export const dismissAlert = createAsyncThunk(
  'alerts/dismiss',
  async (alertId: string, { rejectWithValue }) => {
    try {
      const response = await alertsApi.dismissAlert(alertId);
      if (response.success) {
        return alertId;
      } else {
        return rejectWithValue(response.error || 'Failed to dismiss alert');
      }
    } catch (error: any) {
      return rejectWithValue(error.message || 'Failed to dismiss alert');
    }
  }
);

// Alerts slice
const alertsSlice = createSlice({
  name: 'alerts',
  initialState,
  reducers: {
    addAlert: (state, action: PayloadAction<Alert>) => {
      state.alerts.unshift(action.payload);
      state.unreadCount += 1;
    },
    
    removeAlert: (state, action: PayloadAction<string>) => {
      const index = state.alerts.findIndex(alert => alert.id === action.payload);
      if (index !== -1) {
        state.alerts.splice(index, 1);
        state.unreadCount = Math.max(0, state.unreadCount - 1);
      }
    },
    
    markAsRead: (state, action: PayloadAction<string>) => {
      const alert = state.alerts.find(alert => alert.id === action.payload);
      if (alert && !alert.read) {
        alert.read = true;
        state.unreadCount = Math.max(0, state.unreadCount - 1);
      }
    },
    
    markAllAsRead: (state) => {
      state.alerts.forEach(alert => {
        alert.read = true;
      });
      state.unreadCount = 0;
    },
    
    clearAllAlerts: (state) => {
      state.alerts = [];
      state.unreadCount = 0;
    },
    
    updateUnreadCount: (state) => {
      state.unreadCount = state.alerts.filter(alert => !alert.read).length;
    },
    
    clearError: (state) => {
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    // Fetch alerts
    builder
      .addCase(fetchAlerts.pending, (state) => {
        state.isLoading = true;
        state.error = null;
      })
      .addCase(fetchAlerts.fulfilled, (state, action) => {
        state.isLoading = false;
        state.alerts = action.payload;
        state.unreadCount = action.payload.filter((alert: Alert) => !alert.read).length;
      })
      .addCase(fetchAlerts.rejected, (state, action) => {
        state.isLoading = false;
        state.error = action.payload as string;
      });

    // Mark alert as read
    builder
      .addCase(markAlertAsRead.fulfilled, (state, action) => {
        const alert = state.alerts.find(alert => alert.id === action.payload);
        if (alert && !alert.read) {
          alert.read = true;
          state.unreadCount = Math.max(0, state.unreadCount - 1);
        }
      })
      .addCase(markAlertAsRead.rejected, (state, action) => {
        state.error = action.payload as string;
      });

    // Dismiss alert
    builder
      .addCase(dismissAlert.fulfilled, (state, action) => {
        const index = state.alerts.findIndex(alert => alert.id === action.payload);
        if (index !== -1) {
          const wasUnread = !state.alerts[index].read;
          state.alerts.splice(index, 1);
          if (wasUnread) {
            state.unreadCount = Math.max(0, state.unreadCount - 1);
          }
        }
      })
      .addCase(dismissAlert.rejected, (state, action) => {
        state.error = action.payload as string;
      });
  },
});

export const {
  addAlert,
  removeAlert,
  markAsRead,
  markAllAsRead,
  clearAllAlerts,
  updateUnreadCount,
  clearError,
} = alertsSlice.actions;

export default alertsSlice.reducer;