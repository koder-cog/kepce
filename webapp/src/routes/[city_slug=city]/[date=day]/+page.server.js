import { redirect, error } from '@sveltejs/kit';
import { CITY_MAP } from '@/utils/turkish.js';

/**
 * Gün sayfası yönlendirmesi:
 * /{sehir}/{tarih} istekleri 301 kalıcı yönlendirmeyle /{sehir}?gun={tarih}
 * adresine aktarılır. Bu sayede hem eski linkler ve botlar korunur hem de
 * kullanıcı doğrudan o gün seçili olarak şehir ana sayfasına düşer.
 */
export function load({ params }) {
	const { city_slug, date } = params;

	if (!CITY_MAP[city_slug]) {
		error(404, 'Bu şehir için bir sayfa bulunamadı.');
	}

	redirect(301, `/${city_slug}?gun=${date}`);
}
