import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],

  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  },

  // Vite options tailored for Tauri development
  clearScreen: false,

  // Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,
    strictPort: true,
    host: 'localhost',
    watch: {
      ignored: ['**/src-tauri/**']
    }
  },

  // to make use of `TAURI_DEBUG` and other env variables
  envPrefix: ['VITE_', 'TAURI_'],

  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: process.env.TAURI_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
    // 多入口配置
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        launcher: resolve(__dirname, 'launcher.html'),
        'sql-panel': resolve(__dirname, 'sql-panel.html'),
        'quick-task': resolve(__dirname, 'quick-task.html'),
        'task-float': resolve(__dirname, 'task-float.html'),
        'task-calendar': resolve(__dirname, 'task-calendar.html'),
        'clipboard-history': resolve(__dirname, 'clipboard-history.html')
      }
    }
  }
})
