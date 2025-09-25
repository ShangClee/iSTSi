# Frontend-Backend Integration Guide

This document describes the integration between the React frontend and the Loco.rs backend.

## Overview

The frontend has been updated to communicate with the Loco.rs backend running on `http://localhost:8080`. The integration includes:

- **HTTP API Client**: Axios-based client with retry logic and error handling
- **WebSocket Client**: Native WebSocket client for real-time updates
- **Authentication Service**: JWT-based authentication with automatic token refresh
- **Connection Monitoring**: Health checks and connection status monitoring

## Services

### API Client (`src/services/api.ts`)

The API client provides methods to communicate with all backend endpoints:

```typescript
import { api } from '@/services';

// Authentication
const loginResult = await api.auth.login('user@example.com', 'password');

// System information
const systemOverview = await api.system.getOverview();

// Integration operations
const depositResult = await api.integration.executeBitcoinDeposit({
  amount: 0.001,
  address: 'bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
  userId: 'user-id'
});
```

### WebSocket Client (`src/services/websocket.ts`)

Real-time communication with the backend:

```typescript
import { initializeWebSocket } from '@/services';

const wsClient = initializeWebSocket(token, {
  onConnect: () => console.log('Connected'),
  onSystemUpdate: (data) => console.log('System update:', data),
  onOperationUpdate: (data) => console.log('Operation update:', data),
});

// Subscribe to updates
wsClient.subscribeToSystemUpdates();
wsClient.subscribeToUserUpdates(userId);
```

### Authentication Service (`src/services/auth.ts`)

Handles user authentication and session management:

```typescript
import { authService } from '@/services';

// Login
const result = await authService.login({
  email: 'user@example.com',
  password: 'password'
});

// Get current user
const user = authService.getCurrentUserSync();

// Check authentication status
const isAuth = authService.isAuthenticated();
```

### Connection Monitoring (`src/services/connection.ts`)

Monitor backend connectivity:

```typescript
import { getConnectionStatus, startConnectionMonitoring } from '@/services';

// Get current status
const status = await getConnectionStatus();

// Start monitoring
const monitor = startConnectionMonitoring();
monitor.subscribe((status) => {
  console.log('Connection status:', status.overall);
});
```

## Configuration

### Environment Variables

Create a `.env.development` file:

```env
# API Configuration
VITE_API_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8080

# Feature Flags
VITE_ENABLE_DEBUG=true
VITE_ENABLE_CONNECTION_MONITORING=true

# API Configuration
VITE_API_TIMEOUT=15000
VITE_API_RETRY_ATTEMPTS=3
VITE_API_RETRY_DELAY=1000

# WebSocket Configuration
VITE_WS_RECONNECT_INTERVAL=5000
VITE_WS_MAX_RECONNECT_ATTEMPTS=10
VITE_WS_HEARTBEAT_INTERVAL=30000
```

### Vite Proxy Configuration

The Vite development server is configured to proxy API requests:

```typescript
// vite.config.ts
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:8080',
      changeOrigin: true,
    },
    '/ws': {
      target: 'ws://localhost:8080',
      ws: true,
    },
  },
}
```

## Backend Endpoints

The frontend expects these endpoints from the Loco.rs backend:

### Authentication
- `POST /api/auth/login` - User login
- `POST /api/auth/register` - User registration
- `POST /api/auth/logout` - User logout
- `GET /api/auth/me` - Get current user

### System
- `GET /api/system/health` - Health check
- `GET /api/system/version` - Version information
- `GET /api/integration/system-overview` - System overview
- `GET /api/integration/status` - Integration status

### Integration Operations
- `POST /api/integration/bitcoin-deposit` - Execute Bitcoin deposit
- `POST /api/integration/token-withdrawal` - Execute token withdrawal
- `POST /api/integration/cross-token-exchange` - Cross-token exchange
- `GET /api/integration/events` - Get contract events
- `POST /api/integration/transaction-status` - Get transaction status

### User Management
- `GET /api/users/{id}` - Get user profile
- `PUT /api/users/{id}` - Update user profile
- `GET /api/users/{id}/operations` - Get user operations

### KYC and Compliance
- `GET /api/kyc/status/{userId}` - Get KYC status
- `POST /api/kyc/submit/{userId}` - Submit KYC data
- `POST /api/kyc/check-compliance` - Check compliance

### Tokens and Reserves
- `GET /api/tokens/balance/{userId}` - Get token balance
- `GET /api/tokens/info` - Get token information
- `GET /api/reserves/status` - Get reserve status
- `GET /api/reserves/proof` - Get proof of reserves

## Testing

### Connection Test

Run the connection test to verify backend communication:

```bash
npm run test:connection
```

This will test:
- API endpoint connectivity
- WebSocket connection
- Authentication flow
- Integration status

### Manual Testing

```typescript
import { logConnectionTest } from '@/utils/connectionTest';

// Run in browser console
await logConnectionTest();
```

## Error Handling

The API client includes comprehensive error handling:

- **Network Errors**: Automatic retry with exponential backoff
- **Authentication Errors**: Automatic logout and redirect
- **Server Errors**: Proper error messages and logging
- **Timeout Handling**: Configurable request timeouts

## WebSocket Features

- **Automatic Reconnection**: Reconnects on connection loss
- **Heartbeat**: Keeps connection alive
- **Subscription Management**: Subscribe/unsubscribe from channels
- **Error Recovery**: Handles connection errors gracefully

## Development Workflow

1. **Start Backend**: Ensure Loco.rs backend is running on port 8080
2. **Start Frontend**: Run `npm run dev` to start the development server
3. **Test Connection**: Run `npm run test:connection` to verify integration
4. **Monitor Logs**: Check browser console for connection status

## Troubleshooting

### Common Issues

1. **CORS Errors**: Ensure backend CORS is configured for `http://localhost:3000`
2. **Connection Refused**: Check if backend is running on port 8080
3. **WebSocket Errors**: Verify WebSocket endpoint is available
4. **Authentication Issues**: Check JWT token format and expiration

### Debug Mode

Enable debug logging by setting `VITE_ENABLE_DEBUG=true` in your environment file.

### Health Check

The frontend includes a health check utility:

```typescript
import { quickHealthCheck } from '@/utils/connectionTest';

const health = await quickHealthCheck();
console.log(health.healthy ? 'OK' : health.message);
```

## Production Considerations

- Update API URLs for production environment
- Configure proper CORS settings
- Set up SSL/TLS for WebSocket connections
- Implement proper error monitoring
- Configure connection timeouts appropriately