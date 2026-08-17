import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter({
			pages: 'dist',
			assets: 'dist',
			// `index.html` fallback olarak ayarlanırsa ana sayfa prerender
			// çıktısının üzerine yazılıyor ("Overwriting dist/index.html with
			// fallback page" uyarısı). `200.html` hem Cloudflare Pages hem
			// Netlify hem de çoğu static host tarafından SPA fallback
			// olarak tanınır ve ana sayfayı ezinmez.
			fallback: '200.html',
			precompress: true,
			strict: false
		}),
		prerender: {
			handleHttpError: 'warn',
			handleMissingId: 'warn',
			handleUnseenRoutes: 'ignore'
		},
		appDir: 'internal',
		paths: {
			relative: false
		},
		alias: {
			'@': './src'
		}
		// CSP not configured here on purpose:
		// - `ssr = false` + `prerender = true` (SPA fallback) means SvelteKit's
		//   `csp.mode: 'hash'` cannot accurately enumerate inline scripts at build time,
		//   causing false-positive blocks of Vite HMR / SvelteKit hydration scripts in dev.
		// - nginx.conf already provides a strict CSP for the production build.
		// - Dev server is intentionally permissive (Vite injects inline scripts dynamically).
	},
	compilerOptions: {
		css: 'external'
	}
};

export default config;
