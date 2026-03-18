import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import AutoImport from "unplugin-auto-import/vite";
import Components from "unplugin-vue-components/vite";
import { ElementPlusResolver } from "unplugin-vue-components/resolvers";
import * as Path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = Path.dirname(__filename);

// 获取当前时间作为构建时间
const buildTime = new Date().toISOString().slice(0, 19).replace('T', ' ');

  // 生产构建时 NODE_ENV=production
  const isProd = process.env.NODE_ENV === 'production';

  // https://vite.dev/config/
  export default defineConfig({
    root: 'src',
    base: '/',
    envDir: Path.resolve(__dirname), // 指定环境变量文件目录为项目根目录
    plugins: [
      vue(),
      AutoImport({
        resolvers: [ElementPlusResolver()],
      }),
      Components({
        resolvers: [ElementPlusResolver()],
      }),
    ],

    define: {
      'import.meta.env.VITE_APP_VERSION': JSON.stringify(process.env.npm_package_version || '1.0.0'),
      'import.meta.env.VITE_APP_BUILD_TIME': JSON.stringify(buildTime),
      'import.meta.env.VITE_SERVER_HOST': JSON.stringify(process.env.VITE_SERVER_HOST || 'http://43.136.218.87'),
      'import.meta.env.VITE_DEV_MODE': JSON.stringify(process.env.VITE_DEV_MODE || 'false'),
      'import.meta.env.PROD': JSON.stringify(isProd),
    },

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
    // 配置代理，解决图片等静态资源请求和API请求
    proxy: {
      '/resource': {
        target: 'http://43.136.218.87',
        changeOrigin: true,
        rewrite: (path) => path,
      },
      '/api/': {
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
