import { error } from '@sveltejs/kit';
import { apiGet } from '@/lib/server/api.js';

/**
 * Aylık menü sitemap parçası: /sitemap/menus/YYYY-MM.xml
 * API'den yalnızca id + serve_date çekilir (ucuz, indeksli sorgu).
 */
const BASE_URL = 'https://kepce.org';

/** @type {import('./$types').RequestHandler} */
export async function GET({ params, setHeaders }) {
	// Route segmenti dosya adını da kapsar: "2026-08.xml" → "2026-08"
	const month = params.month.replace(/\.xml$/, '');

	if (!/^\d{4}-(0[1-9]|1[0-2])$/.test(month)) {
		error(404, 'Geçersiz sitemap parçası.');
	}

	const nowMonth = new Intl.DateTimeFormat('en-CA', {
		timeZone: 'Europe/Istanbul',
		year: 'numeric',
		month: '2-digit'
	}).format(new Date());

	// Geçmiş aylar değişmez → uzun cache; güncel ay kısa cache
	const ttl = month === nowMonth ? 3600 : 86400;
	setHeaders({
		'Content-Type': 'application/xml; charset=utf-8',
		'Cache-Control': `public, max-age=0, s-maxage=${ttl}`
	});

	const items = await apiGet(
		`/api/v1/public/menus/index?month=${encodeURIComponent(month)}`,
		{ fallback: [], timeout: 15000 }
	);

	const urls = (Array.isArray(items) ? items : [])
		.map(
			(item) => `  <url>
    <loc>${BASE_URL}/menu/${item.id}</loc>
    <lastmod>${item.serve_date}</lastmod>
  </url>`
		)
		.join('\n');

	const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${urls}
</urlset>`;

	return new Response(xml);
}
