# Kepçe REST API Referansı

> Ayrıntılı modüler kılavuzlar ve parametre listeleri için [docs/api/](api/README.md) dizinini inceleyebilirsiniz:
> - [Kimlik Doğrulama ve Kotalar](api/authentication.md)
> - [Uç Noktalar ve cURL Örnekleri](api/endpoints.md)
> - [Veri Modelleri](api/models.md)
> - [Hata Kodları](api/errors.md)

Kepçe REST API, `https://kepce.org/api/v1` taban adresi üzerinden hizmet verir.

## Erişim ve Hız Sınırları

Genel menü ve şehir okuma uç noktaları herkese açıktır.

Anonim isteklerde IP adresi başına dakikada en fazla 240, saniyede en fazla 10 istek (SpikeArrest koruması) yapılabilir.

Geliştirici anahtarı (`X-API-Key`) kullanan istemcilere ücretsiz seviyede günlük 2.500 istek, ticari seviyede ise günlük 100.000 istek kotası tanımlanır. Anlık tüketim istatistikleri `/gelistirici` sayfasından takip edilebilir.

## Önbellekleme ve ETag Desteği

Menü ve şehir listeleme yanıtları `Cache-Control` ve `ETag` başlıkları içerir:
- `Cache-Control: public, max-age=300, s-maxage=3600, stale-while-revalidate=86400`
- `ETag: "<sha256-hash>"`

İstemciler `If-None-Match: "<etag>"` başlığı gönderdiğinde veri değişmemişse gövdesiz `304 Not Modified` yanıtı döner.

## Hata Kodları

Standart HTTP durum kodları kullanılır:

- `400 Bad Request`: Geçersiz parametre veya istek gövdesi.
- `401 Unauthorized`: Yetkilendirme jetonu veya API anahtarı eksik ya da geçersiz.
- `403 Forbidden`: E-posta onayı yapılmamış hesap veya yetkisiz işlem.
- `404 Not Found`: İstenen kayıt bulunamadı.
- `429 Too Many Requests`: Hız sınırı veya günlük kota aşıldı. `Retry-After` başlığı kontrol edilmelidir.

## Uç Noktalar

### 1. Menü Sorgulama ve Filtreleme
```http
GET /api/v1/menus?city=:city_slug&date=today
```
- Parametreler:
  - `city` (metin, isteğe bağlı): Şehir kısa adı (`istanbul`, `ankara` gibi).
  - `date` (metin, isteğe bağlı): Tarih sorgusu (`today` veya `YYYY-MM-DD` biçiminde gün, örnek: `2026-05-15`).
  - `dietary_type` (metin, isteğe bağlı): Diyet filtresi (`normal`, `celiac`).
  - `year` (tam sayı, isteğe bağlı): Arşiv yılı (`2026`).
  - `month` (tam sayı, isteğe bağlı): Arşiv ayı (`1` ile `12` arası).

### 2. Şehir İçin Bugünün Menüsü
```http
GET /api/v1/menus/today/:city
```

### 3. Tekil Menü Detayı
```http
GET /api/v1/menus/:menu_id
```

### 4. Arşiv Yılları
```http
GET /api/v1/menus/archive/years?city=:city
```

### 5. Şehir Listesi
```http
GET /api/v1/public/cities
```
- Aktif menüsü bulunan şehirleri (`id`, `name`, `slug`, `has_celiac`) listeler.
- Eski yol `GET /api/v1/cities` kalıcı olarak (308) bu adrese yönlendirir.

### 6. Gün Dizini (Site Haritası Veri Kaynağı)
```http
GET /api/v1/menus/days?month=YYYY-MM
```
- Belirtilen aydaki onaylı menülerin tekil `{ city_slug, date }` gün listesi.
- Gün sayfası (`/{sehir}/{tarih}`) site haritası parçalarını besler, yemek detaylarını içermez.
- Önbellek: Güncel ay `s-maxage=3600`, geçmiş aylar `s-maxage=86400`.
- Kardeş uç noktalar: `/api/v1/menus/months`, `/api/v1/menus/index?month=`. Geriye dönük uyumluluk adına `/api/v1/public/menus/*` yolları da desteklenir.

## Kullanıcı İşlemleri (Oturum Jetonu Zorunlu)

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
