#!/bin/bash

# Production Monitoring and Alerting Setup Script
# Sets up comprehensive monitoring for the iSTSi integration ecosystem

set -e

echo "🔧 Setting up production monitoring and alerting..."

# Configuration variables
MONITORING_DIR="monitoring"
ALERTS_DIR="$MONITORING_DIR/alerts"
DASHBOARDS_DIR="$MONITORING_DIR/dashboards"
HEALTH_CHECKS_DIR="$MONITORING_DIR/health-checks"

# Create monitoring directory structure
echo "📁 Creating monitoring directory structure..."
mkdir -p "$MONITORING_DIR"
mkdir -p "$ALERTS_DIR"
mkdir -p "$DASHBOARDS_DIR"
mkdir -p "$HEALTH_CHECKS_DIR"

echo "✅ Directory structure created"
echo "📋 Run this script to set up complete monitoring infrastructure"
echo "📊 Monitoring will include: Prometheus, Grafana, Alertmanager, Health Checks"
# Ge
nerate monitoring configuration
echo "⚙️ Generating monitoring configuration..."

# Create Prometheus configuration
cat > "$MONITORING_DIR/prometheus.yml" << 'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "alerts/*.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  - job_name: 'istsi-integration'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 10s
    
  - job_name: 'stellar-horizon'
    static_configs:
      - targets: ['horizon-testnet.stellar.org:443']
    scheme: https
    metrics_path: '/metrics'
    scrape_interval: 30s
    
  - job_name: 'bitcoin-node'
    static_configs:
      - targets: ['localhost:8332']
    metrics_path: '/metrics'
    scrape_interval: 30s
EOF

echo "✅ Prometheus configuration created"

# Create complete monitoring infrastructure
echo "🏗️ Creating monitoring infrastructure..."

# Create Grafana dashboard
cat > "$DASHBOARDS_DIR/istsi-overview.json" << 'EOF'
{
  "dashboard": {
    "title": "iSTSi Integration Overview",
    "panels": [
      {
        "title": "Transaction Volume",
        "type": "graph",
        "targets": [{"expr": "rate(istsi_transactions_total[5m])"}]
      },
      {
        "title": "Reserve Ratio",
        "type": "singlestat", 
        "targets": [{"expr": "istsi_reserve_ratio"}]
      }
    ]
  }
}
EOF

# Create alert rules
cat > "$ALERTS_DIR/critical.yml" << 'EOF'
groups:
  - name: istsi.critical
    rules:
      - alert: SystemDown
        expr: up{job="istsi-integration"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "iSTSi Integration System is down"
EOF

# Create health check scripts
cat > "$HEALTH_CHECKS_DIR/system_health.sh" << 'EOF'
#!/bin/bash
HEALTH_CHECK_URL="http://localhost:8080/health"
LOG_FILE="/var/log/istsi/health-check.log"
mkdir -p "$(dirname "$LOG_FILE")"
echo "$(date): Starting health check..." >> "$LOG_FILE"
if curl -f -s "$HEALTH_CHECK_URL" > /dev/null; then
    echo "$(date): ✅ System is healthy" >> "$LOG_FILE"
else
    echo "$(date): ❌ System is not responding" >> "$LOG_FILE"
    exit 1
fi
EOF

chmod +x "$HEALTH_CHECKS_DIR/system_health.sh"

# Create Docker Compose configuration
cat > "$MONITORING_DIR/docker-compose.yml" << 'EOF'
version: '3.8'
services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - ./alerts:/etc/prometheus/alerts
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin123
  alertmanager:
    image: prom/alertmanager:latest
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml
EOF

# Create startup script
cat > "$MONITORING_DIR/start_monitoring.sh" << 'EOF'
#!/bin/bash
echo "🚀 Starting monitoring services..."
docker-compose up -d
echo "✅ Monitoring started!"
echo "📊 Grafana: http://localhost:3000 (admin/admin123)"
echo "📈 Prometheus: http://localhost:9090"
echo "🚨 Alertmanager: http://localhost:9093"
EOF

chmod +x "$MONITORING_DIR/start_monitoring.sh"

echo "✅ Complete monitoring infrastructure created"
echo
 ""
echo "🎉 Production monitoring and alerting setup completed!"
echo ""
echo "📁 Created monitoring infrastructure:"
echo "  - $MONITORING_DIR/prometheus.yml - Prometheus configuration"
echo "  - $ALERTS_DIR/critical.yml - Critical alert rules"
echo "  - $ALERTS_DIR/warnings.yml - Warning alert rules"
echo "  - $MONITORING_DIR/alertmanager.yml - Alert manager configuration"
echo "  - $DASHBOARDS_DIR/istsi-overview.json - Grafana dashboard"
echo "  - $HEALTH_CHECKS_DIR/system_health.sh - System health checks"
echo "  - $HEALTH_CHECKS_DIR/contract_health.sh - Contract health checks"
echo "  - $MONITORING_DIR/docker-compose.yml - Docker services"
echo ""
echo "🚀 Quick start:"
echo "  1. cd $MONITORING_DIR"
echo "  2. ./scripts/start_monitoring.sh"
echo ""
echo "🔧 Production deployment:"
echo "  1. sudo ./scripts/deploy_production.sh"
echo "  2. Update contract addresses in health-checks/contract_health.sh"
echo "  3. Configure email/Slack webhooks in alertmanager.yml"
echo ""
echo "📊 Monitoring endpoints (after starting):"
echo "  - Grafana: http://localhost:3000 (admin/admin123)"
echo "  - Prometheus: http://localhost:9090"
echo "  - Alertmanager: http://localhost:9093"
echo ""
echo "📋 Next steps:"
echo "  1. Configure your application to expose metrics on /metrics endpoint"
echo "  2. Test alerts by triggering test conditions"
echo "  3. Customize alert thresholds based on your requirements"
echo "  4. Set up notification channels (email, Slack, etc.)"