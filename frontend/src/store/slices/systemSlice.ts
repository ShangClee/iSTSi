import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit';
import { systemApi } from '@/services/api';
import type { SystemState } from '@/types';

interface SystemSliceState {
  systemState: SystemState | null;
  isLoading: boolean;
  error: string | null;
  lastUpdated: string | null;
}

const initialState: SystemSliceState = {
  systemState: null,
  isLoading: false,
  error: null,
  lastUpdated: null,
};

// Async thunks
export const fetchSystemOverview = createAsyncThunk(
  'system/fetchOverview',
  async (_, { rejectWithValue }) => {
    try {
      const response = await systemApi.getOverview();
      if (response.success) {
        return response.data;
      } else {
        return rejectWithValue(response.error || 'Failed to fetch system overview');
      }
    } catch (error: any) {
      return rejectWithValue(error.message || 'Failed to fetch system overview');
    }
  }
);

export const pauseSystem = createAsyncThunk(
  'system/pause',
  async (_, { rejectWithValue }) => {
    try {
      const response = await systemApi.pauseSystem();
      if (response.success) {
        return 'paused';
      } else {
        return rejectWithValue(response.error || 'Failed to pause system');
      }
    } catch (error: any) {
      return rejectWithValue(error.message || 'Failed to pause system');
    }
  }
);

export const resumeSystem = createAsyncThunk(
  'system/resume',
  async (_, { rejectWithValue }) => {
    try {
      const response = await systemApi.resumeSystem();
      if (response.success) {
        return 'operational';
      } else {
        return rejectWithValue(response.error || 'Failed to resume system');
      }
    } catch (error: any) {
      return rejectWithValue(error.message || 'Failed to resume system');
    }
  }
);

// System slice
const systemSlice = createSlice({
  name: 'system',
  initialState,
  reducers: {
    updateSystemState: (state, action: PayloadAction<Partial<SystemState>>) => {
      if (state.systemState) {
        state.systemState = { ...state.systemState, ...action.payload };
        state.lastUpdated = new Date().toISOString();
      }
    },
    
    setSystemStatus: (state, action: PayloadAction<SystemState['status']>) => {
      if (state.systemState) {
        state.systemState.status = action.payload;
        state.lastUpdated = new Date().toISOString();
      }
    },
    
    incrementOperations: (state, action: PayloadAction<number>) => {
      if (state.systemState) {
        state.systemState.totalOperations += action.payload;
        state.lastUpdated = new Date().toISOString();
      }
    },
    
    updateSuccessRate: (state, action: PayloadAction<number>) => {
      if (state.systemState) {
        state.systemState.successRate = action.payload;
        state.lastUpdated = new Date().toISOString();
      }
    },
    
    updateReserveRatio: (state, action: PayloadAction<number>) => {
      if (state.systemState) {
        state.systemState.reserveRatio = action.payload;
        state.lastUpdated = new Date().toISOString();
      }
    },
    
    updateActiveUsers: (state, action: PayloadAction<number>) => {
      if (state.systemState) {
        state.systemState.activeUsers = action.payload;
        state.lastUpdated = new Date().toISOString();
      }
    },
    
    clearError: (state) => {
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    // Fetch system overview
    builder
      .addCase(fetchSystemOverview.pending, (state) => {
        state.isLoading = true;
        state.error = null;
      })
      .addCase(fetchSystemOverview.fulfilled, (state, action) => {
        state.isLoading = false;
        state.systemState = action.payload;
        state.lastUpdated = new Date().toISOString();
      })
      .addCase(fetchSystemOverview.rejected, (state, action) => {
        state.isLoading = false;
        state.error = action.payload as string;
      });

    // Pause system
    builder
      .addCase(pauseSystem.pending, (state) => {
        state.isLoading = true;
        state.error = null;
      })
      .addCase(pauseSystem.fulfilled, (state, action) => {
        state.isLoading = false;
        if (state.systemState) {
          state.systemState.status = action.payload as SystemState['status'];
          state.lastUpdated = new Date().toISOString();
        }
      })
      .addCase(pauseSystem.rejected, (state, action) => {
        state.isLoading = false;
        state.error = action.payload as string;
      });

    // Resume system
    builder
      .addCase(resumeSystem.pending, (state) => {
        state.isLoading = true;
        state.error = null;
      })
      .addCase(resumeSystem.fulfilled, (state, action) => {
        state.isLoading = false;
        if (state.systemState) {
          state.systemState.status = action.payload as SystemState['status'];
          state.lastUpdated = new Date().toISOString();
        }
      })
      .addCase(resumeSystem.rejected, (state, action) => {
        state.isLoading = false;
        state.error = action.payload as string;
      });
  },
});

export const {
  updateSystemState,
  setSystemStatus,
  incrementOperations,
  updateSuccessRate,
  updateReserveRatio,
  updateActiveUsers,
  clearError,
} = systemSlice.actions;

export default systemSlice.reducer;