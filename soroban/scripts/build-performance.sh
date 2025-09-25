#!/bin/bash

# Soroban Contract Build Performance Monitor
# Tracks and reports WASM build performance metrics

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROFILE=${1:-release}
REPORT_FILE="contract-build-performance.json"
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
BUILD_ID=$(date +%s)
TARGET_DIR="target/wasm32-unknown-unknown"

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

# Get list of contracts
get_contracts() {
    find contracts -name "Cargo.toml" -exec dirname {} \; | sort
}

# Measure contract compilation time
measure_contract_compilation() {
    local contract_dir=$1
    local contract_name=$(basename "$contract_dir")
    
    log "Measuring compilation time for contract: $contract_name"
    
    # Clean previous builds
    cargo clean
    
    local start_time=$(date +%s)
    
    cd "$contract_dir"
    
    case $PROFILE in
        "dev")
            cargo build --target wasm32-unknown-unknown
            ;;
        "dev-fast")
            cargo build --target wasm32-unknown-unknown --profile dev-fast
            ;;
        "release")
            cargo build --target wasm32-unknown-unknown --release
            ;;
        "release-size")
            cargo build --target wasm32-unknown-unknown --profile release-size
            ;;
        *)
            cargo build --target wasm32-unknown-unknown --release
            ;;
    esac
    
    cd - > /dev/null
    
    local end_time=$(date +%s)
    local compilation_time=$((end_time - start_time))
    
    echo $compilation_time
}

# Analyze WASM file size
analyze_wasm_size() {
    local contract_dir=$1
    local contract_name=$(basename "$contract_dir")
    
    local wasm_path
    case $PROFILE in
        "dev"|"dev-fast")
            wasm_path="$TARGET_DIR/debug/${contract_name}.wasm"
            ;;
        "release"|"release-size")
            wasm_path="$TARGET_DIR/release/${contract_name}.wasm"
            ;;
    esac
    
    if [ -f "$wasm_path" ]; then
        local size=$(stat -f%z "$wasm_path" 2>/dev/null || stat -c%s "$wasm_path" 2>/dev/null)
        echo $size
    else
        warn "WASM file not found at $wasm_path"
        echo 0
    fi
}

# Optimize contract and measure size reduction
optimize_contract() {
    local contract_dir=$1
    local contract_name=$(basename "$contract_dir")
    
    if ! command -v soroban &> /dev/null; then
        warn "Soroban CLI not found. Skipping optimization."
        echo "{\"original_size\": 0, \"optimized_size\": 0, \"reduction_percent\": 0}"
        return
    fi
    
    local wasm_path
    case $PROFILE in
        "dev"|"dev-fast")
            wasm_path="$TARGET_DIR/debug/${contract_name}.wasm"
            ;;
        "release"|"release-size")
            wasm_path="$TARGET_DIR/release/${contract_name}.wasm"
            ;;
    esac
    
    if [ ! -f "$wasm_path" ]; then
        echo "{\"original_size\": 0, \"optimized_size\": 0, \"reduction_percent\": 0}"
        return
    fi
    
    local original_size=$(stat -f%z "$wasm_path" 2>/dev/null || stat -c%s "$wasm_path" 2>/dev/null)
    local optimized_path="${wasm_path%.wasm}_optimized.wasm"
    
    # Optimize the WASM file
    soroban contract optimize --wasm "$wasm_path" --out "$optimized_path" 2>/dev/null || {
        echo "{\"original_size\": $original_size, \"optimized_size\": 0, \"reduction_percent\": 0}"
        return
    }
    
    local optimized_size=$(stat -f%z "$optimized_path" 2>/dev/null || stat -c%s "$optimized_path" 2>/dev/null)
    local reduction=$((original_size - optimized_size))
    local reduction_percent=0
    
    if [ $original_size -gt 0 ]; then
        reduction_percent=$(echo "scale=2; ($reduction * 100) / $original_size" | bc -l 2>/dev/null || echo "0")
    fi
    
    echo "{\"original_size\": $original_size, \"optimized_size\": $optimized_size, \"reduction_percent\": $reduction_percent}"
}

