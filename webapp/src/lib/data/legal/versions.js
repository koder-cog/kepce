/**
 * Yasal metin sürüm kaydı
 *
 * Koşullar/Gizlilik metinlerinde değişiklik 30 gün sonra yürürlüğe girdiği
 * için her değişiklik yeni bir girdi olarak eklenir:
 *   1. Mevcut girdiden `current: true` bayrağını kaldır.
 *   2. Yeni tarihli girdiyi `current: true` ile listenin SONUNA ekle.
 *   3. Eski sürümün tam metni lib/data/legal/ altında tarihli bir modülde
 *      saklanır (tam metin arşivi büyüdükçe buradan servis edilir).
 */
export const LEGAL_VERSIONS = {
  'gizlilik-politikasi': [
    {
      slug: '20260805',
      version: '2026.08.05',
      note: 'İlk yayımlanan sürüm.',
    },
    {
      slug: '20260902',
      version: '2026.09.02',
      note: '5651 m. 9 iptali düzeltmesi. KVKK m. 9 açıklaması belirginleştirildi.',
    },
    {
      slug: '20260913',
      version: '2026.09.13',
      current: true,
      note: 'Yapay zekâ denetimi ibaresi kaldırıldı, 5651 m. 5 salt yer sağlayıcı ve uyar-kaldır mekanizması netleştirildi, KVKK m. 5/2-c sözleşme sebebi ve m. 11/g otomatik karar alma yokluğu beyanı eklendi.',
    },
  ],
  'kullanim-kosullari': [
    {
      slug: '20260805',
      version: '2026.08.05',
      note: 'İlk yayımlanan sürüm.',
    },
    {
      slug: '20260902',
      version: '2026.09.02',
      note: '5651 m. 9 iptali düzeltmesi. Uyar-Kaldır dayanağı m. 5/2 olarak güncellendi.',
    },
    {
      slug: '20260913',
      version: '2026.09.13',
      current: true,
      note: 'Yapay zekâ filtreleme ifadesi kaldırıldı, menü verilerinin hukuki niteliği eklendi, 5651 m. 5/1 pasif yer sağlayıcı beyanı ve uyar-kaldır şartları berraklaştırıldı, FSEK Ek m. 8 veri tabanı koruması ve TBK m. 115 dengesi işlendi.',
    },
  ],
};
