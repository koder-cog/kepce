import { error } from '@sveltejs/kit';
import { apiGet } from '@/lib/server/api.js';

/**
 * Menü detay sayfası: SSR'da menüyü API'den (container içi) çekip HTML'e
 * gömer — yemek isimleri, title ve Menu JSON-LD sunucuda üretilir.
 * Yorumlar client-side yüklenmeye devam eder (kullanıcı üretimi içerik).
 */
export async function load({ params, setHeaders }) {
	const menuId = params.id;

	if (!/^\d+$/.test(menuId)) {
		error(404, 'Menü bulunamadı.');
	}

	setHeaders({
		'cache-control': 'public, s-maxage=120, stale-while-revalidate=600'
	});

	const raw = await apiGet(`/api/v1/menus/${menuId}`);

	if (!raw || !raw.id) {
		error(404, 'Menü bulunamadı veya silinmiş.');
	}

	// API `serve_date` + `city_name` döndürür; sayfa bileşenleri `date`
	// alanını beklediği için normalize edilir (client normalizer ile aynı sözleşme).
	const menu = {
		...raw,
		date: raw.date ?? raw.serve_date,
		city: raw.city ?? { name: raw.city_name }
	};

	return { menu };
}
