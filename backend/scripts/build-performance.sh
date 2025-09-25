#!/bin/bash

# Backend Build Performance Monitor
# Tracks and reports Rust build performance metrics

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROFILE=${1:-release}
REPORT_FILE="build-performance-report.json"
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
BUILD_ID=$(date +%s)

log() {
    echo -e "${BLUE}[PERF]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Function to format time
format_time() {
    local seconds=$1
    if [ $seconds -lt 60 ]; then
        echo "${seconds}s"
    else
        local minutes=$((seconds / 60))
        local remaining_seconds=$((seconds % 60))
        echo "${minutes}m ${remaining_seconds}s"
    fi
}

# Function to format bytes
format_bytes() {
    local bytes=$1
    local units=("B" "KB" "MB" "GB")
    local unit=0
    
    while [ $bytes -gt 1024 ] && [ $unit -lt 3 ]; do
        bytes=$((bytes / 1024))
        unit=$((unit + 1))
    done
    
    echo "${bytes}${units[$unit]}"
}

# Measure compilation time
measure_compilation_time() {
    log "Measuring compilation time for profile: $PROFILE"
    
    # Clean previous builds for accurate measurement
    cargo clean
    
    local start_time=$(date +%s)
    
    case $PROFILE in
        "dev")
            cargo build
            ;;
        "dev-fast")
            cargo build --profile dev-fast
            ;;
        "release")
            cargo build --release
            ;;
        "release-debug")
            cargo build --profile release-debug
            ;;
        *)
            error "Unknown profile: $PROFILE"
            return 1
            ;;
    esac
    
    local end_time=$(date +%s)
    local compilation_time=$((end_time - start_time))
    
    echo $compilation_time
}

# Measure incremental compilation time
measure_incremental_time() {
    log "Measuring incremental compilation time"
    
    # Make a small change to trigger incremental build
    local test_file="src/lib.rs"
    local backup_file="${test_file}.backup"
    
    # Backup original file
    cp "$test_file" "$backup_file"
    
    # Add a comment to trigger recompilation
    echo "// Build performance test comment $(date)" >> "$test_file"
    
    local start_time=$(date +%s)
    
    case $PROFILE in
        "dev")
            cargo build
            ;;
        "dev-fast")
            cargo build --profile dev-fast
            ;;
        "release")
            cargo build --release
            ;;
        *)
            cargo build
            ;;
    esac
    
    local end_time=$(date +%s)
    local incremental_time=$((end_time - start_time))
    
    # Restore original file
    mv "$backup_file" "$test_file"
    
    echo $incremental_time
}

# Analyze binary size
analyze_binary_size() {
    log "Analyzing binary size"
    
    local binary_path
    case $PROFILE in
        "dev"|"dev-fast")
            binary_path="target/debug/bitcoin-custody-backend"
            ;;
        "release"|"release-debug")
            binary_path="target/release/bitcoin-custody-backend"
            ;;
    esac
    
    if [ -f "$binary_path" ]; then
        local size=$(stat -f%z "$binary_path" 2>/dev/null || stat -c%s "$binary_path" 2>/dev/null)
        echo $size
    else
        warn "Binary not found at $binary_path"
        echo 0
    fi
}

