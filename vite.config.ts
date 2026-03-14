import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import AutoImport from "unplugin-auto-import/vite";
import Components from "unplugin-vue-components/vite";
import { ElementPlusResolver } from "unplugin-vue-components/resolvers";
import * as Path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = Path.dirname(__filename);

// https://vite.dev/config/
export default defineConfig({
  root: 'src',
  base: './',
  plugins: [
    vue(),
    AutoImport({
      resolvers: [ElementPlusResolver()],
    }),
    Components({
      resolvers: [ElementPlusResolver()],
    }),
  ],

  css: {
    preprocessorOptions: {
      less: {
        javascriptEnabled: true,
      },
    },
  },

  resolve: {
    extensions: [".ts", ".js", ".json", ".vue"],
  },

  build: {
    outDir: Path.resolve(__dirname, 'dist'),
    emptyOutDir: true,
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,
    strictPort: false,
    host: '0.0.0.0',
    hmr: {
      protocol: 'http',
      host: '10.0.2.2',
      port: 5173,
    },
    // 配置代理，解决图片等静态资源请求
    proxy: {
      '/resource': {
        target: 'http://43.136.218.87',
        changeOrigin: true,
        rewrite: (path) => path,
      },
    },
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
});
