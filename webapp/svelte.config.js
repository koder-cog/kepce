import adapter from '@sveltejs/adapter-node';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter({
			// `node build/index.js` ile çalışır; PORT ve HOST env değişkenleriyle kontrol edilir.
			// SSR: her istekte taze HTML - prerender/prefetch/rebuild altyapısı yoktur.
			precompress: true
		}),
		appDir: 'internal',
		inlineStyleThreshold: Infinity,
		paths: {
			relative: false
		},
		alias: {
			'@': './src'
		},
		csp: {
			mode: 'auto',
			directives: {
				'default-src': ['self'],
				'base-uri': ['self'],
				'object-src': ['none'],
				'font-src': ['self', 'data:'],
				'img-src': ['self', 'data:', 'blob:', 'https:'],
				'script-src': [
					'self',
					'strict-dynamic',
					'https://static.cloudflareinsights.com',
					'https://analitik.kepce.org'
				],
				'style-src': ['self', 'unsafe-inline'],
				'connect-src': [
					'self',
					'ws:',
					'wss:',
					'https://cloudflareinsights.com',
					'https://analitik.kepce.org'
				],
				'frame-ancestors': ['none']
			}
		}
	},
	compilerOptions: {
		css: 'external'
	}
};

export default config;
