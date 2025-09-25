import { useState, useEffect, useCallback } from 'react';
import { authService, type LoginCredentials, type RegisterData } from '@/services/auth';
import type { AuthState } from '@/types';

/**
 * Hook for managing authentication state and operations
 */
export function useAuth() {
  const [authState, setAuthState] = useState<AuthState>(authService.getAuthState());

  // Subscribe to auth state changes
  useEffect(() => {
    const unsubscribe = authService.subscribe(setAuthState);
    return unsubscribe;
  }, []);

  const login = useCallback(async (credentials: LoginCredentials) => {
    return await authService.login(credentials);
  }, []);

  const register = useCallback(async (userData: RegisterData) => {
    return await authService.register(userData);
  }, []);

  const logout = useCallback(async () => {
    await authService.logout();
  }, []);

  const refreshToken = useCallback(async () => {
    return await authService.refreshToken();
  }, []);

  const getCurrentUser = useCallback(async () => {
    return await authService.getCurrentUser();
  }, []);

  return {
    // State
    user: authState.user,
    token: authState.token,
    isAuthenticated: authState.isAuthenticated,
    isLoading: authState.isLoading,
    
    // Actions
    login,
    register,
    logout,
    refreshToken,
    getCurrentUser,
  };
}

/**
 * Hook for protecting routes that require authentication
 */
export function useRequireAuth(redirectTo?: string) {
  const { isAuthenticated, isLoading } = useAuth();

  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      if (redirectTo) {
        window.location.href = redirectTo;
      } else {
        // Default redirect to login
        window.location.href = '/login';
      }
    }
  }, [isAuthenticated, isLoading, redirectTo]);

  return { isAuthenticated, isLoading };
}

/**
 * Hook for checking user permissions/roles
 */
export function usePermissions() {
  const { user } = useAuth();

  const hasRole = useCallback((role: string) => {
    return user?.role === role;
  }, [user]);

  const hasAnyRole = useCallback((roles: string[]) => {
    return user?.role ? roles.includes(user.role) : false;
  }, [user]);

  const isAdmin = useCallback(() => {
    return hasRole('admin');
  }, [hasRole]);

  const isOperator = useCallback(() => {
    return hasAnyRole(['admin', 'operator']);
  }, [hasAnyRole]);

  const canManageSystem = useCallback(() => {
    return hasAnyRole(['admin', 'operator']);
  }, [hasAnyRole]);

  const canViewReports = useCallback(() => {
    return hasAnyRole(['admin', 'operator', 'auditor']);
  }, [hasAnyRole]);

  return {
    user,
    hasRole,
    hasAnyRole,
    isAdmin,
    isOperator,
    canManageSystem,
    canViewReports,
  };
}

/**
 * Hook for login form state management
 */
export function useLoginForm() {
  const [credentials, setCredentials] = useState<LoginCredentials>({
    email: '',
    password: '',
  });
  const [errors, setErrors] = useState<Partial<LoginCredentials>>({});
  const { login, isLoading } = useAuth();

  const updateCredentials = useCallback((field: keyof LoginCredentials, value: string) => {
    setCredentials(prev => ({ ...prev, [field]: value }));
    
    // Clear error when user starts typing
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: undefined }));
    }
  }, [errors]);

  const validateForm = useCallback(() => {
    const newErrors: Partial<LoginCredentials> = {};

    if (!credentials.email) {
      newErrors.email = 'Email is required';
    } else if (!/\S+@\S+\.\S+/.test(credentials.email)) {
      newErrors.email = 'Email is invalid';
    }

    if (!credentials.password) {
      newErrors.password = 'Password is required';
    } else if (credentials.password.length < 6) {
      newErrors.password = 'Password must be at least 6 characters';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  }, [credentials]);

  const handleSubmit = useCallback(async () => {
    if (!validateForm()) {
      return { success: false, error: 'Please fix the form errors' };
    }

    return await login(credentials);
  }, [credentials, validateForm, login]);

  const reset = useCallback(() => {
    setCredentials({ email: '', password: '' });
    setErrors({});
  }, []);

  return {
    credentials,
    errors,
    isLoading,
    updateCredentials,
    handleSubmit,
    reset,
  };
}

/**
 * Hook for registration form state management
 */
export function useRegisterForm() {
  const [userData, setUserData] = useState<RegisterData>({
    email: '',
    password: '',
    username: '',
  });
  const [confirmPassword, setConfirmPassword] = useState('');
  const [errors, setErrors] = useState<Partial<RegisterData & { confirmPassword: string }>>({});
  const { register, isLoading } = useAuth();

  const updateUserData = useCallback((field: keyof RegisterData, value: string) => {
    setUserData(prev => ({ ...prev, [field]: value }));
    
    // Clear error when user starts typing
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: undefined }));
    }
  }, [errors]);

  const updateConfirmPassword = useCallback((value: string) => {
    setConfirmPassword(value);
    
    if (errors.confirmPassword) {
      setErrors(prev => ({ ...prev, confirmPassword: undefined }));
    }
  }, [errors]);

  const validateForm = useCallback(() => {
    const newErrors: Partial<RegisterData & { confirmPassword: string }> = {};

    if (!userData.email) {
      newErrors.email = 'Email is required';
    } else if (!/\S+@\S+\.\S+/.test(userData.email)) {
      newErrors.email = 'Email is invalid';
    }

    if (!userData.username) {
      newErrors.username = 'Username is required';
    } else if (userData.username.length < 3) {
      newErrors.username = 'Username must be at least 3 characters';
    }

    if (!userData.password) {
      newErrors.password = 'Password is required';
    } else if (userData.password.length < 8) {
      newErrors.password = 'Password must be at least 8 characters';
    }

    if (!confirmPassword) {
      newErrors.confirmPassword = 'Please confirm your password';
    } else if (userData.password !== confirmPassword) {
      newErrors.confirmPassword = 'Passwords do not match';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  }, [userData, confirmPassword]);

  const handleSubmit = useCallback(async () => {
    if (!validateForm()) {
      return { success: false, error: 'Please fix the form errors' };
    }

    return await register(userData);
  }, [userData, validateForm, register]);

  const reset = useCallback(() => {
    setUserData({ email: '', password: '', username: '' });
    setConfirmPassword('');
    setErrors({});
  }, []);

  return {
    userData,
    confirmPassword,
    errors,
    isLoading,
    updateUserData,
    updateConfirmPassword,
    handleSubmit,
    reset,
  };
}

export default useAuth;