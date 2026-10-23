import { apiGet, normalizeMenuList, istanbulToday } from '$lib/server/api.js';

/**
 * Dinamik llms-full.txt uç noktası.
 * Platform rehberleri, saatler, beslenme yardımı, SSS, API dokümantasyonu
 * ve bugünün güncel yemek menüsünü tek bir Markdown akışında birleştirir.
 */
export async function GET() {
  const date = istanbulToday();
  const payload = await apiGet(`/api/v1/menus?city=istanbul&date=${date}`, {
    fallback: [],
    timeout: 3000
  });
  const menus = normalizeMenuList(payload);

  const sections = [];

  // 1. Giriş ve Başlık
  sections.push(`# Kepçe - Tam Metin Dokümantasyonu ve Bilgi Tabanı (llms-full.txt)

> Kepçe, Türkiye genelindeki KYK yurt yemek menülerini, kalori ve besin değerlerini, öğrenci değerlendirmelerini ve açık API hizmetini sunan bağımsız, kâr amacı gütmeyen açık kaynaklı bir platformdur.

- Web: https://kepce.org/
- Açık Kaynak: https://github.com/koder-cog/kepce (AGPLv3)
- Kısa İndeks: https://kepce.org/llms.txt
- Son Güncelleme: ${date}
`);

  // 2. Bugünün Güncel Menüsü (Örnek Vitrin)
  sections.push(`## Bugünün güncel menüsü (${date})

Türkiye genelindeki tüm KYK yurtlarında aynı yemek çıkmaz. Gençlik ve Spor İl Müdürlükleri ihaleleri il bazında açar. Örneğin Ankara'da kavurma varken Kars'ta mercimek çorbası, İzmir'de pırasa çıkıyor olabilir ancak aynı il sınırları içindeki tüm devlet yurtlarında menü kural olarak ortaktır.

Aşağıdaki liste yalnızca İstanbul için günün örnek vitrin menüsüdür. Kars, Erzurum, Konya, İzmir veya diğer illerin güncel menüsü için API uç noktasını (?city=sehir_slug) sorgulayabilirsiniz.
`);

  if (Array.isArray(menus) && menus.length > 0) {
    for (const menu of menus) {
      const typeLabel = menu.menu_type === 'breakfast' ? 'Kahvaltı' : 'Akşam Yemeği';
      sections.push(`### ${typeLabel} (İstanbul)`);
      if (Array.isArray(menu.items) && menu.items.length > 0) {
        for (const item of menu.items) {
          const calStr = item.calories ? ` - ${item.calories} kcal` : '';
          sections.push(`- ${item.name}${calStr}`);
        }
      } else {
        sections.push('- Menü öğesi bilgisi bulunamadı.');
      }
      sections.push('');
    }
  } else {
    sections.push('Bugün için kayıtlı menü bulunamadı veya sisteme henüz girilmedi.\n');
  }

  // 3. KYK Yemek Saatleri
  sections.push(`## KYK yemek saatleri rehberi

Kahvaltı hafta içi 06:00 - 12:00, hafta sonu 06:00 - 12:30 arasında verilir. Erken derse veya staja gidenler için 06:00 - 10:30 arasında paket kahvaltı (Al Götür) seçeneği bulunur. Akşam yemeği 16:00 - 22:00 saatleri arasındadır; sınav dönemlerinde bazı yurtlarda 23:00'e kadar uzatılabilir. Ramazan boyunca kahvaltı saati sahur düzenine (00:00 - 06:00) kaydırılır.
`);

  // 4. KYK Beslenme Yardımı ve Tabldot Matematiği
  sections.push(`## KYK beslenme yardımı ve tabldot kuralları

KYK yurtlarında kalan öğrencilere günlük 300 TL beslenme yardımı verilir. Bu tutarın 110 TL'si kahvaltıya, 190 TL'si akşam yemeğine ayrılmıştır.

Standart tabldot menü (kahvaltıda yumurta, peynir, zeytin, ekmek ve çay; akşam yemeğinde çorba, ana yemek, pilav veya makarna, tatlı veya meyve) bu limitlere denk gelir ve öğrenci hiçbir ücret ödemez. Dolaptan ekstra içecek veya alakart yemek alındığında 110 TL veya 190 TL sınırını aşan fark kasada karttan çekilir. Sabah kullanılmayan yardım akşama devretmez, ertesi güne aktarılamaz veya nakde çevrilemez; o öğünde kullanılmayan hak yanar. Temmuz ve Ağustos aylarında nöbetçi yurtlar haricinde yardım geçerli değildir.
`);

  // 5. Sıkça Sorulan Sorular (SSS)
  sections.push(`## Sıkça sorulan sorular

### Kepçe resmi bir devlet kurumu mu?
Hayır. Bakanlıkla kurumsal bağı yoktur. Yurtlarda kalan öğrenciler ve açık kaynak geliştiricileri tarafından imece usulü yürütülen bağımsız bir projedir.

### Veriler nereden toplanıyor?
Öğrenci WhatsApp gruplarına atılmış yemekhane panosu fotoğraflarından, il müdürlüğü duyurularından ve öğrencilerin sisteme yüklediği listelerden derlenir. Botlar bu listeleri ayıklayıp veritabanına işler.

### Menü yanlış çıkarsa ne yapmalı?
Mutfaktaki aşçının o günkü tercihi, sevkiyat gecikmesi veya malzeme eksiği gibi kelebek etkileri menüyü anlık değiştirebilir. Yanlışlık varsa sitedeki hata bildir butonundan düzeltebilirsiniz.

### Kalori değerleri kesin mi?
Standart kurumsal tabldot gramajları üzerinden hesaplanan yaklaşık değerlerdir. Aşçının kepçeyi daldırırken gösterdiği cömertliğe göre tabaktaki kalori miktarı sapma gösterebilir.
`);

  // 6. Açık API Uç Noktaları Dokümantasyonu
  sections.push(`## Açık API dokümantasyonu

Tüm uç noktalar JSON formatında yanıt döner ve kimlik doğrulaması gerektirmez.

### Belirli tarih ve şehir menüsü
GET https://kepce.org/api/v1/menus?city={sehir}&date={YYYY-MM-DD}

Parametreler:
- city (zorunlu): İl adı veya plaka slug'ı (örnek: istanbul, ankara, kars)
- date (isteğe bağlı): YYYY-MM-DD formatında gün (varsayılan: bugün)

Örnek çağrı:
curl "https://kepce.org/api/v1/menus?city=istanbul&date=${date}"

### Bugünün menüsü (kısa yol)
GET https://kepce.org/api/v1/public/menus/today/{city}

### Aktif şehirler listesi
GET https://kepce.org/api/v1/public/cities
`);

  const fullContent = sections.join('\n');

  return new Response(fullContent, {
    status: 200,
    headers: {
      'Content-Type': 'text/plain; charset=utf-8',
      'Cache-Control': 'public, s-maxage=1800, stale-while-revalidate=3600'
    }
  });
}
