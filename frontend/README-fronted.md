# Bitcoin Custody Frontend

A modern React + TypeScript frontend for the Bitcoin Custody full-stack application, providing a comprehensive user interface for managing Bitcoin-backed tokens, KYC compliance, and reserve operations.

## 🏗️ Architecture

This frontend is built with:
- **React 18** with TypeScript for type-safe component development
- **Vite** for fast development and optimized production builds
- **Redux Toolkit** for predictable state management
- **Tailwind CSS** for utility-first styling
- **Radix UI** for accessible, unstyled UI primitives
- **Axios** for HTTP API communication
- **Socket.io** for real-time WebSocket connections

## 📁 Project Structure

```
frontend/
├── src/
│   ├── components/          # React components
│   │   ├── ui/             # Reusable UI components (Radix UI)
│   │   ├── SystemOverview.tsx
│   │   ├── IntegrationRouter.tsx
│   │   ├── ReserveManager.tsx
│   │   ├── ComplianceMonitor.tsx
│   │   └── ...
│   ├── services/           # API clients and external services
│   │   ├── api.ts          # HTTP API client
│   │   ├── websocket.ts    # WebSocket client
│   │   ├── auth.ts         # Authentication service
│   │   └── connection.ts   # Connection utilities
│   ├── store/              # Redux store and slices
│   │   ├── store.ts        # Store configuration
│   │   ├── hooks.ts        # Typed Redux hooks
│   │   └── slices/         # Redux slices
│   ├── hooks/              # Custom React hooks
│   ├── types/              # TypeScript type definitions
│   ├── utils/              # Utility functions
│   └── styles/             # Global styles and CSS
├── public/                 # Static assets
├── package.json            # Dependencies and scripts
├── vite.config.ts          # Vite configuration
├── tailwind.config.js      # Tailwind CSS configuration
├── tsconfig.json           # TypeScript configuration
└── .env.example            # Environment variables template
```

## 🚀 Quick Start

### Prerequisites

- Node.js 18+ and npm
- Backend service running on `http://localhost:8080`
- PostgreSQL database (handled by backend)

### Installation

1. **Clone and navigate to frontend directory:**
   ```bash
   cd frontend
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Set up environment variables:**
   ```bash
   cp .env.example .env.development
   ```
   
   Edit `.env.development` with your configuration:
   ```env
   VITE_API_URL=http://localhost:8080
   VITE_WS_URL=ws://localhost:8080
   VITE_ENVIRONMENT=development
   ```

4. **Start development server:**
   ```bash
   npm run dev
   ```

The application will be available at `http://localhost:3000`

## 🛠️ Development

### Available Scripts

- `npm run dev` - Start development server with hot reloading
- `npm run build` - Build for production
- `npm run preview` - Preview production build locally
- `npm run lint` - Run ESLint for code quality
- `npm run type-check` - Run TypeScript type checking
- `npm run test` - Run unit tests (when configured)

### Development Workflow

1. **Component Development:**
   - Create new components in `src/components/`
   - Use TypeScript for all components
   - Follow the existing component patterns
   - Utilize Radix UI primitives for accessibility

2. **State Management:**
   - Use Redux Toolkit for global state
   - Create slices in `src/store/slices/`
   - Use typed hooks from `src/store/hooks.ts`

3. **API Integration:**
   - Add new API calls to `src/services/api.ts`
   - Define TypeScript interfaces in `src/types/`
   - Handle errors consistently across the application

4. **Styling:**
   - Use Tailwind CSS utility classes
   - Follow the design system established in UI components
   - Maintain responsive design principles

### Hot Reloading

The development server supports hot reloading for:
- React components
- CSS and Tailwind styles
- TypeScript files
- Environment variables (requires restart)

### Code Quality

The project enforces code quality through:
- **ESLint** for JavaScript/TypeScript linting
- **TypeScript** for type safety
- **Prettier** for code formatting (configure in your editor)

## 🔌 API Integration

### HTTP API Client

The frontend communicates with the backend through a configured Axios client:

```typescript
// Example API usage
import { integrationApi } from '@/services/api';

const handleBitcoinDeposit = async (depositData) => {
  try {
    const response = await integrationApi.executeBitcoinDeposit(depositData);
    // Handle success
  } catch (error) {
    // Handle error
  }
};
```

### WebSocket Connection

Real-time updates are handled through Socket.io:

```typescript
// Example WebSocket usage
import { useWebSocket } from '@/hooks/useWebSocket';

const MyComponent = () => {
  const { isConnected, subscribe } = useWebSocket();
  
  useEffect(() => {
    const unsubscribe = subscribe('system-update', (data) => {
      // Handle real-time update
    });
    
    return unsubscribe;
  }, [subscribe]);
};
```

### Authentication

Authentication is handled through JWT tokens:

```typescript
// Example authentication usage
import { useAuth } from '@/hooks/useAuth';

const MyComponent = () => {
  const { user, login, logout, isAuthenticated } = useAuth();
  
  const handleLogin = async (credentials) => {
    await login(credentials);
  };
};
```

