import { ACTIVE_CITIES, CITY_MAP } from '@/utils/turkish.js';
import { apiGet, istanbulToday } from '@/lib/server/api.js';

const MONTHS_TR = [
	'Ocak', 'Şubat', 'Mart', 'Nisan', 'Mayıs', 'Haziran',
	'Temmuz', 'Ağustos', 'Eylül', 'Ekim', 'Kasım', 'Aralık'
];

function formatMonthYear(dateStr) {
	if (!dateStr || dateStr.length < 7) return null;
	const year = dateStr.slice(0, 4);
	const monthIdx = parseInt(dateStr.slice(5, 7), 10) - 1;
	const monthName = MONTHS_TR[monthIdx];
	if (!monthName) return null;
	return `${monthName} ${year}`;
}

/**
 * Şehirler dizini: Menüsü bulunan tüm şehirleri güncellik durumuna göre kategorize eder.
 * Googlebot için ana sayfadan şehir iniş sayfalarına doğrudan HTML bağlantı köprüsü sağlar.
 */
export async function load({ setHeaders }) {
	setHeaders({
		'cache-control': 'public, s-maxage=3600, stale-while-revalidate=86400'
	});

	const cityLastmods = await apiGet('/api/v1/public/menus/latest-by-city', {
		fallback: {}
	});

	const todayStr = istanbulToday();
	const currentYearMonth = todayStr.slice(0, 7);
	const todayDate = new Date(todayStr + 'T00:00:00Z');

	const allSlugs = Array.from(new Set([
		...Object.keys(cityLastmods || {}).map((s) => (s === 'canakri' ? 'cankiri' : s)),
		...ACTIVE_CITIES
	]));

	const collator = new Intl.Collator('tr-TR', { sensitivity: 'base' });

	const currentMonthCities = [];
	const recentCities = [];
	const outdatedCities = [];

	for (const slug of allSlugs) {
		const name = CITY_MAP[slug] || slug;
		const lastmod = cityLastmods?.[slug] || (slug === 'cankiri' ? cityLastmods?.['canakri'] : null);
		const lastmodLabel = formatMonthYear(lastmod);

		let diffDays = Infinity;
		if (lastmod) {
			const lastDate = new Date(lastmod + 'T00:00:00Z');
			if (!isNaN(lastDate.getTime())) {
				diffDays = Math.floor((todayDate - lastDate) / (1000 * 60 * 60 * 24));
			}
		}

		const cityObj = {
			slug,
			name,
			lastmod,
			lastmodLabel
		};

		if (lastmod && lastmod.slice(0, 7) === currentYearMonth) {
			currentMonthCities.push(cityObj);
		} else if (diffDays <= 90) {
			recentCities.push(cityObj);
		} else {
			outdatedCities.push(cityObj);
		}
	}

	currentMonthCities.sort((a, b) => collator.compare(a.name, b.name));
	recentCities.sort((a, b) => collator.compare(a.name, b.name));
	outdatedCities.sort((a, b) => collator.compare(a.name, b.name));

	const groups = [
		{
			id: 'current',
			title: 'Bu Ay Menüsü Olanlar',
			description: 'İçinde bulunduğumuz ay için Kepçe\'de menü verisi bulunan şehirler.',
			cities: currentMonthCities
		},
		{
			id: 'recent',
			title: 'Son 3 Ay İçinde Menüsü Olanlar',
			description: 'Son 90 gün içinde Kepçe\'de menüsü paylaşılmış şehirler.',
			cities: recentCities
		},
		{
			id: 'outdated',
			title: 'Uzun Süredir Güncellenmeyenler',
			description: '3 aydan uzun süredir Kepçe\'ye yeni menü verisi girilmemiş şehirler.',
			cities: outdatedCities
		}
	].filter((group) => group.cities.length > 0);

	const allCities = [...currentMonthCities, ...recentCities, ...outdatedCities]
		.sort((a, b) => collator.compare(a.name, b.name));

	return {
		groups,
		cities: allCities,
		totalCount: allSlugs.length
	};
}
