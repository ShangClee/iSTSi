# ADR-001: Project Structure Reorganization

## Status

Accepted

## Date

2024-01-15

## Context

The original project structure mixed frontend, backend, and smart contract code in a single directory without clear separation of concerns. This made it difficult for developers to:

- Understand the system architecture
- Work on specific components independently
- Set up development environments efficiently
- Deploy components separately
- Maintain clear dependency boundaries

The previous structure looked like:
```
previous-project/
├── uxui/           # React frontend (MIGRATED to /frontend)
├── contracts/      # Soroban smart contracts (MIGRATED to /soroban)
├── various config files
└── mixed documentation
```

## Decision

We will restructure the project into a clean monorepo with three main directories:

```
project/
├── frontend/       # React + TypeScript frontend
├── backend/        # Loco.rs + PostgreSQL backend
├── soroban/        # Soroban smart contracts
├── scripts/        # Build and deployment scripts
├── docs/           # Documentation
└── docker-compose.yml # Development environment
```

### Rationale

1. **Clear Separation of Concerns**: Each directory contains only code relevant to that layer
2. **Independent Development**: Teams can work on components without interference
3. **Scalable Architecture**: Each component can be scaled and deployed independently
4. **Industry Standards**: Follows common patterns for full-stack applications
5. **Developer Experience**: Easier onboarding and development workflows

## Consequences

### Positive

- **Improved Developer Experience**: Clear project structure makes onboarding faster
- **Better Maintainability**: Each component has focused responsibilities
- **Independent Deployment**: Components can be deployed separately
- **Clearer Dependencies**: Explicit boundaries between layers
- **Easier Testing**: Component-specific testing strategies

### Negative

- **Migration Effort**: Requires moving and updating existing code
- **Temporary Complexity**: During migration, both old and new structures exist
- **Documentation Updates**: All documentation needs to be updated
- **CI/CD Changes**: Build and deployment pipelines need updates

### Neutral

- **Learning Curve**: Developers need to understand new structure
- **Tooling Updates**: Development tools and scripts need updates

## Implementation

### Phase 1: Directory Creation and File Movement ✅ COMPLETED
1. ✅ Create new directory structure
2. ✅ Move `/uxui` → `/frontend` with configuration updates
3. ✅ Move `/contracts` → `/soroban/contracts`
4. ✅ Create `/backend` with Loco.rs initialization

### Phase 2: Configuration Updates ✅ COMPLETED
1. ✅ Update all import paths and references
2. ✅ Configure cross-component communication
3. ✅ Set up development environment with Docker Compose
4. ✅ Create unified build and deployment scripts

### Phase 3: Documentation and Validation ✅ COMPLETED
1. ✅ Update all documentation
2. ✅ Verify all components build independently
3. ✅ Test full-stack integration
4. ✅ Update CI/CD pipelines

## Alternatives Considered

### Alternative 1: Keep Existing Structure
- **Pros**: No migration effort required
- **Cons**: Continued confusion and poor developer experience
- **Rejected**: Does not solve the fundamental organizational problems

### Alternative 2: Separate Repositories
- **Pros**: Complete isolation between components
- **Cons**: Complex dependency management, difficult cross-component changes
- **Rejected**: Adds unnecessary complexity for a single product

### Alternative 3: Nested Structure with Shared Root
- **Pros**: Some separation while keeping shared configuration
- **Cons**: Still mixing concerns at the root level
- **Rejected**: Does not provide clear enough separation

## Related Decisions

- ADR-002: Frontend Technology Stack
- ADR-003: Backend Framework Selection
- ADR-008: Development Environment Setup

## Notes

This restructuring is the foundation for all other architectural decisions and must be completed before implementing the full-stack integration features.