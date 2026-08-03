import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import path from 'path';

export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait()],
  base: '/WorldSmith/',
  resolve: {
    alias: {
      'worldsmith-wasm': path.resolve(__dirname, '../wasm/pkg'),
    },
  },
});
