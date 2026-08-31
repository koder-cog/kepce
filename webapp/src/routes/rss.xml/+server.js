import { apiGet, normalizeMenuList, istanbulToday } from '@/lib/server/api.js';
import { withEtag } from '@/lib/server/etag.js';
import { CITY_MAP } from '@/utils/turkish.js';

const BASE_URL = 'https://kepce.org';

function escapeXml(unsafe) {
	if (!unsafe) return '';
	return String(unsafe).replace(/[<>&'"]/g, (c) => {
		switch (c) {
			case '<':
				return '&lt;';
			case '>':
				return '&gt;';
			case '&':
				return '&amp;';
			case '\'':
				return '&apos;';
			case '"':
				return '&quot;';
			default:
				return c;
		}
	});
}

/** @type {import('./$types').RequestHandler} */
export async function GET({ request }) {
	const today = istanbulToday();

	// Bugünün onaylı menülerini çek
	const payload = await apiGet(`/api/v1/menus?date=${today}`, {
		fallback: [],
		timeout: 8000
	});

	const menus = normalizeMenuList(payload);
	const nowRfc822 = new Date().toUTCString();

	const items = (Array.isArray(menus) ? menus : []).map((m) => {
		const citySlug = m.city_slug || 'istanbul';
		const cityName = CITY_MAP[citySlug] || citySlug;
		const mealTitle = m.meal_type === 'breakfast' ? 'Kahvaltı' : 'Akşam Yemeği';
		const dishes = (m.items || m.dishes || [])
			.map((d) => (typeof d === 'string' ? d : d.raw_name ?? d.master_data?.name ?? d.name))
			.filter(Boolean);

		const title = `${cityName} - ${mealTitle} (${today})`;
		const link = `${BASE_URL}/${citySlug}/${today}`;
		const desc = dishes.length > 0 ? dishes.join(', ') : 'Menü içeriği';

		return `    <item>
      <title>${escapeXml(title)}</title>
      <link>${link}</link>
      <guid isPermaLink="true">${link}#${m.id || m.meal_type}</guid>
      <pubDate>${nowRfc822}</pubDate>
      <description>${escapeXml(desc)}</description>
    </item>`;
	});

	const xml = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>Kepçe - Günlük KYK Yemek Menüleri</title>
    <link>${BASE_URL}</link>
    <description>KYK yurtlarında çıkan günlük kahvaltı ve akşam yemeği menüleri.</description>
    <language>tr</language>
    <lastBuildDate>${nowRfc822}</lastBuildDate>
    <atom:link href="${BASE_URL}/rss.xml" rel="self" type="application/rss+xml" />
${items.join('\n')}
  </channel>
</rss>`;

	return withEtag(request, xml, {
		'Content-Type': 'application/xml; charset=utf-8',
		'Cache-Control': 'public, max-age=0, s-maxage=1800'
	});
}
