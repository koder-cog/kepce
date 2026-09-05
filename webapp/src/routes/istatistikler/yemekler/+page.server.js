import { apiGet } from '@/lib/server/api.js';

/**
 * Yemekhane istatistikleri SSR: İlk yüklemede en çok beğenilen yemekleri
 * API'den sunucuda çeker ve HTML çıktısına gömer. Botlar ve ilk ziyaretçiler
 * loader yerine zengin yemek verilerini anında görür.
 */
export async function load({ setHeaders }) {
	setHeaders({
		'cache-control': 'public, s-maxage=300, stale-while-revalidate=1200'
	});

	const initialTopDishes = await apiGet('/api/v1/statistics/top-dishes?limit=10', {
		fallback: [],
		timeout: 2000
	});

	return {
		initialTopDishes: Array.isArray(initialTopDishes) ? initialTopDishes : []
	};
}
