import { apiGet } from '@/lib/server/api.js';
import { withEtag } from '@/lib/server/etag.js';

/**
 * Sitemap INDEX - aylık bölünmüş parçaları dizer.
 * Parçalar: /sitemap/static.xml + /sitemap/menus/YYYY-MM.xml
 * Yapı ilk günden ölçeklenir; menü sayısı büyüdükçe yalnızca parça sayısı artar.
 */
const BASE_URL = 'https://kepce.org';

/** @type {import('./$types').RequestHandler} */
export async function GET({ request }) {

	const months = await apiGet('/api/v1/public/menus/months', {
		fallback: [],
		timeout: 10000
	});

	const parts = [
		`${BASE_URL}/sitemap/static.xml`,
		...(Array.isArray(months) ? months : []).map(
			(m) => `${BASE_URL}/sitemap/menus/${m}.xml`
		)
	];

	const xml = `<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${parts.map((loc) => `  <sitemap><loc>${loc}</loc></sitemap>`).join('\n')}
</sitemapindex>`;

	return withEtag(request, xml, {
		'Content-Type': 'application/xml; charset=utf-8',
		'Cache-Control': 'public, max-age=0, s-maxage=3600'
	});
}
