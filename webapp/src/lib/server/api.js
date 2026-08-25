/**
 * Server-only API istemcisi.
 *
 * SvelteKit `src/lib/server` altında bulunduğu için yalnızca load()/+server.js
 * tarafından import edilebilir - client bundle'a girmez.
 *
 * Container içinden Rust API'ye doğrudan bağlanır (Caddy/nginx atlanır),
 * böylece SSR render ekstra network hop ödemez.
 */

const API_INTERNAL = env('API_INTERNAL', 'http://127.0.0.1:8000');
const DEFAULT_TIMEOUT_MS = 1500;

function env(key, fallback) {
	try {
		return process.env[key] || fallback;
	} catch {
		return fallback;
	}
}

/**
 * Europe/Istanbul saat dilimine göre bugünün tarihi (YYYY-MM-DD).
 * API UTC tabanlı "today" kullanıyor olabilir; gece 00:00-03:00 TR arasındaki
 * farkı önlemek için açık tarih göndermeyi tercih ediyoruz.
 */
export function istanbulToday() {
	return new Intl.DateTimeFormat('en-CA', {
		timeZone: 'Europe/Istanbul',
		year: 'numeric',
		month: '2-digit',
		day: '2-digit'
	}).format(new Date());
}

/**
 * Internal API'den JSON çeker. Hata/timeout durumunda `fallback` döner
 * (SSR sayfası hiç veri gelmese de render olur; client hydration tazeler).
 *
 * @param {string} path  `/api/v1/...` ile başlayan yol
 * @param {{ timeout?: number, fallback?: any }} [options]
 */
export async function apiGet(path, { timeout = DEFAULT_TIMEOUT_MS, fallback = null } = {}) {
	try {
		const res = await fetch(`${API_INTERNAL}${path}`, {
			headers: { Accept: 'application/json' },
			signal: AbortSignal.timeout(timeout)
		});
		if (!res.ok) return fallback;
		return await res.json();
	} catch {
		return fallback;
	}
}

/**
 * Menü listesi yanıtlarını normalize eder:
 * API bazen düz dizi, bazen { menus } / { results } / { data } sarmalayıcısı döner.
 */
export function normalizeMenuList(payload) {
	if (Array.isArray(payload)) return payload;
	return payload?.menus ?? payload?.results ?? payload?.data ?? [];
}
