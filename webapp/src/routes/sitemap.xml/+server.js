import { ACTIVE_CITIES } from '@/utils/turkish.js';
import { withEtag } from '@/lib/server/etag.js';
import { apiGet, istanbulToday } from '@/lib/server/api.js';

/**
 * Sitemap - Doğrulanmış statik sayfalar ve aktif menüsü olan şehirler.
 * Thin content ve spam riski önlenerek doğrudan odaklı URL listesi sunulur.
 * Şehir sayfaları için lastmod: o şehrin onaylı en son menü tarihidir (bugünü geçmeyecek şekilde).
 */
const BASE_URL = 'https://kepce.org';

const staticPagesWithLastmod = [
	{ path: '', dynamicRoot: true },
	{ path: '/sehirler', lastmod: '2026-09-01' },
	{ path: '/kyk-yemek-saatleri', lastmod: '2026-08-20' },
	{ path: '/kyk-beslenme-yardimi', lastmod: '2026-08-20' },
	{ path: '/arsiv', lastmod: '2026-08-15' },
	{ path: '/istatistikler/yemekler', lastmod: '2026-09-01' },
	{ path: '/istatistikler/yorumlar', lastmod: '2026-09-01' },
	{ path: '/istatistikler/insaniyet', lastmod: '2026-09-01' },
	{ path: '/istatistikler/denetim', lastmod: '2026-09-01' },
	{ path: '/durum', lastmod: '2026-09-01' },
	{ path: '/menu-gonder', lastmod: '2026-08-10' },
	{ path: '/hakkinda', lastmod: '2026-08-10' },
	{ path: '/sss', lastmod: '2026-08-20' },
	{ path: '/iletisim', lastmod: '2026-08-10' },
	{ path: '/rss', lastmod: '2026-08-30' },
	{ path: '/kullanim-kosullari', lastmod: '2026-08-01' },
	{ path: '/gizlilik-politikasi', lastmod: '2026-08-01' }
];

/** @type {import('./$types').RequestHandler} */
export async function GET({ request }) {
	const today = istanbulToday();

	// Şehirlerin en son onaylı menü tarihlerini (bugünü geçmeyecek şekilde) API'den al
	const cityLastmods = await apiGet('/api/v1/public/menus/latest-by-city', {
		fallback: {}
	});

	const rootLastmod = cityLastmods?.['istanbul'] || today;

	const staticEntries = staticPagesWithLastmod.map(({ path, lastmod, dynamicRoot }) => ({
		loc: `${BASE_URL}${path}`,
		lastmod: dynamicRoot ? rootLastmod : (lastmod || '2026-09-01')
	}));

	const cityEntries = ACTIVE_CITIES.map((slug) => ({
		loc: `${BASE_URL}/${slug}`,
		lastmod: cityLastmods?.[slug] || today
	}));

	const allEntries = [...staticEntries, ...cityEntries];

	const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${allEntries
	.map(
		(entry) => `  <url>
    <loc>${entry.loc}</loc>
    <lastmod>${entry.lastmod}</lastmod>
  </url>`
	)
	.join('\n')}
</urlset>`;

	return withEtag(request, xml, {
		'Content-Type': 'application/xml; charset=utf-8',
		'Cache-Control': 'public, max-age=0, s-maxage=3600'
	});
}
