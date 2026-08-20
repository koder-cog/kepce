/**
 * Yasal metin sürüm kaydı - Görev #53
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
      current: true,
      note: 'İlk yayımlanan sürüm.',
    },
  ],
  'kullanim-kosullari': [
    {
      slug: '20260805',
      version: '2026.08.05',
      current: true,
      note: 'İlk yayımlanan sürüm.',
    },
  ],
};