# Measure workspace build time
measure_workspace_build() {
    log "Measuring workspace build time"
    
    cargo clean
    
    local start_time=$(date +%s)
    
    case $PROFILE in
        "dev")
            cargo build --target wasm32-unknown-unknown
            ;;
        "dev-fast")
            cargo build --target wasm32-unknown-unknown --profile dev-fast
            ;;
        "release")
            cargo build --target wasm32-unknown-unknown --release
            ;;
        "release-size")
            cargo build --target wasm32-unknown-unknown --profile release-size
            ;;
    esac
    
    local end_time=$(date +%s)
    local build_time=$((end_time - start_time))
    
    echo $build_time
}

# Analyze all contracts
analyze_all_contracts() {
    log "Analyzing all contracts"
    
    local contracts_json="["
    local first=true
    
    for contract_dir in $(get_contracts); do
        local contract_name=$(basename "$contract_dir")
        
        log "Analyzing contract: $contract_name"
        
        local compilation_time=$(measure_contract_compilation "$contract_dir")
        local wasm_size=$(analyze_wasm_size "$contract_dir")
        local optimization=$(optimize_contract "$contract_dir")
        
        if [ "$first" = true ]; then
            first=false
        else
            contracts_json="$contracts_json,"
        fi
        
        contracts_json="$contracts_json{
            \"name\": \"$contract_name\",
            \"compilation_time\": {
                \"seconds\": $compilation_time,
                \"formatted\": \"$(format_time $compilation_time)\"
            },
            \"wasm_size\": {
                \"bytes\": $wasm_size,
                \"formatted\": \"$(format_bytes $wasm_size)\"
            },
            \"optimization\": $optimization
        }"
    done
    
    contracts_json="$contracts_json]"
    echo "$contracts_json"
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
    local soroban_version=$(soroban --version 2>/dev/null || echo "not installed")
    
    echo "{\"cpu_count\": $cpu_count, \"memory_total\": $memory_total, \"rust_version\": \"$rust_version\", \"soroban_version\": \"$soroban_version\"}"
}

# Generate performance report
generate_report() {
    log "Generating performance report"
    
    # Measure all metrics
    local workspace_build_time=$(measure_workspace_build)
    local contracts_analysis=$(analyze_all_contracts)
    local cache_info=$(get_cache_info)
    local system_info=$(get_system_info)
    
    # Create JSON report
    local report=$(cat <<EOF
{
  "timestamp": "$TIMESTAMP",
  "build_id": "$BUILD_ID",
  "profile": "$PROFILE",
  "workspace_build_time": {
    "seconds": $workspace_build_time,
    "formatted": "$(format_time $workspace_build_time)"
  },
  "contracts": $contracts_analysis,
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
    success "Contract Build Performance Report Generated"
    echo "=========================================="
    echo "Profile: $PROFILE"
    echo "Workspace Build Time: $(format_time $workspace_build_time)"
    
    # Show contract summaries
    echo
    log "Contract Analysis Summary:"
    echo "$contracts_analysis" | jq -r '.[] | "  \(.name): \(.compilation_time.formatted) | \(.wasm_size.formatted) | \(.optimization.reduction_percent)% reduction"'
    
    echo
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
    
    local current_time=$(echo "$current" | jq '.workspace_build_time.seconds')
    local previous_time=$(echo "$previous" | jq '.workspace_build_time.seconds')
    
    if [ "$previous_time" -gt 0 ]; then
        local time_diff=$((current_time - previous_time))
        local time_percent=$(echo "scale=1; ($time_diff * 100) / $previous_time" | bc -l 2>/dev/null || echo "0")
        
        echo
        log "Performance Trends"
        echo "=================="
        
        if [ "$time_diff" -gt 0 ]; then
            warn "Workspace build time increased by ${time_percent}% ($(format_time ${time_diff#-}))"
        else
            success "Workspace build time decreased by ${time_percent#-}% ($(format_time ${time_diff#-}))"
        fi
    fi
}

# Main execution
main() {
    log "Starting Soroban Contract Build Performance Analysis"
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
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        error "Not in a Cargo workspace directory"
        exit 1
    fi
    
    generate_report
    
    success "Contract performance analysis complete!"
}

# Show help
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: $0 [profile]"
    echo ""
    echo "Profiles:"
    echo "  dev         - Development build"
    echo "  dev-fast    - Fast development build"
    echo "  release     - Release build (default)"
    echo "  release-size - Size-optimized release build"
    echo ""
    echo "Examples:"
    echo "  $0              # Analyze release build"
    echo "  $0 dev          # Analyze dev build"
    echo "  $0 release-size # Analyze size-optimized build"
    exit 0
fi

# Run main function
main "$@"