# iSTSi Integration Platform Tools

This directory contains operational tools for managing the iSTSi integration platform.

## Directory Structure

```
soroban/tools/
├── deployment/           # Contract deployment tools
│   ├── deployment_orchestrator.rs    # Main deployment automation
│   └── deployment_verification.sh    # Post-deployment verification
├── upgrade/             # Contract upgrade management
│   └── contract_upgrade_manager.rs   # Upgrade orchestration with rollback
└── config/              # Configuration management
    └── config_manager.rs             # Environment configuration management
```

## Configuration Structure

```
soroban/config/
├── environments/        # Environment-specific configurations
│   ├── deployment_config.json       # Deployment configuration
│   ├── upgrade_config.json          # Upgrade configuration  
│   └── production_config.json       # Production environment config
└── templates/          # Configuration templates
    └── (future template files)
```

## Usage

### Deployment
```bash
# Deploy contracts
./soroban/tools/deployment/deployment_orchestrator.rs soroban/config/environments/deployment_config.json

# Verify deployment
./soroban/tools/deployment/deployment_verification.sh testnet
```

### Upgrades
```bash
# Execute upgrade
./soroban/tools/upgrade/contract_upgrade_manager.rs soroban/config/environments/upgrade_config.json

# Rollback upgrade
./soroban/tools/upgrade/contract_upgrade_manager.rs soroban/config/environments/upgrade_config.json rollback upgrade_id
```

### Configuration Management
```bash
# Validate configuration
./soroban/tools/config/config_manager.rs soroban/config/environments/production_config.json validate testnet

# Apply configuration
./soroban/tools/config/config_manager.rs soroban/config/environments/production_config.json apply testnet
```

## File Types by Directory

- **`soroban/tools/`**: Executable tools (`.rs` for Rust executables, `.sh` for shell scripts)
- **`soroban/config/environments/`**: Environment-specific JSON configuration files
- **`soroban/config/templates/`**: Reusable configuration templates
- **`scripts/`**: Legacy deployment scripts (to be migrated or removed)