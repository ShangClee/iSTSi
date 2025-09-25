# Comprehensive Testing Guide

This document outlines the complete testing strategy and implementation for the Bitcoin Custody full-stack application.

## Overview

Our testing strategy covers multiple layers:

- **Unit Tests**: Individual component and function testing
- **Integration Tests**: Cross-component interaction testing
- **End-to-End Tests**: Complete user workflow testing
- **Performance Tests**: Load and performance validation
- **Security Tests**: Security vulnerability testing
- **Contract Tests**: Smart contract functionality testing

## Test Structure

```
├── frontend/
│   ├── src/test/
│   │   ├── setup.ts                 # Test configuration
│   │   ├── utils.tsx                # Test utilities and helpers
│   │   ├── e2e/                     # End-to-end tests
│   │   ├── performance/             # Performance tests
│   │   └── security/                # Security tests
│   ├── vitest.config.ts             # Unit test configuration
│   └── playwright.config.ts         # E2E test configuration
├── backend/
│   ├── tests/
│   │   ├── unit/                    # Unit tests
│   │   ├── integration/             # Integration tests
│   │   ├── fixtures/                # Test data fixtures
│   │   └── lib.rs                   # Test utilities
└── soroban/
    └── tests/
        ├── unit/                    # Contract unit tests
        └── integration_test.rs      # Contract integration tests
```

## Frontend Testing

### Unit Tests

Frontend unit tests use **Vitest** and **React Testing Library**.

#### Running Tests

```bash
cd frontend

# Run all unit tests
npm run test

# Run tests in watch mode
npm run test:watch

# Run tests with coverage
npm run test:coverage

# Run specific test file
npm run test -- SystemOverview.test.tsx
```

#### Test Categories

1. **Component Tests**: Test React component rendering and behavior
2. **Hook Tests**: Test custom React hooks
3. **Service Tests**: Test API clients and business logic
4. **Store Tests**: Test Redux state management

#### Example Component Test

```typescript
import { render, screen, userEvent } from '@/test/utils';
import SystemOverview from '../SystemOverview';

test('displays system metrics correctly', async () => {
  render(<SystemOverview />);
  
  await waitFor(() => {
    expect(screen.getByText('10.00 BTC')).toBeInTheDocument();
  });
});
```

### End-to-End Tests

E2E tests use **Playwright** to test complete user workflows.

#### Running E2E Tests

```bash
cd frontend

# Run all E2E tests
npm run test:e2e

# Run E2E tests with UI
npm run test:e2e:ui

# Run E2E tests in debug mode
npm run test:e2e:debug

# Run specific test suite
npm run test:e2e -- bitcoin-deposit.spec.ts
```

#### Test Categories

1. **User Workflows**: Complete user journeys (login → deposit → withdrawal)
2. **Form Validation**: Input validation and error handling
3. **Real-time Updates**: WebSocket functionality
4. **Error Scenarios**: Network failures and API errors

### Performance Tests

Performance tests validate application speed and responsiveness.

```bash
# Run performance tests
npm run test:performance
```

#### Performance Metrics

- **Page Load Time**: < 3 seconds
- **API Response Time**: < 500ms
- **Memory Usage**: Stable over time
- **WebSocket Connection**: < 1 second

### Security Tests

Security tests validate protection against common vulnerabilities.

```bash
# Run security tests
npm run test:security
```

#### Security Validations

- **XSS Protection**: Input sanitization
- **CSRF Protection**: Token validation
- **Authentication**: JWT handling
- **Authorization**: Route protection
- **Input Validation**: Length limits and format validation

## Backend Testing

### Unit Tests

Backend unit tests use Rust's built-in testing framework with additional testing utilities.

#### Running Tests

```bash
cd backend

# Run all unit tests
cargo test --lib

# Run specific test module
cargo test --lib services::integration_service

# Run tests with output
cargo test --lib -- --nocapture

# Run tests with coverage
cargo tarpaulin --out html
```

#### Test Categories

1. **Controller Tests**: API endpoint testing
2. **Service Tests**: Business logic testing
3. **Model Tests**: Database model testing
4. **Middleware Tests**: Authentication and CORS testing

#### Example Service Test

```rust
#[tokio::test]
async fn test_bitcoin_deposit_validation() {
    let service = IntegrationService::new(create_test_config()).unwrap();
    
    let request = BitcoinDepositRequest {
        user_address: "GAAAA...".to_string(),
        btc_amount: 100000000,
        btc_tx_hash: "abcdef...".to_string(),
        confirmations: 6,
    };
    
    let result = service.validate_bitcoin_deposit(&request).await;
    assert!(result.is_ok());
}
```

### Integration Tests

Integration tests validate cross-component interactions.

```bash
# Run integration tests
cargo test --test '*'

# Run specific integration test
cargo test --test full_stack_test
```

#### Integration Test Categories

1. **API Integration**: Frontend-backend communication
2. **Database Integration**: ORM and migration testing
3. **Soroban Integration**: Smart contract interaction
4. **Full Stack**: Complete system testing

## Soroban Contract Testing

### Contract Unit Tests

Smart contract tests validate individual contract functionality.

#### Running Tests

