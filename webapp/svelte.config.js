import adapter from '@sveltejs/adapter-node';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter({
			// `node build/index.js` ile çalışır; PORT ve HOST env değişkenleriyle kontrol edilir.
			// SSR: her istekte taze HTML — prerender/prefetch/rebuild altyapısı yoktur.
			precompress: true
		}),
		appDir: 'internal',
		inlineStyleThreshold: Infinity,
		paths: {
			relative: false
		},
		alias: {
			'@': './src'
		}
		// CSP not configured here on purpose:
		// - nginx.conf kaldırıldı; production CSP artık Caddyfile'da yönetiliyor.
		// - Dev server is intentionally permissive (Vite injects inline scripts dynamically).
	},
	compilerOptions: {
		css: 'external'
	}
};

export default config;
