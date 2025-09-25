import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

// Utility function for combining Tailwind classes
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// Format currency values
export function formatCurrency(amount: number, currency = 'USD'): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency,
  }).format(amount);
}

// Format Bitcoin amounts
export function formatBitcoin(amount: number): string {
  return `${amount.toFixed(8)} BTC`;
}

// Format percentage values
export function formatPercentage(value: number, decimals = 2): string {
  return `${value.toFixed(decimals)}%`;
}

// Format timestamps
export function formatTimestamp(timestamp: string): string {
  return new Date(timestamp).toLocaleString();
}

// Validate Bitcoin address (basic validation)
export function isValidBitcoinAddress(address: string): boolean {
  // Basic regex for Bitcoin address validation
  const btcRegex = /^[13][a-km-zA-HJ-NP-Z1-9]{25,34}$|^bc1[a-z0-9]{39,59}$/;
  return btcRegex.test(address);
}

// Generate unique IDs
export function generateId(): string {
  return Math.random().toString(36).substr(2, 9);
}

// Debounce function for search inputs
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout;
  return (...args: Parameters<T>) => {
    clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
}