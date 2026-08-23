# Kepçe REST API Referansı

Kepçe REST API, `https://kepce.org/api/v1` taban adresi üzerinden hizmet verir.

## Erişim ve Hız Sınırları

API uç noktaları herkese açıktır.

- Anonim Erişim: IP başına saniyede en fazla 10, dakikada en fazla 240 istek yapılabilir.
- Geliştirici Erişimi (`X-API-Key`): Bot veya uygulama geliştirenler `/gelistirici` sayfasından aldıkları anahtarı `X-API-Key` başlığı olarak göndererek yüksek kota ile çalışabilir ve kullanım istatistiklerini panelden takip edebilir.

## Önbellekleme ve ETag Desteği

Menü ve şehir listeleme yanıtları `Cache-Control` ve `ETag` başlıkları içerir:
- `Cache-Control: public, max-age=300, s-maxage=3600, stale-while-revalidate=86400`
- `ETag: "<sha256-hash>"`

İstemciler `If-None-Match: "<etag>"` başlığı gönderdiğinde veri değişmemişse gövdesiz `304 Not Modified` yanıtı döner.

## Hata Kodları

Standart HTTP durum kodları kullanılır:

- `400 Bad Request`: Geçersiz parametre veya istek gövdesi.
- `401 Unauthorized`: Yetkilendirme token'ı veya API anahtarı eksik / geçersiz.
- `403 Forbidden`: E-posta onayı yapılmamış hesap.
- `404 Not Found`: İstenen kayıt bulunamadı.
- `429 Too Many Requests`: Hız sınırı aşıldı. `Retry-After` başlığında belirtilen saniye kadar beklenmelidir.

## Uç Noktalar

### 1. Menü Sorgulama ve Filtreleme
```http
GET /api/v1/menus?city=:city_slug&date=today
```
- Parametreler:
  - `city` (string, opsiyonel): Şehir kısa adı (`istanbul`, `ankara` vb.)
  - `date` (string, opsiyonel): Tarih sorgusu (`today` veya `YYYY-MM-DD`, örn: `2026-05-15`)
  - `dietary_type` (string, opsiyonel): Diyet filtresi (`normal`, `colyak`)
  - `year` (int, opsiyonel): Arşiv yılı (`2026`)
  - `month` (int, opsiyonel): Arşiv ayı (1-12)

### 2. Tekil Menü Detayı
```http
GET /api/v1/menus/:menu_id
```
- Yanıttaki `city_slug` alanı, gün sayfası kanonik URL'ini
  (`/{city_slug}/{serve_date}`) üretmek için kullanılır.

### 3. Arşiv Yılları
```http
GET /api/v1/menus/archive/years?city=:city_slug
```

### 4. Şehir Listesi
```http
GET /api/v1/public/cities
```
- Aktif menüsü bulunan şehirleri (`id`, `name`, `slug`, `has_celiac`) listeler.
- Eski yol `GET /api/v1/cities` kalıcı olarak (308) bu adrese yönlendirir.

### 5. Gün Index (Sitemap Veri Kaynağı)
```http
GET /api/v1/public/menus/days?month=YYYY-MM
```
- Belirtilen aydaki onaylı menülerin tekil `{ city_slug, date }` gün listesi.
- Gün sayfası (`/{sehir}/{tarih}`) sitemap parçalarını besler; item join'i yoktur.
- Cache: güncel ay `s-maxage=3600`, geçmiş aylar `s-maxage=86400`.
- Kardeş endpoint'ler: `/api/v1/public/menus/months`, `/api/v1/public/menus/index?month=`.

### 5. Genel İstatistikler
```http
GET /api/v1/statistics/overview
```
- Toplam menü sayısı, şehir kapsamı ve genel oylama metrikleri.

## Kullanıcı İşlemleri (Kimlik Doğrulama Zorunlu)

Oy verme ve yorum yapma işlemleri için `Authorization: Bearer <JWT_TOKEN>` başlığı veya oturum çerezi gereklidir.

### Menüye Oy Verme
```http
POST /api/v1/menus/:menu_id/vote
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "sentiment": "positive"
}
```
- `sentiment`: `"positive"` (beğendim) veya `"negative"` (beğenmedim).

### Yorum Ekleme
```http
POST /api/v1/comments
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "menu_id": 1050,
  "content": "Yemek yorum metni."
}
```

### Menü Yorumlarını Okuma (Herkese Açık)
```http
GET /api/v1/comments?menu_id=:menu_id&page=1&limit=20
```
