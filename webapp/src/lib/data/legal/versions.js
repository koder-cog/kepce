/**
 * Yasal metin sürüm kaydı
 *
 * Koşullar ve Gizlilik metinlerinde değişiklikler aksi belirtilmedikçe
 * yayımlandığı tarihte yürürlüğe girer:
 *   1. Mevcut girdiden `current: true` bayrağını kaldır.
 *   2. Yeni tarihli girdiyi `current: true` ile listenin SONUNA ekle.
 *   3. Eski sürümün tam metni lib/data/legal/ altında tarihli bir modülde
 *      saklanır (tam metin arşivi buradan servis edilir).
 */
export const LEGAL_VERSIONS = {
  'gizlilik-politikasi': [
    {
      slug: '20260805',
      version: '2026.08.05',
      note: 'İlk yayımlanan sürüm.'
    },
    {
      slug: '20260902',
      version: '2026.09.02',
      note: 'Yurt dışına veri aktarımı ve içerik kaldırma süreçlerine ilişkin yasal açıklamalar güncellendi.'
    },
    {
      slug: '20260913',
      version: '2026.09.13',
      note: 'Otomatik karar alma sistemlerinin kullanılmadığı açıklandı. Hizmetin sunulması için zorunlu veriler ve içerik şikayet süreçleri netleştirildi.'
    },
    {
      slug: '20260920',
      version: '2026.09.20',
      note: 'Sunucuların Fransa/Marsilya veri merkezinde barındırıldığı ve yurt dışı veri güvenliği standartları açıklandı.'
    },
    {
      slug: '20260927',
      version: '2026.09.27',
      note: 'Kullanıcıların yüklediği menü verilerinin kapsamı, silinen içeriklerin saklanma koşulları ve şikayetlerdeki gizlilik ilkeleri detaylandırıldı.'
    },
    {
      slug: '20261004',
      version: '2026.10.04',
      current: true,
      note: 'Menüleri inceleyen anonim ziyaretçilerin hiçbir verisinin kaydedilmediği vurgulandı. Yasal trafik kayıtlarının yalnızca içerik yükleyen veya yorum yapan kullanıcılar için tutulduğu belirtildi.'
    }
  ],
  'kullanim-kosullari': [
    {
      slug: '20260805',
      version: '2026.08.05',
      note: 'İlk yayımlanan sürüm.'
    },
    {
      slug: '20260902',
      version: '2026.09.02',
      note: 'Mevzuata veya telife aykırı içeriklere yönelik uyar-kaldır (içerik şikayet) süreci güncellendi.'
    },
    {
      slug: '20260913',
      version: '2026.09.13',
      note: "Kepçe'nin yer sağlayıcı rolü açıklandı. Menü verilerinin bilgilendirme amaçlı niteliği ve içerik şikayet şartları detaylandırıldı."
    },
    {
      slug: '20260920',
      version: '2026.09.20',
      note: 'Veri tabanının kullanım sınırları, alerjen ve diyet bilgilerindeki kullanıcı sorumluluğu ile altyapı konumu açıklandı.'
    },
    {
      slug: '20260927',
      version: '2026.09.27',
      note: 'Kullanıcı menü yüklemelerinin kullanım hakları, otomatik veri çekme sınırları ve turnike/fiş fiyatlarının önceliği netleştirildi.'
    },
    {
      slug: '20261004',
      version: '2026.10.04',
      current: true,
      note: 'Yalnızca menü inceleyen ziyaretçilerden hiçbir kayıt tutulmadığı, yasal kayıt tutma yükümlülüğünün sadece yorum ve fotoğraf yükleyen kullanıcıları kapsadığı belirtildi.'
    }
  ]
};
