import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { VitePWA } from 'vite-plugin-pwa'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    VitePWA({
      srcDir: 'src',
      outDir: 'dist',
      filename: 'sw.js',
      strategies: 'injectManifest',
      registerType: "autoUpdate",
      includeAssets: [
        'img/**/*.png', 
        'img/**/*.svg', 
        'font/*.woff2', 
        'font/*.woff'
      ],
      includeManifestIcons: true,
      manifest: {
        name: "SplatQuery",
        short_name: "SplatQuery",
        start_url: "/?mode=standalone",
        orientation: "portrait-primary",
        display: "standalone",
        theme_color: "#101827",
        background_color: "#101827",
        description: "Get notified about your favourite gear/schedule updates of splatoon 3.",
        icons: [
          {
            src: "/logo-48.png",
            sizes: "48x48",
            type: "image/png"
          },
          {
            src: "/logo-72.png",
            sizes: "72x72",
            type: "image/png"
          },
          {
            src: "/logo-96.png",
            sizes: "96x96",
            type: "image/png"
          },
          {
            src: "/logo-128.png",
            sizes: "128x128",
            type: "image/png"
          },
          {
            src: "/logo-256.png",
            sizes: "256x256",
            type: "image/png"
          },
          {
            src: "/logo-512.png",
            sizes: "512x512",
            type: "image/png"
          }
        ],
        shortcuts: [
          {
            "name": "Queries",
            "url": "/query/list"
          },
          {
            "name": "Actions",
            "url": "/action/list"
          },
          {
            "name": "Settings",
            "url": "/settings"
          },
          {
            "name": "New Query",
            "url": "/query/new"
          }
        ],
        serviceworker: {
          scope: "/",
          src: "/sw.js"
        }
      }
    })
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
