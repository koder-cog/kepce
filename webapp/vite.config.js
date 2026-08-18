import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import fs from 'fs';
import path from 'path';

function kepceIconChecker() {
  return {
    name: 'kepce-icon-checker',
    enforce: 'pre',
    buildStart() {
      const iconsPath = path.resolve('src/components/ui/icons.js');
      if (fs.existsSync(iconsPath)) {
        const content = fs.readFileSync(iconsPath, 'utf-8');
        const inlineSvgRegex = /([a-zA-Z0-9_-]+)\s*:\s*['"`]<svg/g;
        let match;
        while ((match = inlineSvgRegex.exec(content)) !== null) {
          console.warn(`\n[Kepçe Uyarı] Gömülü (inline) SVG bulundu: "${match[1]}". Lütfen bu ikonu gerçek bir .svg dosyasına ayırın!`);
        }
      }
    }
  };
}

export default defineConfig({
  plugins: [kepceIconChecker(), sveltekit()],

  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('node_modules')) {
            return 'vendor';
          }
        }
      }
    }
  },

  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:8000',
        changeOrigin: true,
      },
      '/static': {
        target: 'http://localhost:8000',
        changeOrigin: true,
      },
      '/rss.xml': {
        target: 'http://localhost:8000',
        changeOrigin: true,
      },
    },
  },
  
  preview: {
    port: 4173,
    proxy: {
      '/api': {
        target: 'http://localhost:8000',
        changeOrigin: true,
      },
      '/static': {
        target: 'http://localhost:8000',
        changeOrigin: true,
      },
      '/rss.xml': {
        target: 'http://localhost:8000',
        changeOrigin: true,
      },
    },
  },
});
