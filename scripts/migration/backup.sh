#!/bin/bash
set -e

# Bitcoin Custody System - Project Structure Backup Script
# This script creates a backup of the current project structure before migration

BACKUP_DIR="backup-$(date +%Y%m%d-%H%M%S)"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "🔄 Creating backup of current project structure..."
echo "Project root: $PROJECT_ROOT"
echo "Backup directory: $BACKUP_DIR"

# Create backup directory
mkdir -p "$PROJECT_ROOT/$BACKUP_DIR"

# Backup directories that will be moved/modified
echo "📁 Backing up /uxui directory..."
if [ -d "$PROJECT_ROOT/uxui" ]; then
    cp -r "$PROJECT_ROOT/uxui" "$PROJECT_ROOT/$BACKUP_DIR/"
    echo "✅ /uxui backed up"
else
    echo "⚠️  /uxui directory not found"
fi

echo "📁 Backing up /contracts directory..."
if [ -d "$PROJECT_ROOT/contracts" ]; then
    cp -r "$PROJECT_ROOT/contracts" "$PROJECT_ROOT/$BACKUP_DIR/"
    echo "✅ /contracts backed up"
else
    echo "⚠️  /contracts directory not found"
fi

# Backup important configuration files
echo "📄 Backing up configuration files..."
files_to_backup=(
    "Cargo.toml"
    "Cargo.lock"
    "README.md"
    "SETUP.md"
    "DeploymentReadMe.md"
)

for file in "${files_to_backup[@]}"; do
    if [ -f "$PROJECT_ROOT/$file" ]; then
        cp "$PROJECT_ROOT/$file" "$PROJECT_ROOT/$BACKUP_DIR/"
        echo "✅ $file backed up"
    else
        echo "⚠️  $file not found"
    fi
done

# Backup .kiro directory (excluding the new specs we're creating)
echo "📁 Backing up .kiro directory..."
if [ -d "$PROJECT_ROOT/.kiro" ]; then
    cp -r "$PROJECT_ROOT/.kiro" "$PROJECT_ROOT/$BACKUP_DIR/"
    echo "✅ .kiro backed up"
fi

# Create backup manifest
echo "📋 Creating backup manifest..."
cat > "$PROJECT_ROOT/$BACKUP_DIR/BACKUP_MANIFEST.md" << EOF
# Project Structure Backup

**Created:** $(date)
**Backup Directory:** $BACKUP_DIR

## Backed Up Items

### Directories
- \`uxui/\` - Original React frontend code
- \`contracts/\` - Original Soroban smart contracts (moved to soroban/contracts/)
- \`.kiro/\` - Kiro configuration and specs

### Files
$(for file in "${files_to_backup[@]}"; do
    if [ -f "$PROJECT_ROOT/$file" ]; then
        echo "- \`$file\` - $(ls -lh "$PROJECT_ROOT/$file" | awk '{print $5}') - $(date -r "$PROJECT_ROOT/$file")"
    fi
done)

## Restoration Instructions

To restore the original structure:

1. Stop any running services
2. Remove new directories: \`rm -rf frontend backend soroban\`
3. Restore from backup: \`cp -r $BACKUP_DIR/* .\`
4. Remove backup directory: \`rm -rf $BACKUP_DIR\`

## Migration Context

This backup was created as part of the project restructure migration that:
- Moves \`/uxui\` → \`/frontend\`
- Moves \`/contracts\` → \`/soroban/contracts\`
- Creates new \`/backend\` with Loco.rs
- Establishes clean component separation

**⚠️ Important:** Keep this backup until migration is complete and validated.
EOF

echo ""
echo "✅ Backup completed successfully!"
echo "📍 Backup location: $PROJECT_ROOT/$BACKUP_DIR"
echo "📋 Manifest: $PROJECT_ROOT/$BACKUP_DIR/BACKUP_MANIFEST.md"
echo ""
echo "🔒 The backup includes:"
echo "   - Original /uxui directory (React frontend)"
echo "   - Original /contracts directory (Soroban contracts)"
echo "   - Configuration files (Cargo.toml, README.md, etc.)"
echo "   - .kiro directory (specs and settings)"
echo ""
echo "💡 To restore original structure, see instructions in BACKUP_MANIFEST.md"