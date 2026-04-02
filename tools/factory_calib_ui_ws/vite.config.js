import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    host: '0.0.0.0',
    port: 18110,
  },
  preview: {
    host: '0.0.0.0',
    port: 18110,
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
});
