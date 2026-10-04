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
      note: '5651 m. 9 iptali düzeltmesi. KVKK m. 9 açıklaması belirginleştirildi.'
    },
    {
      slug: '20260913',
      version: '2026.09.13',
      note: 'Yapay zekâ denetimi ibaresi kaldırıldı, 5651 m. 5 salt yer sağlayıcı ve uyar-kaldır mekanizması netleştirildi, KVKK m. 5/2-c sözleşme sebebi ve m. 11/g otomatik karar alma yokluğu beyanı eklendi.'
    },
    {
      slug: '20260920',
      version: '2026.09.20',
      note: '7499 sayılı Kanun ile güncellenen KVKK m. 9 (standart sözleşme m. 9/4-c ve sözleşmenin ifası m. 9/6-b) dayanakları netleştirildi; Fransa/Marsilya bulut veri merkezi teyit edildi.'
    },
    {
      slug: '20260927',
      version: '2026.09.27',
      note: 'Doğal ve şeffaf dil revizyonu; özellik odaklı veri toplama açıklaması, kullanıcı menü yüklemeleri lisansı, 5651 silinen içerik saklama dengesi ve uyar-kaldır şikayet gizliliği eklendi.'
    },
    {
      slug: '20261004',
      version: '2026.10.04',
      current: true,
      note: '5651 ve KVKK çelişkisi giderildi; kamu menülerini inceleyen anonim ziyaretçiler için anlık bellek işlemesi ve sıfır log beyanı netleştirildi, 5651 yasal trafik kayıtları içerik üreten üyelere hasredildi.'
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
      note: '5651 m. 9 iptali düzeltmesi. Uyar-Kaldır dayanağı m. 5/2 olarak güncellendi.'
    },
    {
      slug: '20260913',
      version: '2026.09.13',
      note: 'Yapay zekâ filtreleme ifadesi kaldırıldı, menü verilerinin hukuki niteliği eklendi, 5651 m. 5/1 pasif yer sağlayıcı beyanı ve uyar-kaldır şartları berraklaştırıldı, FSEK Ek m. 8 veri tabanı koruması ve TBK m. 115 dengesi işlendi.'
    },
    {
      slug: '20260920',
      version: '2026.09.20',
      note: 'FSEK Ek m. 8 veri tabanı yapımcısı hakkı ve TTK m. 55 haksız rekabet sınırları pekiştirildi; TBK m. 115 alerjen ve diyet sorumluluk dengesi ile Fransa/Marsilya sunucu konumu teyit edildi.'
    },
    {
      slug: '20260927',
      version: '2026.09.27',
      note: 'Doğal dil revizyonu; kullanıcı menü yüklemeleri lisansı, FSEK Ek m. 8 alıntılama ve scraping sınırları, turnike/fiş fiyat önceliği ve uyar-kaldır şikayet gizliliği eklendi.'
    },
    {
      slug: '20261004',
      version: '2026.10.04',
      current: true,
      note: 'İçerik sağlayıcı ve yer sağlayıcı ayrımı netleştirildi; kamu menü okumalarında sıfır log, kullanıcı yorum/fotoğraf yüklemelerinde 5651 m. 5/3 yer sağlayıcı trafik kaydı esası bağlandı.'
    }
  ]
};
