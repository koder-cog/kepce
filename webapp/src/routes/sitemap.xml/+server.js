import { CITY_MAP } from '@/utils/turkish.js';

export const prerender = true;

/** @type {import('./$types').RequestHandler} */
export function GET() {
  const BASE_URL = 'https://kepce.org';
  const TODAY = new Date().toISOString().split('T')[0];

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

  const cityPages = Object.keys(CITY_MAP).map((slug) => `/${slug}`);

  const allPages = [...staticPages, ...cityPages];

  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${allPages
  .map(
    (path) => `  <url>
    <loc>${BASE_URL}${path}</loc>
    <lastmod>${TODAY}</lastmod>
  </url>`
  )
  .join('\n')}
</urlset>`;

  return new Response(xml, {
    headers: {
      'Content-Type': 'application/xml; charset=utf-8',
      'Cache-Control': 'max-age=0, s-maxage=3600'
    }
  });
}
