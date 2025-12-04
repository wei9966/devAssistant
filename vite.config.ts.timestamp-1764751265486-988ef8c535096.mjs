// vite.config.ts
import { defineConfig } from "file:///E:/%E5%85%B6%E4%BB%96%E4%BB%A3%E7%A0%81/devAssistant/node_modules/vite/dist/node/index.js";
import vue from "file:///E:/%E5%85%B6%E4%BB%96%E4%BB%A3%E7%A0%81/devAssistant/node_modules/@vitejs/plugin-vue/dist/index.mjs";
import { resolve } from "path";
var __vite_injected_original_dirname = "E:\\\u5176\u4ED6\u4EE3\u7801\\devAssistant";
var vite_config_default = defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": resolve(__vite_injected_original_dirname, "src")
    }
  },
  // Vite options tailored for Tauri development
  clearScreen: false,
  // Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,
    strictPort: true,
    host: "localhost",
    watch: {
      ignored: ["**/src-tauri/**"]
    }
  },
  // to make use of `TAURI_DEBUG` and other env variables
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: process.env.TAURI_PLATFORM === "windows" ? "chrome105" : "safari13",
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
    // 多入口配置
    rollupOptions: {
      input: {
        main: resolve(__vite_injected_original_dirname, "index.html"),
        launcher: resolve(__vite_injected_original_dirname, "launcher.html"),
        "sql-panel": resolve(__vite_injected_original_dirname, "sql-panel.html"),
        "quick-task": resolve(__vite_injected_original_dirname, "quick-task.html"),
        "task-float": resolve(__vite_injected_original_dirname, "task-float.html"),
        "task-calendar": resolve(__vite_injected_original_dirname, "task-calendar.html"),
        "clipboard-history": resolve(__vite_injected_original_dirname, "clipboard-history.html")
      }
    }
  }
});
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcudHMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCJFOlxcXFxcdTUxNzZcdTRFRDZcdTRFRTNcdTc4MDFcXFxcZGV2QXNzaXN0YW50XCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ZpbGVuYW1lID0gXCJFOlxcXFxcdTUxNzZcdTRFRDZcdTRFRTNcdTc4MDFcXFxcZGV2QXNzaXN0YW50XFxcXHZpdGUuY29uZmlnLnRzXCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ltcG9ydF9tZXRhX3VybCA9IFwiZmlsZTovLy9FOi8lRTUlODUlQjYlRTQlQkIlOTYlRTQlQkIlQTMlRTclQTAlODEvZGV2QXNzaXN0YW50L3ZpdGUuY29uZmlnLnRzXCI7aW1wb3J0IHsgZGVmaW5lQ29uZmlnIH0gZnJvbSAndml0ZSdcclxuaW1wb3J0IHZ1ZSBmcm9tICdAdml0ZWpzL3BsdWdpbi12dWUnXHJcbmltcG9ydCB7IHJlc29sdmUgfSBmcm9tICdwYXRoJ1xyXG5cclxuLy8gaHR0cHM6Ly92aXRlanMuZGV2L2NvbmZpZy9cclxuZXhwb3J0IGRlZmF1bHQgZGVmaW5lQ29uZmlnKHtcclxuICBwbHVnaW5zOiBbdnVlKCldLFxyXG5cclxuICByZXNvbHZlOiB7XHJcbiAgICBhbGlhczoge1xyXG4gICAgICAnQCc6IHJlc29sdmUoX19kaXJuYW1lLCAnc3JjJylcclxuICAgIH1cclxuICB9LFxyXG5cclxuICAvLyBWaXRlIG9wdGlvbnMgdGFpbG9yZWQgZm9yIFRhdXJpIGRldmVsb3BtZW50XHJcbiAgY2xlYXJTY3JlZW46IGZhbHNlLFxyXG5cclxuICAvLyBUYXVyaSBleHBlY3RzIGEgZml4ZWQgcG9ydCwgZmFpbCBpZiB0aGF0IHBvcnQgaXMgbm90IGF2YWlsYWJsZVxyXG4gIHNlcnZlcjoge1xyXG4gICAgcG9ydDogNTE3MyxcclxuICAgIHN0cmljdFBvcnQ6IHRydWUsXHJcbiAgICBob3N0OiAnbG9jYWxob3N0JyxcclxuICAgIHdhdGNoOiB7XHJcbiAgICAgIGlnbm9yZWQ6IFsnKiovc3JjLXRhdXJpLyoqJ11cclxuICAgIH1cclxuICB9LFxyXG5cclxuICAvLyB0byBtYWtlIHVzZSBvZiBgVEFVUklfREVCVUdgIGFuZCBvdGhlciBlbnYgdmFyaWFibGVzXHJcbiAgZW52UHJlZml4OiBbJ1ZJVEVfJywgJ1RBVVJJXyddLFxyXG5cclxuICBidWlsZDoge1xyXG4gICAgLy8gVGF1cmkgdXNlcyBDaHJvbWl1bSBvbiBXaW5kb3dzIGFuZCBXZWJLaXQgb24gbWFjT1MgYW5kIExpbnV4XHJcbiAgICB0YXJnZXQ6IHByb2Nlc3MuZW52LlRBVVJJX1BMQVRGT1JNID09PSAnd2luZG93cycgPyAnY2hyb21lMTA1JyA6ICdzYWZhcmkxMycsXHJcbiAgICAvLyBkb24ndCBtaW5pZnkgZm9yIGRlYnVnIGJ1aWxkc1xyXG4gICAgbWluaWZ5OiAhcHJvY2Vzcy5lbnYuVEFVUklfREVCVUcgPyAnZXNidWlsZCcgOiBmYWxzZSxcclxuICAgIC8vIHByb2R1Y2Ugc291cmNlbWFwcyBmb3IgZGVidWcgYnVpbGRzXHJcbiAgICBzb3VyY2VtYXA6ICEhcHJvY2Vzcy5lbnYuVEFVUklfREVCVUcsXHJcbiAgICAvLyBcdTU5MUFcdTUxNjVcdTUzRTNcdTkxNERcdTdGNkVcclxuICAgIHJvbGx1cE9wdGlvbnM6IHtcclxuICAgICAgaW5wdXQ6IHtcclxuICAgICAgICBtYWluOiByZXNvbHZlKF9fZGlybmFtZSwgJ2luZGV4Lmh0bWwnKSxcclxuICAgICAgICBsYXVuY2hlcjogcmVzb2x2ZShfX2Rpcm5hbWUsICdsYXVuY2hlci5odG1sJyksXHJcbiAgICAgICAgJ3NxbC1wYW5lbCc6IHJlc29sdmUoX19kaXJuYW1lLCAnc3FsLXBhbmVsLmh0bWwnKSxcclxuICAgICAgICAncXVpY2stdGFzayc6IHJlc29sdmUoX19kaXJuYW1lLCAncXVpY2stdGFzay5odG1sJyksXHJcbiAgICAgICAgJ3Rhc2stZmxvYXQnOiByZXNvbHZlKF9fZGlybmFtZSwgJ3Rhc2stZmxvYXQuaHRtbCcpLFxyXG4gICAgICAgICd0YXNrLWNhbGVuZGFyJzogcmVzb2x2ZShfX2Rpcm5hbWUsICd0YXNrLWNhbGVuZGFyLmh0bWwnKSxcclxuICAgICAgICAnY2xpcGJvYXJkLWhpc3RvcnknOiByZXNvbHZlKF9fZGlybmFtZSwgJ2NsaXBib2FyZC1oaXN0b3J5Lmh0bWwnKVxyXG4gICAgICB9XHJcbiAgICB9XHJcbiAgfVxyXG59KVxyXG4iXSwKICAibWFwcGluZ3MiOiAiO0FBQW9SLFNBQVMsb0JBQW9CO0FBQ2pULE9BQU8sU0FBUztBQUNoQixTQUFTLGVBQWU7QUFGeEIsSUFBTSxtQ0FBbUM7QUFLekMsSUFBTyxzQkFBUSxhQUFhO0FBQUEsRUFDMUIsU0FBUyxDQUFDLElBQUksQ0FBQztBQUFBLEVBRWYsU0FBUztBQUFBLElBQ1AsT0FBTztBQUFBLE1BQ0wsS0FBSyxRQUFRLGtDQUFXLEtBQUs7QUFBQSxJQUMvQjtBQUFBLEVBQ0Y7QUFBQTtBQUFBLEVBR0EsYUFBYTtBQUFBO0FBQUEsRUFHYixRQUFRO0FBQUEsSUFDTixNQUFNO0FBQUEsSUFDTixZQUFZO0FBQUEsSUFDWixNQUFNO0FBQUEsSUFDTixPQUFPO0FBQUEsTUFDTCxTQUFTLENBQUMsaUJBQWlCO0FBQUEsSUFDN0I7QUFBQSxFQUNGO0FBQUE7QUFBQSxFQUdBLFdBQVcsQ0FBQyxTQUFTLFFBQVE7QUFBQSxFQUU3QixPQUFPO0FBQUE7QUFBQSxJQUVMLFFBQVEsUUFBUSxJQUFJLG1CQUFtQixZQUFZLGNBQWM7QUFBQTtBQUFBLElBRWpFLFFBQVEsQ0FBQyxRQUFRLElBQUksY0FBYyxZQUFZO0FBQUE7QUFBQSxJQUUvQyxXQUFXLENBQUMsQ0FBQyxRQUFRLElBQUk7QUFBQTtBQUFBLElBRXpCLGVBQWU7QUFBQSxNQUNiLE9BQU87QUFBQSxRQUNMLE1BQU0sUUFBUSxrQ0FBVyxZQUFZO0FBQUEsUUFDckMsVUFBVSxRQUFRLGtDQUFXLGVBQWU7QUFBQSxRQUM1QyxhQUFhLFFBQVEsa0NBQVcsZ0JBQWdCO0FBQUEsUUFDaEQsY0FBYyxRQUFRLGtDQUFXLGlCQUFpQjtBQUFBLFFBQ2xELGNBQWMsUUFBUSxrQ0FBVyxpQkFBaUI7QUFBQSxRQUNsRCxpQkFBaUIsUUFBUSxrQ0FBVyxvQkFBb0I7QUFBQSxRQUN4RCxxQkFBcUIsUUFBUSxrQ0FBVyx3QkFBd0I7QUFBQSxNQUNsRTtBQUFBLElBQ0Y7QUFBQSxFQUNGO0FBQ0YsQ0FBQzsiLAogICJuYW1lcyI6IFtdCn0K
