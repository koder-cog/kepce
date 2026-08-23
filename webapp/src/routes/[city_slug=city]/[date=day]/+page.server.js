import { error } from '@sveltejs/kit';
import { apiGet, normalizeMenuList } from '@/lib/server/api.js';
import { CITY_MAP } from '@/utils/turkish.js';

/**
 * Gün sayfası: SSR'da belirli bir şehir+tarihin TÜM öğünlerini (kahvaltı+akşam)
 * API'den (container içi) çekip HTML'e gömer. Kanonik sayfa budur;
 * /menu/[id] sayfaları canonical ile buraya sinyal devreder.
 * Takvimde olmayan tarih (2026-02-30) ve menüsüz gün → GERÇEK 404.
 */
export async function load({ params, setHeaders }) {
	const { city_slug, date } = params;

	if (!CITY_MAP[city_slug]) {
		error(404, 'Bu şehir için bir sayfa bulunamadı.');
	}

	// Matcher formatı doğruladı; şimdi takvim geçerliliğini kontrol et.
	const parsed = new Date(`${date}T00:00:00Z`);
	if (
		Number.isNaN(parsed.getTime()) ||
		parsed.toISOString().slice(0, 10) !== date
	) {
		error(404, 'Geçersiz tarih.');
	}

	setHeaders({
		'cache-control': 'public, s-maxage=120, stale-while-revalidate=600'
	});

	const payload = await apiGet(
		`/api/v1/menus?city=${encodeURIComponent(city_slug)}&date=${date}`
	);
	const menus = normalizeMenuList(payload);

	if (!Array.isArray(menus) || menus.length === 0) {
		error(404, 'Bu gün için bir menü bulunamadı.');
	}

	// Kahvaltı önce (API zaten meal_type'a göre sıralı; yine de garantiye al).
	menus.sort((a, b) => (a.meal_type === 'breakfast' ? -1 : 1) - (b.meal_type === 'breakfast' ? -1 : 1));

	// Önceki/sonraki gün navigasyonu (iç link ağı).
	const shiftDate = (iso, days) => {
		const d = new Date(`${iso}T00:00:00Z`);
		d.setUTCDate(d.getUTCDate() + days);
		return d.toISOString().slice(0, 10);
	};

	return {
		citySlug: city_slug,
		date,
		prevDate: shiftDate(date, -1),
		nextDate: shiftDate(date, 1),
		menus
	};
}
