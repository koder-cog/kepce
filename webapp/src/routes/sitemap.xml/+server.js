import { ACTIVE_CITIES } from '@/utils/turkish.js';
import { withEtag } from '@/lib/server/etag.js';

/**
 * Sitemap - Doğrulanmış statik sayfalar ve aktif menüsü olan şehirler.
 * Thin content ve spam riski önlenerek doğrudan odaklı URL listesi sunulur.
 */
const BASE_URL = 'https://kepce.org';

function istanbulToday() {
	return new Intl.DateTimeFormat('en-CA', {
		timeZone: 'Europe/Istanbul',
		year: 'numeric',
		month: '2-digit',
		day: '2-digit'
	}).format(new Date());
}

const staticPagesWithLastmod = [
	{ path: '', lastmod: null },
	{ path: '/kyk-yemek-saatleri', lastmod: '2026-08-20' },
	{ path: '/kyk-beslenme-yardimi', lastmod: '2026-08-20' },
	{ path: '/arsiv', lastmod: '2026-08-15' },
	{ path: '/istatistikler', lastmod: null },
	{ path: '/istatistikler/yemekler', lastmod: null },
	{ path: '/istatistikler/yorumlar', lastmod: null },
	{ path: '/istatistikler/insaniyet', lastmod: null },
	{ path: '/istatistikler/denetim', lastmod: null },
	{ path: '/durum', lastmod: null },
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

	const staticEntries = staticPagesWithLastmod.map(({ path, lastmod }) => ({
		loc: `${BASE_URL}${path}`,
		lastmod: lastmod || today
	}));

	const cityEntries = ACTIVE_CITIES.map((slug) => ({
		loc: `${BASE_URL}/${slug}`,
		lastmod: today
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
