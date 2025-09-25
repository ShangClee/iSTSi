#!/usr/bin/env node

/**
 * Build Performance Monitor
 * Tracks and reports build performance metrics for optimization
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

class BuildPerformanceMonitor {
  constructor() {
    this.metrics = {
      timestamp: new Date().toISOString(),
      buildId: Date.now().toString(),
      environment: process.env.NODE_ENV || 'development',
    };
    this.startTime = Date.now();
  }

  // Measure build time
  measureBuildTime() {
    const buildStart = Date.now();
    
    try {
      console.log('🚀 Starting build performance measurement...');
      
      // Run the build
      execSync('npm run build:fast', { 
        stdio: 'inherit',
        env: { ...process.env, NODE_ENV: 'production' }
      });
      
      const buildEnd = Date.now();
      const buildTime = buildEnd - buildStart;
      
      this.metrics.buildTime = {
        total: buildTime,
        totalFormatted: this.formatTime(buildTime),
      };
      
      console.log(`✅ Build completed in ${this.formatTime(buildTime)}`);
      
    } catch (error) {
      console.error('❌ Build failed:', error.message);
      this.metrics.buildError = error.message;
    }
  }

  // Analyze bundle size
  analyzeBundleSize() {
    const distPath = path.join(__dirname, '../dist');
    
    if (!fs.existsSync(distPath)) {
      console.warn('⚠️  Dist directory not found, skipping bundle analysis');
      return;
    }

    console.log('📊 Analyzing bundle size...');
    
    const bundleStats = this.getDirectoryStats(distPath);
    
    this.metrics.bundleSize = {
      total: bundleStats.size,
      totalFormatted: this.formatBytes(bundleStats.size),
      files: bundleStats.files,
      breakdown: this.analyzeBundleBreakdown(distPath),
    };
    
    console.log(`📦 Total bundle size: ${this.formatBytes(bundleStats.size)}`);
    console.log(`📄 Total files: ${bundleStats.files}`);
  }

  // Get directory statistics
  getDirectoryStats(dirPath) {
    let totalSize = 0;
    let totalFiles = 0;
    
    const items = fs.readdirSync(dirPath);
    
    for (const item of items) {
      const itemPath = path.join(dirPath, item);
      const stats = fs.statSync(itemPath);
      
      if (stats.isDirectory()) {
        const subStats = this.getDirectoryStats(itemPath);
        totalSize += subStats.size;
        totalFiles += subStats.files;
      } else {
        totalSize += stats.size;
        totalFiles++;
      }
    }
    
    return { size: totalSize, files: totalFiles };
  }

  // Analyze bundle breakdown by file type
  analyzeBundleBreakdown(distPath) {
    const breakdown = {
      js: { size: 0, files: 0 },
      css: { size: 0, files: 0 },
      images: { size: 0, files: 0 },
      fonts: { size: 0, files: 0 },
      other: { size: 0, files: 0 },
    };

    this.walkDirectory(distPath, (filePath, stats) => {
      const ext = path.extname(filePath).toLowerCase();
      
      if (['.js', '.mjs', '.ts'].includes(ext)) {
        breakdown.js.size += stats.size;
        breakdown.js.files++;
      } else if (['.css', '.scss', '.sass'].includes(ext)) {
        breakdown.css.size += stats.size;
        breakdown.css.files++;
      } else if (['.png', '.jpg', '.jpeg', '.gif', '.svg', '.webp'].includes(ext)) {
        breakdown.images.size += stats.size;
        breakdown.images.files++;
      } else if (['.woff', '.woff2', '.ttf', '.eot'].includes(ext)) {
        breakdown.fonts.size += stats.size;
        breakdown.fonts.files++;
      } else {
        breakdown.other.size += stats.size;
        breakdown.other.files++;
      }
    });

    // Format breakdown
    Object.keys(breakdown).forEach(key => {
      breakdown[key].sizeFormatted = this.formatBytes(breakdown[key].size);
    });

    return breakdown;
  }

  // Walk directory recursively
  walkDirectory(dirPath, callback) {
    const items = fs.readdirSync(dirPath);
    
    for (const item of items) {
      const itemPath = path.join(dirPath, item);
      const stats = fs.statSync(itemPath);
      
      if (stats.isDirectory()) {
        this.walkDirectory(itemPath, callback);
      } else {
        callback(itemPath, stats);
      }
    }
  }

  // Measure dependency installation time
  measureDependencyTime() {
    console.log('📦 Measuring dependency installation time...');
    
    // Clear node_modules and package-lock.json
    try {
      execSync('rm -rf node_modules package-lock.json', { stdio: 'pipe' });
    } catch (error) {
      // Ignore errors if files don't exist
    }
    
    const depStart = Date.now();
    
    try {
      execSync('npm ci', { stdio: 'inherit' });
      
      const depEnd = Date.now();
      const depTime = depEnd - depStart;
      
      this.metrics.dependencyTime = {
        total: depTime,
        totalFormatted: this.formatTime(depTime),
      };
      
      console.log(`✅ Dependencies installed in ${this.formatTime(depTime)}`);
      
    } catch (error) {
      console.error('❌ Dependency installation failed:', error.message);
      this.metrics.dependencyError = error.message;
    }
  }

  // Analyze cache effectiveness
  analyzeCacheEffectiveness() {
    console.log('🗄️  Analyzing cache effectiveness...');
    
    const cacheDir = path.join(__dirname, '../node_modules/.vite');
    
    if (fs.existsSync(cacheDir)) {
      const cacheStats = this.getDirectoryStats(cacheDir);
      
      this.metrics.cache = {
        size: cacheStats.size,
        sizeFormatted: this.formatBytes(cacheStats.size),
        files: cacheStats.files,
        exists: true,
      };
      
      console.log(`💾 Cache size: ${this.formatBytes(cacheStats.size)}`);
    } else {
      this.metrics.cache = {
        exists: false,
      };
      console.log('⚠️  No cache directory found');
    }
  }

  // Get system information
  getSystemInfo() {
    try {
      const os = require('os');
      
      this.metrics.system = {
        platform: os.platform(),
        arch: os.arch(),
        cpus: os.cpus().length,
        memory: {
          total: os.totalmem(),
          totalFormatted: this.formatBytes(os.totalmem()),
          free: os.freemem(),
          freeFormatted: this.formatBytes(os.freemem()),
        },
        node: process.version,
      };
      
    } catch (error) {
      console.warn('⚠️  Could not gather system info:', error.message);
    }
  }

  // Generate performance report
  generateReport() {
    const totalTime = Date.now() - this.startTime;
    this.metrics.totalTime = {
      total: totalTime,
      totalFormatted: this.formatTime(totalTime),
    };

    const reportPath = path.join(__dirname, '../build-performance-report.json');
    
    // Load previous reports for comparison
    let previousReports = [];
    if (fs.existsSync(reportPath)) {
      try {
        const existingData = fs.readFileSync(reportPath, 'utf8');
        previousReports = JSON.parse(existingData);
      } catch (error) {
        console.warn('⚠️  Could not load previous reports:', error.message);
      }
    }

    // Add current report
    previousReports.push(this.metrics);
    
    // Keep only last 10 reports
    if (previousReports.length > 10) {
      previousReports = previousReports.slice(-10);
    }

    // Save reports
    fs.writeFileSync(reportPath, JSON.stringify(previousReports, null, 2));
    
    console.log('\n📊 Performance Report Generated:');
    console.log('================================');
    console.log(`Build Time: ${this.metrics.buildTime?.totalFormatted || 'N/A'}`);
    console.log(`Bundle Size: ${this.metrics.bundleSize?.totalFormatted || 'N/A'}`);
    console.log(`Cache Size: ${this.metrics.cache?.sizeFormatted || 'N/A'}`);
    console.log(`Total Time: ${this.metrics.totalTime.totalFormatted}`);
    console.log(`Report saved to: ${reportPath}`);
    
    // Show performance trends
    this.showPerformanceTrends(previousReports);
  }

  // Show performance trends
  showPerformanceTrends(reports) {
    if (reports.length < 2) {
      console.log('\n📈 Not enough data for trend analysis');
      return;
    }

    const current = reports[reports.length - 1];
    const previous = reports[reports.length - 2];

    console.log('\n📈 Performance Trends:');
    console.log('=====================');

    if (current.buildTime && previous.buildTime) {
      const buildTimeDiff = current.buildTime.total - previous.buildTime.total;
      const buildTimePercent = ((buildTimeDiff / previous.buildTime.total) * 100).toFixed(1);
      const buildTrend = buildTimeDiff > 0 ? '📈' : '📉';
      console.log(`Build Time: ${buildTrend} ${buildTimePercent}% (${this.formatTime(Math.abs(buildTimeDiff))})`);
    }

    if (current.bundleSize && previous.bundleSize) {
      const bundleSizeDiff = current.bundleSize.total - previous.bundleSize.total;
      const bundleSizePercent = ((bundleSizeDiff / previous.bundleSize.total) * 100).toFixed(1);
      const bundleTrend = bundleSizeDiff > 0 ? '📈' : '📉';
      console.log(`Bundle Size: ${bundleTrend} ${bundleSizePercent}% (${this.formatBytes(Math.abs(bundleSizeDiff))})`);
    }
  }

  // Format time in human readable format
  formatTime(ms) {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
  }

  // Format bytes in human readable format
  formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  }

  // Run all performance measurements
  async run() {
    console.log('🔍 Starting Build Performance Analysis');
    console.log('=====================================\n');

    this.getSystemInfo();
    this.analyzeCacheEffectiveness();
    this.measureBuildTime();
    this.analyzeBundleSize();
    
    this.generateReport();
    
    console.log('\n✅ Performance analysis complete!');
  }
}

// Run if called directly
if (require.main === module) {
  const monitor = new BuildPerformanceMonitor();
  monitor.run().catch(console.error);
}

module.exports = BuildPerformanceMonitor;