import Negotiator from 'negotiator';
import { apiGet, normalizeMenuList, istanbulToday } from './api.js';

export const ROOT_MEDIA_TYPES = ['text/html', 'text/markdown', 'text/plain', 'application/json'];

export const ERROR_MEDIA_TYPES = ['text/html', 'application/json'];

/**
 * RFC 9110 uyumlu Accept başlığı ayrıştırıcı.
 * İstemcinin öncelik sırasını, özgüllüğünü ve q-değerlerini hesaplar.
 *
 * @param {Request} request
 * @param {string[]} availableTypes
 * @returns {string} Tercih edilen medya türü veya ilk varsayılan
 */
export function negotiateMediaType(request, availableTypes = ROOT_MEDIA_TYPES) {
  const acceptHeader = request.headers.get('accept');
  if (!acceptHeader || !acceptHeader.trim()) {
    return availableTypes[0];
  }

  try {
    const negotiator = new Negotiator({
      headers: { accept: acceptHeader }
    });
    const preferred = negotiator.mediaType(availableTypes);
    return preferred || availableTypes[0];
  } catch {
    return availableTypes[0];
  }
}

/**
 * Kök dizin için güncel menü verisi ile Markdown içeriği üretir.
 *
 * @param {string} date
 * @param {Array} menus
 * @returns {string}
 */
export function formatRootMarkdown(date, menus) {
  const lines = [
    '# Kepçe - KYK günün yemek menüsü',
    '',
    `Tarih: ${date}`,
    'Şehir: İstanbul (Örnek vitrin - illere göre menüler değişir, diğer iller için ?city=sehir_slug kullanın)',
    ''
  ];

  if (Array.isArray(menus) && menus.length > 0) {
    for (const menu of menus) {
      const typeLabel = menu.menu_type === 'breakfast' ? 'Kahvaltı' : 'Akşam Yemeği';
      lines.push(`## ${typeLabel}`);
      if (Array.isArray(menu.items) && menu.items.length > 0) {
        for (const item of menu.items) {
          const calStr = item.calories ? ` (${item.calories} kcal)` : '';
          lines.push(`- ${item.name}${calStr}`);
        }
      } else {
        lines.push('- Menü öğesi bilgisi bulunamadı.');
      }
      lines.push('');
    }
  } else {
    lines.push('Bugün için kayıtlı menü bulunamadı veya henüz yayımlanmadı.');
    lines.push('');
  }

  lines.push('## Hızlı bağlantılar ve açık API');
  lines.push('- Şehirler dizini: https://kepce.org/sehirler');
  lines.push('- Yemek istatistikleri: https://kepce.org/istatistikler/yemekler');
  lines.push('- Yemek saatleri: https://kepce.org/kyk-yemek-saatleri');
  lines.push('- Beslenme yardımı: https://kepce.org/kyk-beslenme-yardimi');
  lines.push('- Tam bilgi dokümanı: https://kepce.org/llms-full.txt');
  lines.push(
    '- Menü API sorgusu: `GET https://kepce.org/api/v1/menus?city={sehir_slug}&date={YYYY-MM-DD}`'
  );

  return lines.join('\n');
}

/**
 * Kök dizin için JSON formatında durum ve menü özeti üretir.
 * AgentGrade kuralı: En üst düzeyde 'error' anahtarı bulunmamalıdır.
 *
 * @param {string} date
 * @param {Array} menus
 * @returns {object}
 */
export function formatRootJson(date, menus) {
  return {
    name: 'Kepçe',
    description: 'KYK yurt yemek menüleri, kalori ve besin değerleri açık veri platformu',
    date,
    city: 'istanbul',
    menus: menus || [],
    links: {
      cities: '/sehirler',
      archive: '/arsiv',
      stats: '/istatistikler/yemekler',
      meal_hours: '/kyk-yemek-saatleri',
      nutrition_aid: '/kyk-beslenme-yardimi',
      faq: '/sss',
      llms: '/llms.txt',
      llms_full: '/llms-full.txt'
    },
    endpoints: {
      menus: '/api/v1/menus?city={city}&date={YYYY-MM-DD}',
      today_menu: '/api/v1/public/menus/today/{city}',
      cities: '/api/v1/public/cities'
    }
  };
}

/**
 * Kök dizine gelen HTML dışı isteklere yanıt döner.
 * Cloudflare edge önbellek zehirlenmesini önlemek için no-store başlığı zorunludur.
 *
 * @param {string} mediaType
 * @returns {Promise<Response>}
 */
export async function handleRootNegotiatedResponse(mediaType) {
  const date = istanbulToday();
  const payload = await apiGet(`/api/v1/menus?city=istanbul&date=${date}`, {
    fallback: [],
    timeout: 3000
  });
  const menus = normalizeMenuList(payload);

  const baseHeaders = {
    'Cache-Control': 'private, no-cache, no-store, max-age=0',
    Vary: 'Accept'
  };

  if (mediaType === 'application/json') {
    return new Response(JSON.stringify(formatRootJson(date, menus), null, 2), {
      status: 200,
      headers: {
        ...baseHeaders,
        'Content-Type': 'application/json; charset=utf-8'
      }
    });
  }

  const markdownBody = formatRootMarkdown(date, menus);

  if (mediaType === 'text/plain') {
    return new Response(markdownBody, {
      status: 200,
      headers: {
        ...baseHeaders,
        'Content-Type': 'text/plain; charset=utf-8'
      }
    });
  }

  // text/markdown
  return new Response(markdownBody, {
    status: 200,
    headers: {
      ...baseHeaders,
      'Content-Type': 'text/markdown; charset=utf-8'
    }
  });
}

/**
 * Yapılandırılmış 404 JSON hata yanıtı.
 *
 * @param {string} pathname
 * @returns {Response}
 */
export function formatJson404(pathname) {
  return new Response(
    JSON.stringify({
      error: 'not_found',
      status: 404,
      message: 'Aradığınız sayfa bulunamadı',
      path: pathname
    }),
    {
      status: 404,
      headers: {
        'Content-Type': 'application/json; charset=utf-8',
        'Cache-Control': 'private, no-cache, no-store, max-age=0',
        Vary: 'Accept'
      }
    }
  );
}
