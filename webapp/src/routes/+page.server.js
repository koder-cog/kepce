import { apiGet, normalizeMenuList, istanbulToday } from '@/lib/server/api.js';
import { ACTIVE_CITIES } from '@/utils/turkish.js';

/**
 * Ana sayfa: SSR'da bugünün menülerini API'den (container içi) çekip
 * HTML'e gömer. Veri sözleşmesi aynı kalır - +page.svelte
 * `prerenderedMenus/prerenderedCity/prerenderedDate` alanlarını tüketir.
 */
export async function load({ url, setHeaders }) {
	const cityParam = url.searchParams.get('sehir') || url.searchParams.get('city');
	const city =
		cityParam && ACTIVE_CITIES.includes(cityParam) ? cityParam : 'istanbul';
	const date = istanbulToday();

	// SSR HTML'i kısa süre edge-cache'lenebilir; menü tazeliği client
	// hydration'daki timelineState.init() ile de garanti altında.
	setHeaders({
		'cache-control': 'public, s-maxage=120, stale-while-revalidate=600'
	});

	const payload = await apiGet(`/api/v1/menus?city=${encodeURIComponent(city)}&date=${date}`);

	return {
		prerenderedMenus: normalizeMenuList(payload),
		prerenderedCity: city,
		prerenderedDate: date
	};
}
