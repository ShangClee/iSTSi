# Development Workflows

This document outlines the development workflows for the Bitcoin Custody Full-Stack Application, covering individual component development and cross-component integration patterns.

## Table of Contents

- [General Workflow Principles](#general-workflow-principles)
- [Frontend Development Workflow](#frontend-development-workflow)
- [Backend Development Workflow](#backend-development-workflow)
- [Soroban Contract Development Workflow](#soroban-contract-development-workflow)
- [Cross-Component Development Workflow](#cross-component-development-workflow)
- [Testing Workflows](#testing-workflows)
- [Release and Deployment Workflow](#release-and-deployment-workflow)

## General Workflow Principles

### Branch Strategy

We use a Git Flow-inspired branching strategy:

- `main` - Production-ready code
- `develop` - Integration branch for features
- `feature/*` - Individual feature branches
- `hotfix/*` - Critical production fixes
- `release/*` - Release preparation branches

### Commit Convention

We follow Conventional Commits specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(frontend): add bitcoin deposit form validation
fix(backend): resolve database connection timeout
docs(soroban): update contract deployment guide
```

### Code Review Process

1. Create feature branch from `develop`
2. Implement changes with tests
3. Create pull request to `develop`
4. Automated checks must pass (CI/CD)
5. At least one team member review required
6. Merge after approval

## Frontend Development Workflow

### Setting Up for Frontend Development

```bash
cd frontend
npm install
npm run dev
```

### Development Process

1. **Create Feature Branch**
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/frontend-new-component
   ```

2. **Component Development**
   - Create components in `src/components/`
   - Add TypeScript types in `src/types/`
   - Implement business logic in `src/services/`
   - Add state management in `src/store/slices/`

3. **Testing**
   ```bash
   npm run test          # Unit tests
   npm run test:e2e      # End-to-end tests
   npm run type-check    # TypeScript validation
   npm run lint          # ESLint checks
   ```

4. **Build Verification**
   ```bash
   npm run build         # Production build
   npm run preview       # Preview production build
   ```

### Frontend-Specific Guidelines

**Component Structure:**
```typescript
// src/components/ExampleComponent.tsx
import React from 'react';
import { useAppSelector, useAppDispatch } from '@/store/hooks';

interface ExampleComponentProps {
  title: string;
  onAction: () => void;
}

export const ExampleComponent: React.FC<ExampleComponentProps> = ({
  title,
  onAction
}) => {
  const dispatch = useAppDispatch();
  const state = useAppSelector(state => state.example);

  return (
    <div className="example-component">
      <h2>{title}</h2>
      {/* Component JSX */}
    </div>
  );
};
```

**API Integration:**
```typescript
// src/services/api.ts
export const exampleApi = {
  getData: async (): Promise<ExampleData> => {
    const response = await apiClient.get('/example');
    return response.data;
  },
  
  postData: async (data: ExampleRequest): Promise<ExampleResponse> => {
    const response = await apiClient.post('/example', data);
    return response.data;
  }
};
```

## Backend Development Workflow

### Setting Up for Backend Development

```bash
cd backend
cargo build
cargo loco start
```

### Development Process

1. **Create Feature Branch**
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/backend-new-endpoint
   ```

2. **API Development**
   - Add controllers in `src/controllers/`
   - Create models in `src/models/`
   - Implement services in `src/services/`
   - Add database migrations in `migration/`

3. **Database Changes**
   ```bash
   # Create new migration
   cargo loco generate migration create_new_table
   
   # Run migrations
   cargo loco db migrate
   
   # Reset database (development only)
   cargo loco db reset
   ```

4. **Testing**
   ```bash
   cargo test                    # Unit tests
   cargo test --test integration # Integration tests
   cargo clippy                  # Linting
   cargo fmt                     # Formatting
   ```

### Backend-Specific Guidelines

**Controller Structure:**
```rust
// src/controllers/example.rs
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ExampleRequest {
    pub name: String,
    pub value: i32,
}

#[derive(Serialize)]
pub struct ExampleResponse {
    pub id: i32,
    pub message: String,
}

pub async fn create_example(
    State(ctx): State<AppContext>,
    Json(req): Json<ExampleRequest>,
) -> Result<Json<ExampleResponse>> {
    // Implementation
    Ok(Json(ExampleResponse {
        id: 1,
        message: "Created".to_string(),
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/example")
        .add("/", post(create_example))
}
```

**Service Structure:**
```rust
// src/services/example_service.rs
use crate::models::example::Example;
use sea_orm::DatabaseConnection;

pub struct ExampleService {
    db: DatabaseConnection,
}

impl ExampleService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
    
    pub async fn create_example(&self, data: CreateExampleData) -> Result<Example> {
        // Business logic implementation
    }
}
```

## Soroban Contract Development Workflow

### Setting Up for Contract Development

```bash
cd soroban
cargo build --target wasm32-unknown-unknown --release
```

### Development Process

1. **Create Feature Branch**
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/soroban-new-contract
   ```

2. **Contract Development**
   - Implement contracts in `contracts/*/src/`
   - Add shared utilities in `shared/src/`
   - Create integration tests in `tests/`

3. **Testing and Deployment**
   ```bash
   # Run contract tests
   cargo test
   
   # Build for deployment
   cargo build --target wasm32-unknown-unknown --release
   
   # Deploy to testnet
   ./scripts/deploy-testnet.sh
   
   # Run integration tests
   ./scripts/test-integration.sh
   ```

### Contract-Specific Guidelines

**Contract Structure:**
```rust
// contracts/example/src/lib.rs
#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol, symbol_short};

#[contract]
pub struct ExampleContract;

#[contractimpl]
impl ExampleContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        // Initialization logic
        Ok(())
    }
    
    pub fn example_function(env: Env, param: u64) -> Result<u64, Error> {
        // Contract logic
        Ok(param * 2)
    }
}
```

**Testing Structure:**
```rust
// contracts/example/src/test.rs
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Env as _};

    #[test]
    fn test_example_function() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ExampleContract);
        let client = ExampleContractClient::new(&env, &contract_id);
        
        let result = client.example_function(&42);
        assert_eq!(result, 84);
    }
}
```

## Cross-Component Development Workflow

### Planning Cross-Component Changes

1. **Identify Impact Areas**
   - Which components need changes?
   - What are the dependencies between changes?
   - Are there breaking changes?

2. **Create Implementation Plan**
   - Order of implementation (usually: contracts → backend → frontend)
   - Testing strategy for each component
   - Integration testing approach

3. **Coordinate Development**
   - Use feature flags for gradual rollout
   - Maintain backward compatibility when possible
   - Document API changes clearly

### Implementation Order

**Typical Flow:**
1. **Soroban Contracts** - Implement new contract functionality
2. **Backend Services** - Add contract integration and API endpoints
3. **Frontend Components** - Update UI to use new backend APIs
4. **Integration Testing** - Test full flow end-to-end

### Example: Adding New Bitcoin Deposit Feature

1. **Soroban Contract Changes**
   ```bash
   cd soroban
   # Modify integration-router contract
   # Add new deposit validation logic
   cargo test
   ./scripts/deploy-testnet.sh
   ```

2. **Backend API Changes**
   ```bash
   cd backend
   # Add new deposit endpoint
   # Update Soroban client integration
   # Add database migration for deposit records
   cargo loco db migrate
   cargo test
   ```

3. **Frontend UI Changes**
   ```bash
   cd frontend
   # Add deposit form component
   # Update API client
   # Add state management for deposits
   npm test
   npm run build
   ```

4. **Integration Testing**
   ```bash
   # Start full environment
   ./scripts/dev.sh
   
   # Run end-to-end tests
   ./scripts/test-e2e.sh
   ```

## Testing Workflows

### Unit Testing

**Frontend:**
```bash
cd frontend
npm test                    # Run all tests
npm test -- --watch        # Watch mode
npm test -- --coverage     # Coverage report
```

**Backend:**
```bash
cd backend
cargo test                  # All tests
cargo test integration     # Integration tests only
cargo test --lib           # Library tests only
```

**Soroban:**
```bash
cd soroban
cargo test                  # All contract tests
cargo test --package integration-router  # Specific contract
```

### Integration Testing

```bash
# Full integration test suite
./scripts/test-integration.sh

# Specific integration scenarios
./scripts/test-bitcoin-flow.sh
./scripts/test-kyc-compliance.sh
```

### End-to-End Testing

```bash
# Start test environment
docker-compose -f docker-compose.test.yml up -d

# Run E2E tests
cd frontend
npm run test:e2e

# Cleanup
docker-compose -f docker-compose.test.yml down
```

## Release and Deployment Workflow

### Pre-Release Checklist

- [ ] All tests passing
- [ ] Documentation updated
- [ ] Version numbers bumped
- [ ] Changelog updated
- [ ] Security review completed
- [ ] Performance benchmarks acceptable

### Release Process

1. **Create Release Branch**
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b release/v1.2.0
   ```

2. **Prepare Release**
   ```bash
   # Update version numbers
   ./scripts/bump-version.sh 1.2.0
   
   # Update changelog
   ./scripts/generate-changelog.sh
   
   # Final testing
   ./scripts/test-all.sh
   ```

3. **Deploy to Staging**
   ```bash
   ./scripts/deploy-staging.sh
   ```

4. **Production Deployment**
   ```bash
   # Merge to main
   git checkout main
   git merge release/v1.2.0
   git tag v1.2.0
   
   # Deploy to production
   ./scripts/deploy-production.sh
   ```

### Hotfix Workflow

For critical production issues:

1. **Create Hotfix Branch**
   ```bash
   git checkout main
   git checkout -b hotfix/critical-fix
   ```

2. **Implement Fix**
   ```bash
   # Make minimal changes
   # Add tests for the fix
   # Verify fix works
   ```

3. **Deploy Hotfix**
   ```bash
   # Deploy to staging first
   ./scripts/deploy-staging.sh
   
   # After validation, deploy to production
   ./scripts/deploy-production.sh
   
   # Merge back to main and develop
   git checkout main
   git merge hotfix/critical-fix
   git checkout develop
   git merge hotfix/critical-fix
   ```

## Development Environment Management

### Daily Development Routine

```bash
# Start your day
git checkout develop
git pull origin develop
./scripts/dev.sh

# During development
# Make changes, test locally, commit frequently

# End of day
git push origin feature/your-branch
./scripts/dev-stop.sh
```

### Environment Maintenance

```bash
# Weekly cleanup
./scripts/cleanup-dev.sh

# Update dependencies
./scripts/update-deps.sh

# Rebuild everything clean
./scripts/clean-build.sh
```

### Debugging Workflows

**Frontend Debugging:**
- Use browser dev tools
- Check Redux DevTools for state issues
- Use React Developer Tools
- Check network tab for API issues

**Backend Debugging:**
- Use `cargo loco logs` for application logs
- Check database logs for query issues
- Use `cargo loco doctor` for health checks
- Monitor API endpoints with tools like Postman

**Soroban Debugging:**
- Use `soroban contract invoke` for manual testing
- Check transaction results on Stellar Explorer
- Use contract event logs for debugging
- Test with `soroban-cli` in local environment

This workflow documentation ensures consistent development practices across all team members and components.