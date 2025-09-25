import { useState, useEffect, useCallback } from 'react';
import { AxiosError } from 'axios';
import type { ApiResponse } from '@/types';

export interface UseApiState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  success: boolean;
}

export interface UseApiOptions {
  immediate?: boolean;
  onSuccess?: (data: any) => void;
  onError?: (error: string) => void;
}

/**
 * Generic hook for API calls with loading, error, and success states
 */
export function useApi<T = any>(
  apiFunction: (...args: any[]) => Promise<ApiResponse<T>>,
  options: UseApiOptions = {}
) {
  const { immediate = false, onSuccess, onError } = options;

  const [state, setState] = useState<UseApiState<T>>({
    data: null,
    loading: false,
    error: null,
    success: false,
  });

  const execute = useCallback(
    async (...args: any[]) => {
      setState(prev => ({ ...prev, loading: true, error: null, success: false }));

      try {
        const response = await apiFunction(...args);
        
        if (response.success) {
          setState({
            data: response.data || null,
            loading: false,
            error: null,
            success: true,
          });
          
          onSuccess?.(response.data);
          return response.data;
        } else {
          const errorMessage = response.error || 'API call failed';
          setState({
            data: null,
            loading: false,
            error: errorMessage,
            success: false,
          });
          
          onError?.(errorMessage);
          throw new Error(errorMessage);
        }
      } catch (error: any) {
        const errorMessage = error.response?.data?.message || error.message || 'Unknown error occurred';
        
        setState({
          data: null,
          loading: false,
          error: errorMessage,
          success: false,
        });
        
        onError?.(errorMessage);
        throw error;
      }
    },
    [apiFunction, onSuccess, onError]
  );

  const reset = useCallback(() => {
    setState({
      data: null,
      loading: false,
      error: null,
      success: false,
    });
  }, []);

  // Execute immediately if requested
  useEffect(() => {
    if (immediate) {
      execute();
    }
  }, [immediate, execute]);

  return {
    ...state,
    execute,
    reset,
  };
}

/**
 * Hook for API calls that need to be executed on component mount
 */
export function useApiOnMount<T = any>(
  apiFunction: (...args: any[]) => Promise<ApiResponse<T>>,
  args: any[] = [],
  options: Omit<UseApiOptions, 'immediate'> = {}
) {
  return useApi(apiFunction, { ...options, immediate: true });
}

/**
 * Hook for mutation operations (POST, PUT, DELETE)
 */
export function useMutation<T = any, P = any>(
  apiFunction: (params: P) => Promise<ApiResponse<T>>,
  options: UseApiOptions = {}
) {
  const { onSuccess, onError } = options;

  const [state, setState] = useState<UseApiState<T>>({
    data: null,
    loading: false,
    error: null,
    success: false,
  });

  const mutate = useCallback(
    async (params: P) => {
      setState(prev => ({ ...prev, loading: true, error: null, success: false }));

      try {
        const response = await apiFunction(params);
        
        if (response.success) {
          setState({
            data: response.data || null,
            loading: false,
            error: null,
            success: true,
          });
          
          onSuccess?.(response.data);
          return response.data;
        } else {
          const errorMessage = response.error || 'Mutation failed';
          setState({
            data: null,
            loading: false,
            error: errorMessage,
            success: false,
          });
          
          onError?.(errorMessage);
          throw new Error(errorMessage);
        }
      } catch (error: any) {
        const errorMessage = error.response?.data?.message || error.message || 'Unknown error occurred';
        
        setState({
          data: null,
          loading: false,
          error: errorMessage,
          success: false,
        });
        
        onError?.(errorMessage);
        throw error;
      }
    },
    [apiFunction, onSuccess, onError]
  );

  const reset = useCallback(() => {
    setState({
      data: null,
      loading: false,
      error: null,
      success: false,
    });
  }, []);

  return {
    ...state,
    mutate,
    reset,
  };
}

/**
 * Hook for paginated API calls
 */
export function usePaginatedApi<T = any>(
  apiFunction: (page: number, limit: number, ...args: any[]) => Promise<ApiResponse<{ items: T[]; total: number; page: number; limit: number }>>,
  initialPage = 1,
  initialLimit = 10,
  options: UseApiOptions = {}
) {
  const [page, setPage] = useState(initialPage);
  const [limit, setLimit] = useState(initialLimit);
  
  const api = useApi(
    useCallback(
      (...args: any[]) => apiFunction(page, limit, ...args),
      [apiFunction, page, limit]
    ),
    options
  );

  const nextPage = useCallback(() => {
    setPage(prev => prev + 1);
  }, []);

  const prevPage = useCallback(() => {
    setPage(prev => Math.max(1, prev - 1));
  }, []);

  const goToPage = useCallback((newPage: number) => {
    setPage(Math.max(1, newPage));
  }, []);

  const changeLimit = useCallback((newLimit: number) => {
    setLimit(newLimit);
    setPage(1); // Reset to first page when changing limit
  }, []);

  return {
    ...api,
    page,
    limit,
    nextPage,
    prevPage,
    goToPage,
    changeLimit,
    items: api.data?.items || [],
    total: api.data?.total || 0,
    totalPages: Math.ceil((api.data?.total || 0) / limit),
  };
}

export default useApi;