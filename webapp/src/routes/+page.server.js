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
	const isSummer = ['07', '08'].includes(date.slice(5, 7));

	setHeaders({
		'cache-control': 'public, s-maxage=120, stale-while-revalidate=600'
	});

	const payload = await apiGet(`/api/v1/menus?city=${encodeURIComponent(city)}&date=${date}`);
	const menus = normalizeMenuList(payload);
	const hasMenus = Array.isArray(menus) && menus.length > 0;

	let lastMenuDay = null;
	if (!hasMenus) {
		const monthStr = date.slice(0, 7);
		try {
			const monthDays = await apiGet(
				`/api/v1/public/menus/days?month=${encodeURIComponent(monthStr)}`,
				{ fallback: [], timeout: 5000 }
			);
			const cityDays = (Array.isArray(monthDays) ? monthDays : [])
				.filter((d) => d.city_slug === city)
				.map((d) => d.date);

			if (cityDays.length > 0) {
				const latestDate = cityDays[cityDays.length - 1];
				const latestPayload = await apiGet(
					`/api/v1/menus?city=${encodeURIComponent(city)}&date=${latestDate}`,
					{ fallback: [], timeout: 5000 }
				);
				const latestMenus = normalizeMenuList(latestPayload);
				if (latestMenus && latestMenus.length > 0) {
					lastMenuDay = {
						date: latestDate,
						menus: latestMenus
					};
				}
			}
		} catch {
			// fallback
		}
	}

	return {
		prerenderedMenus: menus,
		prerenderedCity: city,
		prerenderedDate: date,
		isSummer,
		noindex: !hasMenus,
		lastMenuDay
	};
}
