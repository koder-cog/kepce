/**
 * Rehber sayfası SEO meta verisi.
 *
 * Beslenme yardımı sayfasıyla aynı mimariyi izler: sayfa içeriği statiktir,
 * meta veri load fonksiyonundan gelir ve bileşende `data?.seoTitle || ...`
 * deseniyle tüketilir. Böylece iki kardeş rehber sayfası tek desende çalışır.
 *
 * @type {import('./$types').PageServerLoad}
 */
export function load() {
  return {
    ogImage: 'https://kepce.org/api/v1/public/og/page/rehber',
    seoTitle: 'KYK Yemek Saatleri | Kepçe',
    seoDescription:
      'KYK yurtlarında kahvaltı ve akşam yemeği saatleri, hafta sonu saatleri, Al Götür paketleri ve Ramazan ayı düzeni.'
  };
}