## 🏗️ Building for Production

### Build Process

1. **Create production build:**
   ```bash
   npm run build
   ```

2. **Preview build locally:**
   ```bash
   npm run preview
   ```

### Build Optimization

The production build includes:
- **Code splitting** for optimal loading
- **Tree shaking** to remove unused code
- **Asset optimization** (images, fonts, etc.)
- **Source maps** for debugging
- **Gzip compression** ready assets

### Environment Configuration

Create environment-specific files:
- `.env.development` - Development environment
- `.env.staging` - Staging environment  
- `.env.production` - Production environment

Example production configuration:
```env
VITE_API_URL=https://api.yourdomain.com
VITE_WS_URL=wss://api.yourdomain.com
VITE_ENVIRONMENT=production
```

## 🚀 Deployment

### Static Hosting (Recommended)

The frontend builds to static files that can be served by any web server:

1. **Build the application:**
   ```bash
   npm run build
   ```

2. **Deploy the `dist/` folder** to your hosting provider:
   - Vercel: `vercel --prod`
   - Netlify: Drag and drop `dist/` folder
   - AWS S3: Upload `dist/` contents to S3 bucket
   - Nginx: Copy `dist/` contents to web root

### Docker Deployment

Use the included Dockerfile for containerized deployment:

```bash
# Build Docker image
docker build -t bitcoin-custody-frontend .

# Run container
docker run -p 3000:3000 bitcoin-custody-frontend
```

### Reverse Proxy Configuration

When deploying behind a reverse proxy, ensure proper configuration:

**Nginx example:**
```nginx
server {
    listen 80;
    server_name yourdomain.com;
    
    location / {
        root /var/www/bitcoin-custody-frontend;
        try_files $uri $uri/ /index.html;
    }
    
    location /api {
        proxy_pass http://backend:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## 🔧 Troubleshooting

### Common Issues

**1. Development server won't start**
```bash
# Clear npm cache
npm cache clean --force

# Delete node_modules and reinstall
rm -rf node_modules package-lock.json
npm install
```

**2. API calls failing**
- Check that backend is running on `http://localhost:8080`
- Verify CORS configuration in backend
- Check network tab in browser dev tools
- Ensure environment variables are set correctly

**3. WebSocket connection issues**
- Verify WebSocket URL in environment variables
- Check browser console for connection errors
- Ensure backend WebSocket server is running
- Check firewall/proxy settings

**4. Build failures**
```bash
# Check for TypeScript errors
npm run type-check

# Check for linting errors
npm run lint

# Clear Vite cache
rm -rf node_modules/.vite
```

**5. Styling issues**
- Ensure Tailwind CSS is properly configured
- Check for conflicting CSS rules
- Verify Tailwind classes are being purged correctly in production

### Performance Issues

**Slow development server:**
- Reduce the number of files being watched
- Exclude unnecessary directories in `vite.config.ts`
- Increase Node.js memory limit: `NODE_OPTIONS="--max-old-space-size=4096" npm run dev`

**Large bundle size:**
- Analyze bundle with `npm run build -- --analyze`
- Implement code splitting for large components
- Use dynamic imports for heavy libraries

### Debugging

**Development debugging:**
- Use React Developer Tools browser extension
- Use Redux DevTools for state debugging
- Enable source maps in development (default)

**Production debugging:**
- Source maps are included in production builds
- Use browser dev tools Network tab for API issues
- Check console for JavaScript errors

## 🧪 Testing

### Unit Testing Setup

To add unit testing to the project:

```bash
# Install testing dependencies
npm install --save-dev vitest @testing-library/react @testing-library/jest-dom jsdom

# Add test script to package.json
"test": "vitest run",
"test:watch": "vitest"
```

### Testing Examples

**Component testing:**
```typescript
// src/components/__tests__/SystemOverview.test.tsx
import { render, screen } from '@testing-library/react';
import { SystemOverview } from '../SystemOverview';

test('renders system overview', () => {
  render(<SystemOverview />);
  expect(screen.getByText('System Overview')).toBeInTheDocument();
});
```

**API testing:**
```typescript
// src/services/__tests__/api.test.ts
import { integrationApi } from '../api';

test('executes bitcoin deposit', async () => {
  const mockResponse = { success: true, txId: 'abc123' };
  // Mock axios and test API call
});
```

## 📚 Additional Resources

- [React Documentation](https://react.dev/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)
- [Vite Guide](https://vitejs.dev/guide/)
- [Redux Toolkit Documentation](https://redux-toolkit.js.org/)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [Radix UI Documentation](https://www.radix-ui.com/docs)

## 🤝 Contributing

1. Follow the existing code style and patterns
2. Add TypeScript types for all new code
3. Test your changes thoroughly
4. Update documentation as needed
5. Follow the component and state management patterns established in the codebase

For questions or issues, refer to the main project documentation or create an issue in the project repository.