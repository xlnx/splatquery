import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { VitePWA } from 'vite-plugin-pwa'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    // VitePWA({
    //   // injectRegister: 'auto',
    //   injectRegister: 'script',
    //   strategies: 'injectManifest',
    //   srcDir: 'src',
    //   filename: 'sw.js',
    //   registerType: 'autoUpdate',
    //   injectManifest: {
    //     rollupFormat: 'iife',
    //   },
    //   devOptions: {
    //     enabled: true
    //   },
    // })
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  build: {
    sourcemap: true
  }
})