```bash
cd soroban

# Build contracts
cargo build --target wasm32-unknown-unknown --release

# Run contract tests
cargo test

# Run specific contract tests
cargo test --package kyc_registry
```

#### Test Categories

1. **Contract Initialization**: Setup and configuration
2. **Business Logic**: Core contract functionality
3. **Access Control**: Admin and user permissions
4. **Error Handling**: Invalid input and edge cases

#### Example Contract Test

```rust
#[test]
fn test_kyc_registry_initialization() {
    let setup = KycTestSetup::new();
    
    assert_eq!(setup.kyc_client.get_admin(), setup.admin);
    assert!(setup.kyc_client.is_registry_enabled());
}
```

### Contract Integration Tests

Integration tests validate cross-contract interactions.

```bash
# Run integration tests
cargo test --test integration_test
```

## Continuous Integration

### GitHub Actions Workflow

Our CI pipeline runs comprehensive tests on every push and pull request:

1. **Frontend Tests**: Unit, E2E, performance, and security tests
2. **Backend Tests**: Unit and integration tests with database
3. **Soroban Tests**: Contract compilation and testing
4. **Cross-Component Validation**: Full-stack integration tests

#### Workflow Stages

```yaml
jobs:
  frontend-tests:     # Frontend unit and component tests
  backend-tests:      # Backend unit and integration tests
  soroban-tests:      # Smart contract tests
  e2e-tests:          # End-to-end workflow tests
  performance-tests:  # Performance and load tests
  security-tests:     # Security vulnerability tests
  integration-validation: # Full-stack integration tests
```

### Running CI Locally

```bash
# Install act (GitHub Actions runner)
brew install act  # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Run CI pipeline locally
act push
```

## Test Data Management

### Fixtures and Factories

We use test fixtures and factories for consistent test data:

#### Frontend Test Data

```typescript
export const mockUser = {
  id: 'test-user-id',
  email: 'test@example.com',
  kycStatus: 'approved',
  tier: 1,
};

export const mockOperation = {
  id: 'op-123',
  type: 'bitcoin_deposit',
  status: 'completed',
  amount: '100000000',
};
```

#### Backend Test Fixtures

```rust
impl UserFixture {
    pub fn create_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            kyc_status: KycStatus::Approved,
            tier: 1,
            // ...
        }
    }
}
```

### Database Setup

Tests use isolated database instances:

```bash
# Test database setup
DATABASE_URL=postgres://postgres:password@localhost:5432/bitcoin_custody_test
```

## Coverage Requirements

### Minimum Coverage Targets

- **Frontend**: 80% line coverage
- **Backend**: 85% line coverage
- **Soroban Contracts**: 90% line coverage

### Coverage Reports

```bash
# Frontend coverage
cd frontend && npm run test:coverage

# Backend coverage
cd backend && cargo tarpaulin --out html

# View coverage reports
open frontend/coverage/index.html
open backend/tarpaulin-report.html
```

## Best Practices

### Test Organization

1. **Arrange-Act-Assert**: Structure tests clearly
2. **Single Responsibility**: One assertion per test
3. **Descriptive Names**: Clear test descriptions
4. **Independent Tests**: No test dependencies

### Mock Strategy

1. **External APIs**: Always mock external services
2. **Database**: Use test database or in-memory storage
3. **Time**: Mock time-dependent functions
4. **Random Data**: Use deterministic test data

### Performance Considerations

1. **Parallel Execution**: Run tests in parallel when possible
2. **Test Isolation**: Avoid shared state between tests
3. **Resource Cleanup**: Clean up after each test
4. **Selective Testing**: Run relevant tests during development

## Debugging Tests

### Frontend Debugging

```bash
# Debug specific test
npm run test:e2e:debug -- bitcoin-deposit.spec.ts

# Run tests with browser UI
npm run test:e2e:ui
```

### Backend Debugging

```bash
# Run tests with output
cargo test -- --nocapture

# Debug specific test
cargo test test_bitcoin_deposit_validation -- --nocapture
```

### Common Issues

1. **Timing Issues**: Use proper waits and timeouts
2. **State Pollution**: Ensure test isolation
3. **Mock Configuration**: Verify mock setup
4. **Environment Variables**: Check test environment config

## Monitoring and Reporting

### Test Results

- **GitHub Actions**: Automated test reporting
- **Coverage Reports**: Codecov integration
- **Performance Metrics**: Lighthouse CI
- **Security Scans**: Automated vulnerability detection

### Notifications

- **Slack Integration**: Test failure notifications
- **Email Alerts**: Critical test failures
- **Dashboard**: Real-time test status

## Contributing

### Adding New Tests

1. **Follow Naming Conventions**: `component.test.tsx`, `service_test.rs`
2. **Update Documentation**: Document new test categories
3. **Maintain Coverage**: Ensure coverage targets are met
4. **Review Guidelines**: Follow code review process

### Test Review Checklist

- [ ] Tests cover happy path and error cases
- [ ] Tests are independent and isolated
- [ ] Mock usage is appropriate
- [ ] Performance impact is minimal
- [ ] Documentation is updated

## Resources

- [Vitest Documentation](https://vitest.dev/)
- [Playwright Documentation](https://playwright.dev/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Soroban Testing](https://soroban.stellar.org/docs/getting-started/hello-world#tests)