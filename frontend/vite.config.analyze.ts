import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react-swc';
import { visualizer } from 'rollup-plugin-visualizer';
import { analyzer } from 'vite-bundle-analyzer';
import path from 'path';

export default defineConfig({
  plugins: [
    react(),
    
    // Bundle analyzer with detailed statistics
    visualizer({
      filename: 'dist/bundle-analysis.html',
      open: false,
      gzipSize: true,
      brotliSize: true,
      template: 'treemap',
      title: 'Bitcoin Custody Frontend Bundle Analysis',
    }),
    
    // Alternative bundle analyzer
    analyzer({
      analyzerMode: 'static',
      reportFilename: 'dist/bundle-report.html',
      openAnalyzer: false,
      generateStatsFile: true,
      statsFilename: 'dist/bundle-stats.json',
    }),
  ],

  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@/components': path.resolve(__dirname, './src/components'),
      '@/services': path.resolve(__dirname, './src/services'),
      '@/hooks': path.resolve(__dirname, './src/hooks'),
      '@/store': path.resolve(__dirname, './src/store'),
      '@/types': path.resolve(__dirname, './src/types'),
      '@/utils': path.resolve(__dirname, './src/utils'),
    },
  },

  build: {
    outDir: 'dist',
    sourcemap: true,
    minify: 'esbuild',
    target: 'es2020',
    
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          // Detailed chunk splitting for analysis
          if (id.includes('node_modules')) {
            if (id.includes('react') || id.includes('react-dom')) {
              return 'react-vendor';
            }
            if (id.includes('@radix-ui')) {
              return 'ui-vendor';
            }
            if (id.includes('@reduxjs') || id.includes('react-redux')) {
              return 'state-vendor';
            }
            if (id.includes('axios') || id.includes('socket.io')) {
              return 'network-vendor';
            }
            if (id.includes('date-fns') || id.includes('lodash')) {
              return 'utils-vendor';
            }
            return 'vendor';
          }
          
          // Split by feature
          if (id.includes('/components/')) {
            return 'components';
          }
          if (id.includes('/services/')) {
            return 'services';
          }
          if (id.includes('/store/')) {
            return 'store';
          }
          if (id.includes('/hooks/')) {
            return 'hooks';
          }
        },
      },
    },
    
    // Generate detailed build reports
    reportCompressedSize: true,
    chunkSizeWarningLimit: 500,
  },

  // Enable detailed analysis
  define: {
    __BUNDLE_ANALYSIS__: true,
  },
});