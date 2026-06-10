import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    tailwindcss(),
    svelte(),
  ],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      // episteme api server (entity, search, health, stats)
      '/api/v1': {
        target: 'http://localhost:58302',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/v1/, ''),
      },
      // episteme web server (graph visualization)
      '/api/web': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/web/, ''),
      },
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
});