# Analyze dependency compilation time
analyze_dependency_time() {
    log "Analyzing dependency compilation time"
    
    # Clean and build only dependencies
    cargo clean
    
    local start_time=$(date +%s)
    
    # Build dependencies only (no source files)
    mkdir -p src_backup
    mv src/* src_backup/ 2>/dev/null || true
    echo "fn main() {}" > src/main.rs
    echo "" > src/lib.rs
    
    case $PROFILE in
        "dev")
            cargo build 2>/dev/null || true
            ;;
        "release")
            cargo build --release 2>/dev/null || true
            ;;
    esac
    
    local end_time=$(date +%s)
    local dep_time=$((end_time - start_time))
    
    # Restore source files
    rm -f src/main.rs src/lib.rs
    mv src_backup/* src/ 2>/dev/null || true
    rmdir src_backup 2>/dev/null || true
    
    echo $dep_time
}

# Get cache information
get_cache_info() {
    log "Analyzing build cache"
    
    local cache_dir="target"
    if [ -d "$cache_dir" ]; then
        local cache_size=$(du -sb "$cache_dir" 2>/dev/null | cut -f1 || echo 0)
        local cache_files=$(find "$cache_dir" -type f | wc -l)
        echo "{\"size\": $cache_size, \"files\": $cache_files}"
    else
        echo "{\"size\": 0, \"files\": 0}"
    fi
}

# Get system information
get_system_info() {
    local cpu_count=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 1)
    local memory_total=$(free -b 2>/dev/null | awk '/^Mem:/{print $2}' || echo 0)
    local rust_version=$(rustc --version)
    
    echo "{\"cpu_count\": $cpu_count, \"memory_total\": $memory_total, \"rust_version\": \"$rust_version\"}"
}

# Generate performance report
generate_report() {
    log "Generating performance report"
    
    # Measure all metrics
    local compilation_time=$(measure_compilation_time)
    local incremental_time=$(measure_incremental_time)
    local binary_size=$(analyze_binary_size)
    local dependency_time=$(analyze_dependency_time)
    local cache_info=$(get_cache_info)
    local system_info=$(get_system_info)
    
    # Create JSON report
    local report=$(cat <<EOF
{
  "timestamp": "$TIMESTAMP",
  "build_id": "$BUILD_ID",
  "profile": "$PROFILE",
  "compilation_time": {
    "total_seconds": $compilation_time,
    "formatted": "$(format_time $compilation_time)"
  },
  "incremental_time": {
    "total_seconds": $incremental_time,
    "formatted": "$(format_time $incremental_time)"
  },
  "binary_size": {
    "bytes": $binary_size,
    "formatted": "$(format_bytes $binary_size)"
  },
  "dependency_time": {
    "total_seconds": $dependency_time,
    "formatted": "$(format_time $dependency_time)"
  },
  "cache": $cache_info,
  "system": $system_info
}
EOF
)
    
    # Load existing reports
    local reports="[]"
    if [ -f "$REPORT_FILE" ]; then
        reports=$(cat "$REPORT_FILE")
    fi
    
    # Add new report
    reports=$(echo "$reports" | jq ". + [$report]" | jq 'if length > 10 then .[-10:] else . end')
    
    # Save reports
    echo "$reports" | jq '.' > "$REPORT_FILE"
    
    # Display summary
    echo
    success "Performance Report Generated"
    echo "=========================="
    echo "Profile: $PROFILE"
    echo "Compilation Time: $(format_time $compilation_time)"
    echo "Incremental Time: $(format_time $incremental_time)"
    echo "Binary Size: $(format_bytes $binary_size)"
    echo "Dependency Time: $(format_time $dependency_time)"
    echo "Cache Size: $(echo "$cache_info" | jq -r '.size' | xargs -I {} bash -c 'echo $(format_bytes {})')"
    echo "Report saved to: $REPORT_FILE"
    
    # Show trends if we have previous data
    show_trends "$reports"
}

# Show performance trends
show_trends() {
    local reports="$1"
    local report_count=$(echo "$reports" | jq 'length')
    
    if [ "$report_count" -lt 2 ]; then
        log "Not enough data for trend analysis"
        return
    fi
    
    local current=$(echo "$reports" | jq '.[-1]')
    local previous=$(echo "$reports" | jq '.[-2]')
    
    local current_time=$(echo "$current" | jq '.compilation_time.total_seconds')
    local previous_time=$(echo "$previous" | jq '.compilation_time.total_seconds')
    
    if [ "$previous_time" -gt 0 ]; then
        local time_diff=$((current_time - previous_time))
        local time_percent=$(echo "scale=1; ($time_diff * 100) / $previous_time" | bc -l 2>/dev/null || echo "0")
        
        echo
        log "Performance Trends"
        echo "=================="
        
        if [ "$time_diff" -gt 0 ]; then
            warn "Compilation time increased by ${time_percent}% ($(format_time ${time_diff#-}))"
        else
            success "Compilation time decreased by ${time_percent#-}% ($(format_time ${time_diff#-}))"
        fi
    fi
}

# Main execution
main() {
    log "Starting Backend Build Performance Analysis"
    log "Profile: $PROFILE"
    echo
    
    # Check dependencies
    if ! command -v jq &> /dev/null; then
        error "jq is required for JSON processing. Please install it."
        exit 1
    fi
    
    if ! command -v bc &> /dev/null; then
        warn "bc not found. Trend calculations may not work."
    fi
    
    generate_report
    
    success "Performance analysis complete!"
}

# Show help
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: $0 [profile]"
    echo ""
    echo "Profiles:"
    echo "  dev         - Development build (default)"
    echo "  dev-fast    - Fast development build"
    echo "  release     - Release build"
    echo "  release-debug - Release with debug info"
    echo ""
    echo "Examples:"
    echo "  $0              # Analyze dev build"
    echo "  $0 release      # Analyze release build"
    echo "  $0 dev-fast     # Analyze fast dev build"
    exit 0
fi

# Run main function
main "$@"