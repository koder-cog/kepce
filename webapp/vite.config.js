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

function spaPreviewFallback() {
  return {
    name: 'spa-preview-fallback',
    configurePreviewServer(server) {
      server.middlewares.use((req, res, next) => {
        const url = req.url ? req.url.split('?')[0] : '';
        if (
          url.startsWith('/api') ||
          url.startsWith('/static') ||
          url.startsWith('/rss.xml') ||
          url.startsWith('/internal')
        ) {
          return next();
        }

        const distPath = path.resolve('dist');
        const exactFile = path.join(distPath, url);
        const htmlFile = path.join(distPath, `${url}.html`);
        const indexFile = path.join(distPath, url, 'index.html');

        if (!fs.existsSync(exactFile) && !fs.existsSync(htmlFile) && !fs.existsSync(indexFile)) {
          const fallback200 = path.join(distPath, '200.html');
          if (fs.existsSync(fallback200)) {
            req.url = '/200.html' + (req.url.includes('?') ? '?' + req.url.split('?')[1] : '');
          }
        }
        next();
      });
    }
  };
}

export default defineConfig({
  plugins: [kepceIconChecker(), spaPreviewFallback(), sveltekit()],

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
