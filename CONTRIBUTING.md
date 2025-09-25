# Contributing Guidelines

Thank you for your interest in contributing to the Bitcoin Custody Full-Stack Application! This document provides guidelines and standards for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Process](#development-process)
- [Code Style Standards](#code-style-standards)
- [Testing Requirements](#testing-requirements)
- [Documentation Standards](#documentation-standards)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of background, experience level, or identity.

### Expected Behavior

- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

### Unacceptable Behavior

- Harassment, discrimination, or offensive comments
- Personal attacks or trolling
- Publishing private information without permission
- Any conduct that would be inappropriate in a professional setting

## Getting Started

### Prerequisites

Before contributing, ensure you have:

1. Read the [Onboarding Guide](ONBOARDING.md)
2. Set up the development environment successfully
3. Familiarized yourself with the [Development Workflows](DEVELOPMENT_WORKFLOWS.md)
4. Reviewed the project architecture and design documents

### First Contribution

For your first contribution:

1. Look for issues labeled `good-first-issue` or `help-wanted`
2. Comment on the issue to express interest
3. Wait for maintainer assignment before starting work
4. Follow the development workflow for your chosen component

## Development Process

### Branch Naming Convention

Use descriptive branch names with the following format:

```
<type>/<component>-<short-description>

Examples:
feature/frontend-bitcoin-deposit-form
fix/backend-database-connection-timeout
docs/soroban-deployment-guide-update
refactor/shared-error-handling-utils
```

### Commit Message Standards

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Commit Types:**
- `feat`: New feature implementation
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code formatting changes
- `refactor`: Code refactoring without feature changes
- `test`: Adding or updating tests
- `chore`: Maintenance tasks, dependency updates
- `perf`: Performance improvements
- `ci`: CI/CD configuration changes

**Examples:**
```
feat(frontend): implement bitcoin deposit form validation

Add comprehensive validation for bitcoin deposit amounts,
addresses, and transaction confirmations.

Closes #123

fix(backend): resolve database connection pool exhaustion

Increase connection pool size and add proper connection
cleanup to prevent pool exhaustion under high load.

Fixes #456

docs(soroban): update contract deployment instructions

Add missing steps for testnet deployment and clarify
network configuration requirements.
```

### Scope Guidelines

Use these scopes to indicate which component is affected:

- `frontend`: React frontend changes
- `backend`: Loco.rs backend changes
- `soroban`: Smart contract changes
- `shared`: Changes affecting multiple components
- `ci`: CI/CD pipeline changes
- `docs`: Documentation changes
- `scripts`: Build/deployment script changes

## Code Style Standards

### General Principles

1. **Consistency**: Follow existing code patterns in the project
2. **Readability**: Write code that is easy to understand
3. **Maintainability**: Structure code for easy modification
4. **Performance**: Consider performance implications of your changes
5. **Security**: Follow security best practices for all components

### Frontend Code Style (TypeScript/React)

#### File Organization

```
src/
├── components/
│   ├── ui/              # Reusable UI components
│   ├── forms/           # Form components
│   ├── layout/          # Layout components
│   └── pages/           # Page-level components
├── services/            # API and business logic
├── hooks/               # Custom React hooks
├── store/               # State management
├── types/               # TypeScript type definitions
└── utils/               # Utility functions
```

#### Naming Conventions

```typescript
// Components: PascalCase
export const BitcoinDepositForm: React.FC<Props> = () => {};

// Hooks: camelCase starting with 'use'
export const useApiCall = () => {};

// Constants: SCREAMING_SNAKE_CASE
export const API_ENDPOINTS = {
  DEPOSITS: '/api/deposits'
};

// Types/Interfaces: PascalCase
interface BitcoinDepositRequest {
  amount: number;
  address: string;
}

// Files: kebab-case
bitcoin-deposit-form.tsx
use-api-call.ts
```

#### Component Structure

```typescript
import React, { useState, useEffect } from 'react';
import { useAppSelector, useAppDispatch } from '@/store/hooks';
import { Button } from '@/components/ui/button';
import { validateBitcoinAddress } from '@/utils/validation';

// Props interface
interface BitcoinDepositFormProps {
  onSubmit: (data: DepositData) => void;
  isLoading?: boolean;
}

// Component implementation
export const BitcoinDepositForm: React.FC<BitcoinDepositFormProps> = ({
  onSubmit,
  isLoading = false
}) => {
  // State declarations
  const [amount, setAmount] = useState<string>('');
  const [address, setAddress] = useState<string>('');
  
  // Redux state
  const dispatch = useAppDispatch();
  const { user } = useAppSelector(state => state.auth);
  
  // Effects
  useEffect(() => {
    // Effect logic
  }, []);
  
  // Event handlers
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // Handle form submission
  };
  
  // Render
  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {/* Form content */}
    </form>
  );
};
```

#### TypeScript Guidelines

```typescript
// Use strict typing
interface User {
  id: string;
  email: string;
  role: 'admin' | 'user';
  createdAt: Date;
}

// Avoid 'any' type - use specific types or unknown
const processApiResponse = (data: unknown): User[] => {
  // Type guards for runtime validation
  if (!Array.isArray(data)) {
    throw new Error('Expected array');
  }
  
  return data.map(item => {
    if (!isValidUser(item)) {
      throw new Error('Invalid user data');
    }
    return item;
  });
};

// Use utility types when appropriate
type CreateUserRequest = Omit<User, 'id' | 'createdAt'>;
type UpdateUserRequest = Partial<Pick<User, 'email' | 'role'>>;
```

### Backend Code Style (Rust)

#### File Organization

```
src/
├── controllers/         # HTTP request handlers
├── models/             # Database models
├── services/           # Business logic
├── middleware/         # Custom middleware
├── workers/            # Background jobs
├── utils/              # Utility functions
└── lib.rs              # Library exports
```

#### Naming Conventions

```rust
// Structs: PascalCase
pub struct BitcoinDepositRequest {
    pub amount: u64,
    pub address: String,
}

// Functions: snake_case
pub async fn process_bitcoin_deposit() -> Result<()> {}

// Constants: SCREAMING_SNAKE_CASE
const MAX_DEPOSIT_AMOUNT: u64 = 1_000_000;

// Modules: snake_case
mod bitcoin_service;
mod kyc_validation;
```

#### Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepositError {
    #[error("Invalid bitcoin address: {address}")]
    InvalidAddress { address: String },
    
    #[error("Amount too large: {amount} exceeds maximum {max}")]
    AmountTooLarge { amount: u64, max: u64 },
    
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

// Use Result types consistently
pub async fn validate_deposit(
    request: &BitcoinDepositRequest
) -> Result<(), DepositError> {
    if !is_valid_bitcoin_address(&request.address) {
        return Err(DepositError::InvalidAddress {
            address: request.address.clone()
        });
    }
    
    if request.amount > MAX_DEPOSIT_AMOUNT {
        return Err(DepositError::AmountTooLarge {
            amount: request.amount,
            max: MAX_DEPOSIT_AMOUNT
        });
    }
    
    Ok(())
}
```

#### Controller Structure

```rust
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateDepositRequest {
    pub amount: u64,
    pub bitcoin_address: String,
}

#[derive(Serialize)]
pub struct DepositResponse {
    pub id: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_deposit(
    State(ctx): State<AppContext>,
    Json(req): Json<CreateDepositRequest>,
) -> Result<Json<DepositResponse>> {
    // Validate request
    validate_deposit_request(&req)?;
    
    // Process business logic
    let deposit = ctx.deposit_service
        .create_deposit(req)
        .await?;
    
    // Return response
    Ok(Json(DepositResponse {
        id: deposit.id,
        status: deposit.status.to_string(),
        created_at: deposit.created_at,
    }))
}
```

### Soroban Contract Code Style

#### Contract Structure

```rust
#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, Symbol, Vec
};

#[contracttype]
pub enum DataKey {
    Admin,
    Config,
    UserBalance(Address),
}

#[contracttype]
pub struct Config {
    pub max_deposit: u64,
    pub fee_rate: u32,
}

#[contract]
pub struct BitcoinDepositContract;

#[contractimpl]
impl BitcoinDepositContract {
    /// Initialize the contract with admin and configuration
    pub fn initialize(
        env: Env,
        admin: Address,
        config: Config,
    ) -> Result<(), Error> {
        // Ensure not already initialized
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        
        // Store admin and config
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Config, &config);
        
        Ok(())
    }
    
    /// Process a bitcoin deposit
    pub fn process_deposit(
        env: Env,
        user: Address,
        amount: u64,
        btc_tx_hash: String,
    ) -> Result<(), Error> {
        // Validate inputs
        Self::validate_deposit_amount(&env, amount)?;
        
        // Process deposit logic
        Self::update_user_balance(&env, &user, amount)?;
        
        // Emit event
        env.events().publish((
            symbol_short!("deposit"),
            user.clone(),
            amount,
        ), btc_tx_hash);
        
        Ok(())
    }
    
    // Private helper functions
    fn validate_deposit_amount(env: &Env, amount: u64) -> Result<(), Error> {
        let config: Config = env.storage().instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;
            
        if amount > config.max_deposit {
            return Err(Error::AmountTooLarge);
        }
        
        Ok(())
    }
}
```

#### Error Handling in Contracts

```rust
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    AmountTooLarge = 4,
    InsufficientBalance = 5,
    InvalidAddress = 6,
}
```

## Testing Requirements

### Test Coverage Standards

- **Frontend**: Minimum 80% code coverage
- **Backend**: Minimum 85% code coverage  
- **Soroban**: Minimum 90% code coverage

### Testing Guidelines

#### Frontend Testing

```typescript
// Component testing with React Testing Library
import { render, screen, fireEvent } from '@testing-library/react';
import { BitcoinDepositForm } from './BitcoinDepositForm';

describe('BitcoinDepositForm', () => {
  it('should validate bitcoin address format', () => {
    const onSubmit = jest.fn();
    render(<BitcoinDepositForm onSubmit={onSubmit} />);
    
    const addressInput = screen.getByLabelText(/bitcoin address/i);
    fireEvent.change(addressInput, { target: { value: 'invalid-address' } });
    
    const submitButton = screen.getByRole('button', { name: /submit/i });
    fireEvent.click(submitButton);
    
    expect(screen.getByText(/invalid bitcoin address/i)).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });
});

// API service testing
import { apiClient } from './api';
import { mockApiResponse } from '../__mocks__/api';

jest.mock('axios');

describe('API Client', () => {
  it('should handle deposit creation', async () => {
    mockApiResponse('/deposits', { id: '123', status: 'pending' });
    
    const result = await apiClient.createDeposit({
      amount: 1000,
      address: 'bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh'
    });
    
    expect(result.id).toBe('123');
    expect(result.status).toBe('pending');
  });
});
```

#### Backend Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use loco_rs::testing::prelude::*;
    
    #[tokio::test]
    async fn test_create_deposit_success() {
        let ctx = testing::app_context().await;
        
        let request = CreateDepositRequest {
            amount: 1000,
            bitcoin_address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
        };
        
        let response = create_deposit(State(ctx), Json(request)).await;
        
        assert!(response.is_ok());
        let deposit = response.unwrap().0;
        assert_eq!(deposit.status, "pending");
    }
    
    #[tokio::test]
    async fn test_create_deposit_invalid_address() {
        let ctx = testing::app_context().await;
        
        let request = CreateDepositRequest {
            amount: 1000,
            bitcoin_address: "invalid-address".to_string(),
        };
        
        let response = create_deposit(State(ctx), Json(request)).await;
        
        assert!(response.is_err());
    }
}
```

#### Soroban Contract Testing

```rust
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Env as _};

    #[test]
    fn test_deposit_success() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BitcoinDepositContract);
        let client = BitcoinDepositContractClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        
        // Initialize contract
        let config = Config {
            max_deposit: 1_000_000,
            fee_rate: 100, // 1%
        };
        client.initialize(&admin, &config);
        
        // Test deposit
        let result = client.process_deposit(
            &user,
            &50_000,
            &"btc_tx_hash_123".to_string()
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    #[should_panic(expected = "AmountTooLarge")]
    fn test_deposit_amount_too_large() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BitcoinDepositContract);
        let client = BitcoinDepositContractClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        
        let config = Config {
            max_deposit: 1_000,
            fee_rate: 100,
        };
        client.initialize(&admin, &config);
        
        // This should panic
        client.process_deposit(&user, &2_000, &"btc_tx_hash_123".to_string());
    }
}
```

## Documentation Standards

### Code Documentation

#### TypeScript/React Documentation

```typescript
/**
 * Bitcoin deposit form component that handles user input validation
 * and submission of bitcoin deposit requests.
 * 
 * @example
 * ```tsx
 * <BitcoinDepositForm
 *   onSubmit={handleDeposit}
 *   isLoading={isSubmitting}
 * />
 * ```
 */
interface BitcoinDepositFormProps {
  /** Callback function called when form is submitted with valid data */
  onSubmit: (data: DepositData) => void;
  /** Whether the form is currently submitting */
  isLoading?: boolean;
}

/**
 * Validates a bitcoin address format
 * 
 * @param address - The bitcoin address to validate
 * @returns True if the address is valid, false otherwise
 * 
 * @example
 * ```typescript
 * const isValid = validateBitcoinAddress('bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh');
 * console.log(isValid); // true
 * ```
 */
export function validateBitcoinAddress(address: string): boolean {
  // Implementation
}
```

#### Rust Documentation

```rust
/// Service for handling bitcoin deposit operations
/// 
/// This service manages the complete lifecycle of bitcoin deposits,
/// from initial validation through blockchain confirmation.
/// 
/// # Examples
/// 
/// ```rust
/// let service = DepositService::new(db_connection);
/// let deposit = service.create_deposit(request).await?;
/// ```
pub struct DepositService {
    db: DatabaseConnection,
}

impl DepositService {
    /// Creates a new deposit record after validation
    /// 
    /// # Arguments
    /// 
    /// * `request` - The deposit request containing amount and address
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing the created deposit or an error
    /// 
    /// # Errors
    /// 
    /// This function will return an error if:
    /// * The bitcoin address is invalid
    /// * The amount exceeds maximum limits
    /// * Database operation fails
    pub async fn create_deposit(
        &self,
        request: CreateDepositRequest,
    ) -> Result<Deposit, DepositError> {
        // Implementation
    }
}
```

### README Standards

Each component should have a comprehensive README with:

1. **Purpose and Overview**
2. **Installation and Setup**
3. **Usage Examples**
4. **API Documentation**
5. **Configuration Options**
6. **Testing Instructions**
7. **Troubleshooting Guide**

## Pull Request Process

### Before Creating a Pull Request

1. **Ensure your branch is up to date:**
   ```bash
   git checkout develop
   git pull origin develop
   git checkout your-feature-branch
   git rebase develop
   ```

2. **Run all tests:**
   ```bash
   ./scripts/test-all.sh
   ```

3. **Check code formatting:**
   ```bash
   # Frontend
   cd frontend && npm run lint && npm run type-check
   
   # Backend
   cd backend && cargo fmt --check && cargo clippy
   
   # Soroban
   cd soroban && cargo fmt --check && cargo clippy
   ```

### Pull Request Template

Use this template for all pull requests:

```markdown
## Description

Brief description of the changes made.

## Type of Change

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Components Affected

- [ ] Frontend
- [ ] Backend
- [ ] Soroban Contracts
- [ ] Documentation
- [ ] CI/CD

## Testing

- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed
- [ ] Test coverage maintained/improved

## Checklist

- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or breaking changes documented)
- [ ] Tests added for new functionality

## Screenshots (if applicable)

Add screenshots for UI changes.

## Additional Notes

Any additional information reviewers should know.
```

### Review Process

1. **Automated Checks**: All CI/CD checks must pass
2. **Code Review**: At least one team member must review
3. **Testing**: Reviewer should test the changes locally
4. **Documentation**: Ensure documentation is updated
5. **Approval**: Maintainer approval required for merge

### Review Criteria

Reviewers should check for:

- **Functionality**: Does the code work as intended?
- **Code Quality**: Is the code clean, readable, and maintainable?
- **Performance**: Are there any performance implications?
- **Security**: Are there any security concerns?
- **Testing**: Is the code adequately tested?
- **Documentation**: Is the code properly documented?

## Issue Reporting

### Bug Reports

Use this template for bug reports:

```markdown
## Bug Description

A clear and concise description of the bug.

## Steps to Reproduce

1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

## Expected Behavior

What you expected to happen.

## Actual Behavior

What actually happened.

## Environment

- OS: [e.g. macOS 12.0]
- Browser: [e.g. Chrome 95.0]
- Node.js version: [e.g. 18.0.0]
- Rust version: [e.g. 1.70.0]

## Additional Context

Add any other context about the problem here.
```

### Feature Requests

Use this template for feature requests:

```markdown
## Feature Description

A clear and concise description of the feature.

## Problem Statement

What problem does this feature solve?

## Proposed Solution

Describe your proposed solution.

## Alternatives Considered

Describe any alternative solutions you've considered.

## Additional Context

Add any other context or screenshots about the feature request.
```

## Getting Help

If you need help with contributing:

1. **Check Documentation**: Review existing docs first
2. **Search Issues**: Look for similar questions/issues
3. **Ask Questions**: Create a discussion or issue
4. **Join Community**: Participate in team communications

Thank you for contributing to the Bitcoin Custody Full-Stack Application! 🚀