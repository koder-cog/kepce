/**
 * Yasal metin sürüm kaydı
 *
 * Koşullar ve Gizlilik metinlerinde değişiklikler aksi belirtilmedikçe
 * yayımlandığı tarihte yürürlüğe girer:
 *   1. Mevcut girdiden `current: true` bayrağını kaldır.
 *   2. Yeni tarihli girdiyi `current: true` ile listenin SONUNA ekle.
 *   3. Eski sürümün tam metni lib/domain/legal/ altında tarihli bir modülde
 *      saklanır (tam metin arşivi buradan servis edilir).
 */
export const LEGAL_VERSIONS = {
  'gizlilik-politikasi': [
    {
      slug: '20260805',
      version: '2026.08.05',
      note: 'İlk yayımlanan gizlilik politikam.'
    },
    {
      slug: '20260902',
      version: '2026.09.02',
      note: 'Yurt dışına veri aktarımı ve içerik kaldırma süreçlerine ilişkin açıklamaları güncelledim.'
    },
    {
      slug: '20260913',
      version: '2026.09.13',
      note: 'Otomatik profil çıkarma yapmadığımı, hizmet için gereken asgari verileri ve şikayet süreçlerini netleştirdim.'
    },
    {
      slug: '20260920',
      version: '2026.09.20',
      note: 'Sunucularımın Fransa veri merkezindeki barınma koşullarını ve güvenlik standartlarını paylaştım.'
    },
    {
      slug: '20260927',
      version: '2026.09.27',
      note: 'Kullanıcıların yüklediği menü verilerinin kapsamını ve silinen içeriklerin saklanma şartlarını detaylandırdım.'
    },
    {
      slug: '20261004',
      version: '2026.10.04',
      note: 'Yalnızca menü okuyan anonim ziyaretçilerden hiçbir veri kaydetmediğimi, yasal kayıtların sadece yorum ve fotoğraf paylaşanları kapsadığını belirttim.'
    },
    {
      slug: '20261023',
      version: '2026.10.23',
      current: true,
      note: 'Açık e-posta ve kimlik tutmayı tamamen bıraktım, hesapları anonimleştirdim. Kanunen tutulması gereken trafik kayıtlarını ise şifreleyerek yalnızca yetkili adli merciler için erişilebilir kıldım.'
    }
  ],
  'kullanim-kosullari': [
    {
      slug: '20260805',
      version: '2026.08.05',
      note: 'İlk yayımlanan kullanım koşullarım.'
    },
    {
      slug: '20260902',
      version: '2026.09.02',
      note: 'Telif veya mevzuat ihlali bildiren içerikler için uyar-kaldır sürecini güncelledim.'
    },
    {
      slug: '20260913',
      version: '2026.09.13',
      note: "Kepçe'nin yer sağlayıcı rolünü ve menü verilerinin bilgilendirme amaçlı niteliğini açıkladım."
    },
    {
      slug: '20260920',
      version: '2026.09.20',
      note: 'Veri tabanımın kullanım sınırlarını ve yemeklerdeki alerjen bilgilerinde kullanıcı sorumluluğunu netleştirdim.'
    },
    {
      slug: '20260927',
      version: '2026.09.27',
      note: 'Kullanıcıların paylaştığı menülerin kullanım haklarını ve yemekhane turnike fiyatlarının önceliğini belirttim.'
    },
    {
      slug: '20261004',
      version: '2026.10.04',
      note: 'Yalnızca menü inceleyen ziyaretçilerden kayıt tutulmadığını, içerik sorumluluğunun yorum ve fotoğraf paylaşan kullanıcıya ait olduğunu belirttim.'
    },
    {
      slug: '20261023',
      version: '2026.10.23',
      current: true,
      note: 'Yemekhane bağımsızlığını, menülerdeki alerjen tahminlerinin bağlayıcı olmadığını ve mutfak teyidi sorumluluğunu en başa taşıdım.'
    }
  ]
};
