import { ACTIVE_CITIES } from '@/utils/turkish.js';

/**
 * Statik sayfalar + 81 şehir sayfası (~100 URL).
 * Şehir sayfaları her gün güncellendiği için lastmod = bugün (Istanbul).
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

const staticPages = [
	'',
	'/kyk-yemek-saatleri',
	'/kyk-beslenme-yardimi',
	'/arsiv',
	'/istatistikler',
	'/istatistikler/yemekler',
	'/istatistikler/yorumlar',
	'/istatistikler/insaniyet',
	'/istatistikler/denetim',
	'/durum',
	'/menu-gonder',
	'/hakkinda',
	'/sss',
	'/iletisim',
	'/rss',
	'/kullanim-kosullari',
	'/gizlilik-politikasi'
];

/** @type {import('./$types').RequestHandler} */
export async function GET({ setHeaders }) {
	const today = istanbulToday();
	setHeaders({
		'Content-Type': 'application/xml; charset=utf-8',
		'Cache-Control': 'public, max-age=0, s-maxage=3600'
	});

	const paths = [...staticPages, ...ACTIVE_CITIES.map((slug) => `/${slug}`)];

	const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${paths
	.map(
		(path) => `  <url>
    <loc>${BASE_URL}${path}</loc>
    <lastmod>${today}</lastmod>
  </url>`
	)
	.join('\n')}
</urlset>`;

	return new Response(xml);
}
