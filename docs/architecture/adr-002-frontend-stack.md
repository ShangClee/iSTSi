# ADR-002: Frontend Technology Stack

## Status

Accepted

## Date

2024-01-16

## Context

The frontend needs to provide a modern, responsive user interface for the Bitcoin custody application. The application requires:

- Real-time updates for system status and operations
- Complex state management for user sessions and application data
- Form handling for deposits, withdrawals, and KYC processes
- Integration with backend APIs and WebSocket connections
- Professional UI components for financial applications
- TypeScript support for type safety and developer experience

## Decision

We will use the following frontend technology stack:

### Core Framework
- **React 18** with functional components and hooks
- **TypeScript** for type safety and better developer experience
- **Vite** as the build tool and development server

### State Management
- **Redux Toolkit** for global application state
- **React Query/TanStack Query** for server state management
- **React Hook Form** for form state and validation

### UI Framework
- **Tailwind CSS** for utility-first styling
- **Radix UI** for accessible, unstyled UI primitives
- **Lucide React** for consistent iconography

### Development Tools
- **ESLint** and **Prettier** for code quality
- **Vitest** for unit testing
- **Playwright** for end-to-end testing
- **Storybook** for component development and documentation

## Rationale

### React 18
- **Industry Standard**: Widely adopted with excellent ecosystem
- **Performance**: Concurrent features and automatic batching
- **Developer Experience**: Excellent tooling and debugging support
- **Team Familiarity**: Team has extensive React experience

### TypeScript
- **Type Safety**: Prevents runtime errors and improves code quality
- **Developer Experience**: Better IDE support and refactoring capabilities
- **API Integration**: Strong typing for API requests and responses
- **Maintainability**: Self-documenting code and better refactoring

### Vite
- **Fast Development**: Hot module replacement and fast builds
- **Modern Tooling**: Native ES modules and optimized bundling
- **TypeScript Support**: Built-in TypeScript support without configuration
- **Plugin Ecosystem**: Rich plugin ecosystem for additional functionality

### Redux Toolkit
- **Predictable State**: Centralized state management with clear patterns
- **DevTools**: Excellent debugging capabilities with Redux DevTools
- **Middleware**: Built-in support for async actions and side effects
- **Boilerplate Reduction**: Simplified Redux usage with modern patterns

### Tailwind CSS + Radix UI
- **Rapid Development**: Utility-first CSS for fast prototyping
- **Consistency**: Design system approach with consistent spacing and colors
- **Accessibility**: Radix UI provides accessible components out of the box
- **Customization**: Easy theming and component customization

## Implementation Details

### Project Structure
```
frontend/
├── src/
│   ├── components/
│   │   ├── ui/              # Radix UI components
│   │   ├── forms/           # Form components
│   │   ├── layout/          # Layout components
│   │   └── pages/           # Page-level components
│   ├── services/            # API clients and business logic
│   ├── store/               # Redux store and slices
│   ├── hooks/               # Custom React hooks
│   ├── types/               # TypeScript type definitions
│   ├── utils/               # Utility functions
│   └── App.tsx
├── public/                  # Static assets
├── package.json
├── vite.config.ts
├── tailwind.config.js
└── tsconfig.json
```

### Configuration Files

**vite.config.ts**:
```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react-swc';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 3000,
    proxy: {
      '/api': 'http://localhost:8080',
    },
  },
});
```

**tailwind.config.js**:
```javascript
module.exports = {
  content: ['./src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#eff6ff',
          500: '#3b82f6',
          900: '#1e3a8a',
        },
      },
    },
  },
  plugins: [require('@tailwindcss/forms')],
};
```

### State Management Pattern
```typescript
// Store setup
export const store = configureStore({
  reducer: {
    auth: authSlice.reducer,
    deposits: depositsSlice.reducer,
    system: systemSlice.reducer,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: [FLUSH, REHYDRATE, PAUSE, PERSIST, PURGE, REGISTER],
      },
    }),
});

// Slice example
const depositsSlice = createSlice({
  name: 'deposits',
  initialState: {
    items: [],
    loading: false,
    error: null,
  },
  reducers: {
    setLoading: (state, action) => {
      state.loading = action.payload;
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(fetchDeposits.pending, (state) => {
        state.loading = true;
      })
      .addCase(fetchDeposits.fulfilled, (state, action) => {
        state.loading = false;
        state.items = action.payload;
      });
  },
});
```

## Consequences

### Positive
- **Type Safety**: TypeScript prevents many runtime errors
- **Developer Productivity**: Modern tooling and hot reloading
- **Maintainability**: Clear patterns and well-structured code
- **Performance**: Optimized builds and runtime performance
- **Accessibility**: Built-in accessibility with Radix UI
- **Testing**: Comprehensive testing capabilities

### Negative
- **Learning Curve**: Team needs to learn Redux Toolkit patterns
- **Bundle Size**: Multiple libraries increase initial bundle size
- **Complexity**: More sophisticated setup than simple React apps

### Neutral
- **Build Time**: Vite provides fast builds, but complex apps still take time
- **Dependency Management**: Need to keep multiple packages updated

## Alternatives Considered

### Alternative 1: Next.js Framework
- **Pros**: Full-stack framework with SSR/SSG capabilities
- **Cons**: Overkill for SPA, adds complexity we don't need
- **Rejected**: We don't need server-side rendering for this application

### Alternative 2: Vue.js + Nuxt
- **Pros**: Simpler learning curve, good performance
- **Cons**: Smaller ecosystem, team unfamiliarity
- **Rejected**: Team expertise is in React ecosystem

### Alternative 3: Zustand for State Management
- **Pros**: Simpler API, smaller bundle size
- **Cons**: Less mature ecosystem, fewer debugging tools
- **Rejected**: Redux Toolkit provides better tooling for complex applications

### Alternative 4: Styled Components for Styling
- **Pros**: CSS-in-JS with component co-location
- **Cons**: Runtime overhead, larger bundle size
- **Rejected**: Tailwind CSS provides better performance and consistency

## Related Decisions

- ADR-001: Project Structure Reorganization
- ADR-007: API Design and Communication
- ADR-008: Development Environment Setup

## Migration Strategy

1. **Setup New Frontend Structure**: Create new directory with modern tooling
2. **Component Migration**: Move existing components to new structure
3. **State Management**: Implement Redux Toolkit for global state
4. **API Integration**: Set up API clients and WebSocket connections
5. **Testing Setup**: Implement comprehensive testing strategy
6. **Build Optimization**: Configure production builds and deployment

## Success Metrics

- **Developer Experience**: Faster development cycles and fewer bugs
- **Performance**: Lighthouse scores > 90 for all metrics
- **Maintainability**: Reduced time for new feature development
- **Type Safety**: Zero TypeScript errors in production builds