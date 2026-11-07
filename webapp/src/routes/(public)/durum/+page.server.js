/**
 * Sistem durum sayfası: SSR önbellek başlığı ekler.
 * Uptime botlarının sık aralıklı kontrollerinde Node.js SSR yükünü hafifletir.
 */
export async function load({ setHeaders }) {
  setHeaders({
    'cache-control': 'public, max-age=30, s-maxage=60, stale-while-revalidate=120'
  });
  return {};
}
