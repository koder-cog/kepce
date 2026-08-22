import { error } from '@sveltejs/kit';
import { apiGet, normalizeMenuList, istanbulToday } from '@/lib/server/api.js';
import { CITY_MAP } from '@/utils/turkish.js';

/**
 * Şehir sayfası: SSR'da o şehrin bugünkü menülerini API'den (container içi)
 * çeker ve HTML'e gömer. Bilinmeyen şehir slug'ı → GERÇEK 404.
 */
export async function load({ params, setHeaders }) {
	const citySlug = params.city_slug;

	if (!CITY_MAP[citySlug]) {
		error(404, 'Bu şehir için bir sayfa bulunamadı.');
	}

	const date = istanbulToday();
	setHeaders({
		'cache-control': 'public, s-maxage=120, stale-while-revalidate=600'
	});

	const payload = await apiGet(
		`/api/v1/menus?city=${encodeURIComponent(citySlug)}&date=${date}`
	);

	return {
		citySlug,
		date,
		menus: normalizeMenuList(payload)
	};
}
