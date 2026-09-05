import { ACTIVE_CITIES, CITY_MAP } from '@/utils/turkish.js';
import { apiGet } from '@/lib/server/api.js';

/**
 * Şehirler dizini: Menüsü bulunan tüm aktif şehirleri listeler.
 * Googlebot için ana sayfadan şehir iniş sayfalarına doğrudan HTML bağlantı köprüsü sağlar.
 */
export async function load({ setHeaders }) {
	setHeaders({
		'cache-control': 'public, s-maxage=3600, stale-while-revalidate=86400'
	});

	const cityLastmods = await apiGet('/api/v1/public/menus/latest-by-city', {
		fallback: {}
	});

	const collator = new Intl.Collator('tr-TR', { sensitivity: 'base' });
	const cities = ACTIVE_CITIES.map((slug) => ({
		slug,
		name: CITY_MAP[slug] || slug,
		lastmod: cityLastmods?.[slug] || null
	})).sort((a, b) => collator.compare(a.name, b.name));

	return {
		cities
	};
}
