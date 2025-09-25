# Deleted Files List

This document tracks files that were removed during the integration features cleanup to focus on core implementation needs.

## Deleted Files and Directories

### Documentation Files (Redundant/Outdated)
- `How does the Contract Wizard simplify contract creation for Stellar developers_.md`
- `how to mint a token step-by-step in Stellar network by OpenZeppelin.md`
- `OpenZeppelin contracts in Stella.md`
- `What specific components can developers select in the Contract Wizard interface_.md`
- `iSHSi-Bitcoin-Anchor-Implementation-Plan.md` (superseded by integration specs)
- `iSHSi-Clarification-Answers.md`
- `iSHSi-Clarification.md`
- `iSTSi-Contract-Audit.md`
- `Token-Name-Update-Summary.md`

### Legacy Code Files
- `iSatoshi.rs` (legacy implementation)
- `project.zip` (archive file)

### Bitcoin Custody System (Separate Concern)
- `bitcoin_custody/` directory and all contents:
  - `bitcoin_custody/__pycache__/`
  - `bitcoin_custody/web/`
  - `bitcoin_custody/demo.py`
  - `bitcoin_custody/descriptor_manager.py`
  - `bitcoin_custody/psbt_manager.py`
  - `bitcoin_custody/psbt_workflow.py`
  - `bitcoin_custody/README.md`
  - `bitcoin_custody/requirements.txt`

### Monitoring Infrastructure (Separate Deployment Concern)
- `monitoring/` directory and all contents:
  - `monitoring/alerts/`
  - `monitoring/dashboards/`
  - `monitoring/grafana/`
  - `monitoring/health-checks/`
  - `monitoring/logrotate/`
  - `monitoring/scripts/`
  - `monitoring/systemd/`
  - `monitoring/alertmanager.yml`
  - `monitoring/docker-compose.yml`
  - `monitoring/prometheus.yml`
  - `monitoring/README.md`
  - `monitoring/test_monitoring.sh`

### Build Artifacts
- `target/` directory (build artifacts, can be regenerated)

## Reason for Deletion

These files were removed to:

1. **Focus on Core Integration**: Remove files not directly related to the Stellar smart contract integration features
2. **Reduce Complexity**: Eliminate redundant documentation and outdated implementation plans
3. **Separate Concerns**: Move infrastructure and monitoring concerns out of the core contract development
4. **Clean Development Environment**: Remove build artifacts and legacy code that could cause confusion

## Retained Files

The following core files were kept for the integration implementation:

### Core Project Files
- `Cargo.toml` - Main project configuration
- `Cargo.lock` - Dependency lock file
- `README.md` - Project documentation
- `SETUP.md` - Setup instructions
- `DeploymentReadMe.md` - Deployment guide

### Integration Specifications
- `.kiro/specs/integration-features/` - Complete integration feature specifications
  - `requirements.md`
  - `design.md` 
  - `tasks.md`

### Smart Contracts
- `contracts/` - All smart contract implementations
  - `contracts/istsi_token/` - Enhanced iSTSi token with integration capabilities
  - `contracts/kyc_registry/` - KYC compliance registry
  - `contracts/reserve_manager/` - Bitcoin reserve management
  - `contracts/integration_router/` - Central integration orchestrator
  - `contracts/fungible/` - Base fungible token implementation

### Documentation
- `docs/KYC_registry.md` - KYC registry documentation
- `docs/ImplementUpdate.md` - Implementation updates
- `docs/integration/` - Integration-specific documentation
- `docs/deletedFiles.md` - This file

### Development Tools
- `.vscode/` - VS Code configuration
- `scripts/` - Build and deployment scripts

## Recovery

If any deleted files are needed later, they can be recovered from git history:

```bash
git log --oneline --name-only
git checkout <commit-hash> -- <file-path>
```

---
*Generated during integration features cleanup - December 2024*