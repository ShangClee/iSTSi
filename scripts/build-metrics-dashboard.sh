#!/bin/bash

# Build Metrics Dashboard
# Generates comprehensive build performance reports and trends

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
REPORT_DIR="build-reports"
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
HTML_REPORT="$REPORT_DIR/build-metrics-dashboard.html"

log() {
    echo -e "${BLUE}[METRICS]${NC} $1"
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

info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

# Create report directory
create_report_dir() {
    mkdir -p "$REPORT_DIR"
}

# Collect build metrics from all components
collect_metrics() {
    log "Collecting build metrics from all components..."
    
    local metrics="{\"timestamp\": \"$TIMESTAMP\", \"components\": {}}"
    
    # Frontend metrics
    if [ -f "frontend/build-performance-report.json" ]; then
        local frontend_metrics=$(cat frontend/build-performance-report.json | jq '.[-1]' 2>/dev/null || echo '{}')
        metrics=$(echo "$metrics" | jq ".components.frontend = $frontend_metrics")
    fi
    
    # Backend metrics
    if [ -f "backend/build-performance-report.json" ]; then
        local backend_metrics=$(cat backend/build-performance-report.json | jq '.[-1]' 2>/dev/null || echo '{}')
        metrics=$(echo "$metrics" | jq ".components.backend = $backend_metrics")
    fi
    
    # Soroban metrics
    if [ -f "soroban/contract-build-performance.json" ]; then
        local soroban_metrics=$(cat soroban/contract-build-performance.json | jq '.[-1]' 2>/dev/null || echo '{}')
        metrics=$(echo "$metrics" | jq ".components.soroban = $soroban_metrics")
    fi
    
    echo "$metrics" > "$REPORT_DIR/latest-metrics.json"
    success "Metrics collected and saved to $REPORT_DIR/latest-metrics.json"
}

# Generate HTML dashboard
generate_html_dashboard() {
    log "Generating HTML dashboard..."
    
    cat > "$HTML_REPORT" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Bitcoin Custody - Build Metrics Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #f5f7fa;
            color: #2d3748;
            line-height: 1.6;
        }
        
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 2rem 0;
            text-align: center;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        
        .header h1 {
            font-size: 2.5rem;
            margin-bottom: 0.5rem;
        }
        
        .header p {
            font-size: 1.1rem;
            opacity: 0.9;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
        }
        
        .metrics-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 2rem;
            margin-bottom: 3rem;
        }
        
        .metric-card {
            background: white;
            border-radius: 12px;
            padding: 1.5rem;
            box-shadow: 0 4px 6px rgba(0,0,0,0.05);
            border: 1px solid #e2e8f0;
            transition: transform 0.2s, box-shadow 0.2s;
        }
        
        .metric-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(0,0,0,0.1);
        }
        
        .metric-card h3 {
            color: #4a5568;
            margin-bottom: 1rem;
            font-size: 1.2rem;
            display: flex;
            align-items: center;
        }
        
        .metric-card .icon {
            width: 24px;
            height: 24px;
            margin-right: 0.5rem;
            border-radius: 4px;
        }
        
        .frontend-icon { background: #61dafb; }
        .backend-icon { background: #f74c00; }
        .soroban-icon { background: #ffd700; }
        
        .metric-value {
            font-size: 2rem;
            font-weight: bold;
            color: #2d3748;
            margin-bottom: 0.5rem;
        }
        
        .metric-label {
            color: #718096;
            font-size: 0.9rem;
        }
        
        .status-good { color: #38a169; }
        .status-warning { color: #d69e2e; }
        .status-error { color: #e53e3e; }
        
        .chart-container {
            background: white;
            border-radius: 12px;
            padding: 2rem;
            margin-bottom: 2rem;
            box-shadow: 0 4px 6px rgba(0,0,0,0.05);
        }
        
        .chart-title {
            font-size: 1.5rem;
            margin-bottom: 1rem;
            color: #2d3748;
        }
        
        .summary-stats {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-top: 2rem;
        }
        
        .stat-item {
            text-align: center;
            padding: 1rem;
            background: #f7fafc;
            border-radius: 8px;
        }
        
        .stat-value {
            font-size: 1.5rem;
            font-weight: bold;
            color: #4a5568;
        }
        
        .stat-label {
            color: #718096;
            font-size: 0.9rem;
            margin-top: 0.25rem;
        }
        
        .footer {
            text-align: center;
            padding: 2rem;
            color: #718096;
            border-top: 1px solid #e2e8f0;
            margin-top: 3rem;
        }
        
        @media (max-width: 768px) {
            .container {
                padding: 1rem;
            }
            
            .header h1 {
                font-size: 2rem;
            }
            
            .metrics-grid {
                grid-template-columns: 1fr;
            }
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🏗️ Build Metrics Dashboard</h1>
        <p>Bitcoin Custody System - Build Performance Analytics</p>
    </div>
    
    <div class="container">
        <div class="metrics-grid" id="metricsGrid">
            <!-- Metrics will be populated by JavaScript -->
        </div>
        
        <div class="chart-container">
            <h2 class="chart-title">📊 Build Time Trends</h2>
            <canvas id="buildTimeChart" width="400" height="200"></canvas>
        </div>
        
        <div class="chart-container">
            <h2 class="chart-title">📦 Bundle Size Analysis</h2>
            <canvas id="bundleSizeChart" width="400" height="200"></canvas>
        </div>
        
        <div class="summary-stats">
            <div class="stat-item">
                <div class="stat-value" id="totalBuildTime">--</div>
                <div class="stat-label">Total Build Time</div>
            </div>
            <div class="stat-item">
                <div class="stat-value" id="totalArtifactSize">--</div>
                <div class="stat-label">Total Artifact Size</div>
            </div>
            <div class="stat-item">
                <div class="stat-value" id="cacheEfficiency">--</div>
                <div class="stat-label">Cache Efficiency</div>
            </div>
            <div class="stat-item">
                <div class="stat-value" id="lastUpdated">--</div>
                <div class="stat-label">Last Updated</div>
            </div>
        </div>
    </div>
    
    <div class="footer">
        <p>Generated on <span id="generatedTime"></span> | Bitcoin Custody Build System</p>
    </div>
    
    <script>
        // Load and display metrics
        async function loadMetrics() {
            try {
                const response = await fetch('./latest-metrics.json');
                const data = await response.json();
                displayMetrics(data);
                createCharts(data);
            } catch (error) {
                console.error('Failed to load metrics:', error);
                displayErrorMessage();
            }
        }
        
        function displayMetrics(data) {
            const grid = document.getElementById('metricsGrid');
            const components = data.components || {};
            
            // Frontend metrics
            if (components.frontend) {
                const frontend = components.frontend;
                grid.innerHTML += createMetricCard(
                    'Frontend Build',
                    'frontend-icon',
                    frontend.buildTime?.totalFormatted || 'N/A',
                    'Build Time',
                    getStatusClass(frontend.buildTime?.total, 30)
                );
            }
            
            // Backend metrics
            if (components.backend) {
                const backend = components.backend;
                grid.innerHTML += createMetricCard(
                    'Backend Build',
                    'backend-icon',
                    backend.compilation_time?.formatted || 'N/A',
                    'Compilation Time',
                    getStatusClass(backend.compilation_time?.total_seconds, 60)
                );
            }
            
            // Soroban metrics
            if (components.soroban) {
                const soroban = components.soroban;
                grid.innerHTML += createMetricCard(
                    'Soroban Contracts',
                    'soroban-icon',
                    soroban.workspace_build_time?.formatted || 'N/A',
                    'Build Time',
                    getStatusClass(soroban.workspace_build_time?.seconds, 45)
                );
            }
            
            // Update summary stats
            updateSummaryStats(data);
        }
        
        function createMetricCard(title, iconClass, value, label, statusClass) {
            return `
                <div class="metric-card">
                    <h3><div class="icon ${iconClass}"></div>${title}</h3>
                    <div class="metric-value ${statusClass}">${value}</div>
                    <div class="metric-label">${label}</div>
                </div>
            `;
        }
        
        function getStatusClass(value, threshold) {
            if (!value) return '';
            if (value <= threshold) return 'status-good';
            if (value <= threshold * 2) return 'status-warning';
            return 'status-error';
        }
        
        function updateSummaryStats(data) {
            const components = data.components || {};
            
            // Calculate total build time
            let totalTime = 0;
            if (components.frontend?.buildTime?.total) totalTime += components.frontend.buildTime.total;
            if (components.backend?.compilation_time?.total_seconds) totalTime += components.backend.compilation_time.total_seconds;
            if (components.soroban?.workspace_build_time?.seconds) totalTime += components.soroban.workspace_build_time.seconds;
            
            document.getElementById('totalBuildTime').textContent = formatTime(totalTime);
            document.getElementById('lastUpdated').textContent = new Date(data.timestamp).toLocaleString();
            document.getElementById('generatedTime').textContent = new Date().toLocaleString();
        }
        
        function formatTime(seconds) {
            if (seconds < 60) return `${seconds}s`;
            const minutes = Math.floor(seconds / 60);
            const remainingSeconds = seconds % 60;
            return `${minutes}m ${remainingSeconds}s`;
        }
        
        function createCharts(data) {
            // Build time chart
            const buildTimeCtx = document.getElementById('buildTimeChart').getContext('2d');
            new Chart(buildTimeCtx, {
                type: 'bar',
                data: {
                    labels: ['Frontend', 'Backend', 'Soroban'],
                    datasets: [{
                        label: 'Build Time (seconds)',
                        data: [
                            data.components.frontend?.buildTime?.total || 0,
                            data.components.backend?.compilation_time?.total_seconds || 0,
                            data.components.soroban?.workspace_build_time?.seconds || 0
                        ],
                        backgroundColor: ['#61dafb', '#f74c00', '#ffd700'],
                        borderColor: ['#21a1c4', '#d63200', '#e6c200'],
                        borderWidth: 1
                    }]
                },
                options: {
                    responsive: true,
                    scales: {
                        y: {
                            beginAtZero: true,
                            title: {
                                display: true,
                                text: 'Time (seconds)'
                            }
                        }
                    }
                }
            });
        }
        
        function displayErrorMessage() {
            document.getElementById('metricsGrid').innerHTML = `
                <div class="metric-card">
                    <h3>⚠️ Error Loading Metrics</h3>
                    <p>Unable to load build metrics. Please ensure the metrics files are available.</p>
                </div>
            `;
        }
        
        // Load metrics on page load
        document.addEventListener('DOMContentLoaded', loadMetrics);
    </script>
</body>
</html>
EOF
    
    success "HTML dashboard generated at $HTML_REPORT"
}

# Generate summary report
generate_summary_report() {
    log "Generating summary report..."
    
    local summary_file="$REPORT_DIR/build-summary-$(date +%Y%m%d).md"
    
    cat > "$summary_file" << EOF
# Build Performance Summary

**Generated:** $(date)

## Component Performance

### Frontend
EOF
    
    if [ -f "frontend/build-performance-report.json" ]; then
        local frontend_data=$(cat frontend/build-performance-report.json | jq '.[-1]' 2>/dev/null)
        if [ "$frontend_data" != "null" ]; then
            echo "- Build Time: $(echo "$frontend_data" | jq -r '.buildTime.totalFormatted // "N/A"')" >> "$summary_file"
            echo "- Bundle Size: $(echo "$frontend_data" | jq -r '.bundleSize.totalFormatted // "N/A"')" >> "$summary_file"
            echo "- Cache Size: $(echo "$frontend_data" | jq -r '.cache.sizeFormatted // "N/A"')" >> "$summary_file"
        fi
    fi
    
    cat >> "$summary_file" << EOF

### Backend
EOF
    
    if [ -f "backend/build-performance-report.json" ]; then
        local backend_data=$(cat backend/build-performance-report.json | jq '.[-1]' 2>/dev/null)
        if [ "$backend_data" != "null" ]; then
            echo "- Compilation Time: $(echo "$backend_data" | jq -r '.compilation_time.formatted // "N/A"')" >> "$summary_file"
            echo "- Binary Size: $(echo "$backend_data" | jq -r '.binary_size.formatted // "N/A"')" >> "$summary_file"
            echo "- Incremental Time: $(echo "$backend_data" | jq -r '.incremental_time.formatted // "N/A"')" >> "$summary_file"
        fi
    fi
    
    cat >> "$summary_file" << EOF

### Soroban Contracts
EOF
    
    if [ -f "soroban/contract-build-performance.json" ]; then
        local soroban_data=$(cat soroban/contract-build-performance.json | jq '.[-1]' 2>/dev/null)
        if [ "$soroban_data" != "null" ]; then
            echo "- Workspace Build Time: $(echo "$soroban_data" | jq -r '.workspace_build_time.formatted // "N/A"')" >> "$summary_file"
            echo "- Contract Count: $(echo "$soroban_data" | jq '.contracts | length')" >> "$summary_file"
        fi
    fi
    
    cat >> "$summary_file" << EOF

## Recommendations

### Performance Optimizations
- Monitor build times and investigate any significant increases
- Regularly clean build caches to maintain optimal performance
- Consider parallel builds for independent components

### Cache Management
- Enable incremental compilation for development builds
- Use appropriate cache strategies for different environments
- Monitor cache effectiveness and adjust settings as needed

### Size Optimization
- Analyze bundle sizes regularly to identify bloat
- Implement code splitting and tree shaking
- Optimize WASM contract sizes for production deployment
EOF
    
    success "Summary report generated at $summary_file"
}

# Main function
main() {
    log "Starting build metrics dashboard generation..."
    
    create_report_dir
    collect_metrics
    generate_html_dashboard
    generate_summary_report
    
    success "Build metrics dashboard complete!"
    info "View the dashboard at: file://$(pwd)/$HTML_REPORT"
}

# Show help
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: $0"
    echo ""
    echo "Generates a comprehensive build metrics dashboard including:"
    echo "  - HTML dashboard with interactive charts"
    echo "  - Summary report in Markdown format"
    echo "  - Performance trends and recommendations"
    echo ""
    echo "The dashboard will be generated in the 'build-reports' directory."
    exit 0
fi

# Check dependencies
if ! command -v jq &> /dev/null; then
    error "jq is required for JSON processing. Please install it."
    exit 1
fi

# Run main function
main