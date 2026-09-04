import { error } from '@sveltejs/kit';
import { apiGet, normalizeMenuList, istanbulToday } from '@/lib/server/api.js';
import { CITY_MAP } from '@/utils/turkish.js';

/**
 * Şehir sayfası: SSR'da o şehrin bugünkü menülerini API'den (container içi)
 * çeker ve HTML'e gömer. Menü yoksa veya yaz tatilindeyse noindex basar,
 * son çıkan menüyü ve arşiv bağlantılarını sunar.
 */
export async function load({ params, url, setHeaders }) {
	const citySlug = params.city_slug;

	if (!CITY_MAP[citySlug]) {
		error(404, 'Bu şehir için bir sayfa bulunamadı.');
	}

	const gunParam = url.searchParams.get('gun') || url.searchParams.get('tarih') || url.searchParams.get('date');
	const date = gunParam && /^\d{4}-\d{2}-\d{2}$/.test(gunParam) ? gunParam : istanbulToday();
	const isSummer = ['07', '08'].includes(date.slice(5, 7));

	setHeaders({
		'cache-control': 'public, s-maxage=120, stale-while-revalidate=600'
	});

	const payload = await apiGet(
		`/api/v1/menus?city=${encodeURIComponent(citySlug)}&date=${date}`
	);
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
				.filter((d) => d.city_slug === citySlug)
				.map((d) => d.date);

			if (cityDays.length > 0) {
				const latestDate = cityDays[cityDays.length - 1];
				const latestPayload = await apiGet(
					`/api/v1/menus?city=${encodeURIComponent(citySlug)}&date=${latestDate}`,
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
		citySlug,
		date,
		menus,
		isSummer,
		noindex: false,
		lastMenuDay
	};
}
