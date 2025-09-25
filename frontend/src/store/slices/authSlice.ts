import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit';
import { authService, type LoginCredentials, type RegisterData } from '@/services/auth';
import type { AuthState, User } from '@/types';

// Initial state
const initialState: AuthState = {
  user: null,
  token: null,
  isAuthenticated: false,
  isLoading: false,
};

// Async thunks
export const loginAsync = createAsyncThunk(
  'auth/login',
  async (credentials: LoginCredentials, { rejectWithValue }) => {
    try {
      const result = await authService.login(credentials);
      if (result.success) {
        return authService.getAuthState();
      } else {
        return rejectWithValue(result.error || 'Login failed');
      }
    } catch (error: any) {
      return rejectWithValue(error.message || 'Login failed');
    }
  }
);

export const registerAsync = createAsyncThunk(
  'auth/register',
  async (userData: RegisterData, { rejectWithValue }) => {
    try {
      const result = await authService.register(userData);
      if (result.success) {
        return authService.getAuthState();
      } else {
        return rejectWithValue(result.error || 'Registration failed');
      }
    } catch (error: any) {
      return rejectWithValue(error.message || 'Registration failed');
    }
  }
);

export const logoutAsync = createAsyncThunk(
  'auth/logout',
  async (_, { rejectWithValue }) => {
    try {
      await authService.logout();
      return;
    } catch (error: any) {
      return rejectWithValue(error.message || 'Logout failed');
    }
  }
);

export const refreshTokenAsync = createAsyncThunk(
  'auth/refreshToken',
  async (_, { rejectWithValue }) => {
    try {
      const success = await authService.refreshToken();
      if (success) {
        return authService.getAuthState();
      } else {
        return rejectWithValue('Token refresh failed');
      }
    } catch (error: any) {
      return rejectWithValue(error.message || 'Token refresh failed');
    }
  }
);

export const getCurrentUserAsync = createAsyncThunk(
  'auth/getCurrentUser',
  async (_, { rejectWithValue }) => {
    try {
      const user = await authService.getCurrentUser();
      return user;
    } catch (error: any) {
      return rejectWithValue(error.message || 'Failed to get current user');
    }
  }
);

// Auth slice
const authSlice = createSlice({
  name: 'auth',
  initialState,
  reducers: {
    // Sync actions
    setAuthState: (state, action: PayloadAction<AuthState>) => {
      return { ...action.payload };
    },
    
    setUser: (state, action: PayloadAction<User | null>) => {
      state.user = action.payload;
    },
    
    setToken: (state, action: PayloadAction<string | null>) => {
      state.token = action.payload;
    },
    
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload;
    },
    
    clearAuth: (state) => {
      state.user = null;
      state.token = null;
      state.isAuthenticated = false;
      state.isLoading = false;
    },
  },
  extraReducers: (builder) => {
    // Login
    builder
      .addCase(loginAsync.pending, (state) => {
        state.isLoading = true;
      })
      .addCase(loginAsync.fulfilled, (state, action) => {
        return { ...action.payload };
      })
      .addCase(loginAsync.rejected, (state) => {
        state.isLoading = false;
        state.isAuthenticated = false;
      });

    // Register
    builder
      .addCase(registerAsync.pending, (state) => {
        state.isLoading = true;
      })
      .addCase(registerAsync.fulfilled, (state, action) => {
        return { ...action.payload };
      })
      .addCase(registerAsync.rejected, (state) => {
        state.isLoading = false;
        state.isAuthenticated = false;
      });

    // Logout
    builder
      .addCase(logoutAsync.pending, (state) => {
        state.isLoading = true;
      })
      .addCase(logoutAsync.fulfilled, (state) => {
        state.user = null;
        state.token = null;
        state.isAuthenticated = false;
        state.isLoading = false;
      })
      .addCase(logoutAsync.rejected, (state) => {
        state.isLoading = false;
      });

    // Refresh token
    builder
      .addCase(refreshTokenAsync.fulfilled, (state, action) => {
        return { ...action.payload };
      })
      .addCase(refreshTokenAsync.rejected, (state) => {
        state.user = null;
        state.token = null;
        state.isAuthenticated = false;
      });

    // Get current user
    builder
      .addCase(getCurrentUserAsync.fulfilled, (state, action) => {
        state.user = action.payload;
      });
  },
});

export const {
  setAuthState,
  setUser,
  setToken,
  setLoading,
  clearAuth,
} = authSlice.actions;

export default authSlice.reducer;