# API Uç Noktaları

Kepçe REST API uç noktaları `https://kepce.org/api/v1` taban adresi üzerinden sunulur.

## 1. Menü Uç Noktaları

Genel menü sorgulama, filtreleme ve arşiv uç noktaları herkese açıktır. İsteğe bağlı olarak kullanıcı oturum jetonuyla da çağrılabilir.

### Menü Listeleme ve Filtreleme
Şehir, tarih, diyet türü veya arşiv yılına göre menü listesi çeker.

```http
GET /api/v1/menus
```

#### Sorgu Parametreleri
- `city` (metin, isteğe bağlı): Şehir kısa adı (`istanbul`, `ankara` gibi).
- `date` (metin, isteğe bağlı): `today` veya `YYYY-MM-DD` biçiminde gün (`2026-09-13`).
- `dietary_type` (metin, isteğe bağlı): Diyet türü filtresi (`normal`, `celiac`).
- `year` (tam sayı, isteğe bağlı): Arşiv yılı (`2026`).
- `month` (tam sayı, isteğe bağlı): Arşiv ayı (`1` ile `12` arası).

#### Örnek İstek
```bash
curl -X GET "https://kepce.org/api/v1/menus?city=istanbul&date=today" \
  -H "Accept: application/json"
```

### Şehir İçin Bugünün Menüsü
Belirtilen şehir (slug veya plaka kodu) için bugünkü menü kayıtlarını (kahvaltı ve akşam) doğrudan döner.

```http
GET /api/v1/menus/today/:city
```

#### Örnek İstek
```bash
curl -X GET "https://kepce.org/api/v1/menus/today/istanbul" \
  -H "Accept: application/json"
```

#### Örnek Yanıt (`200 OK`)
```json
[
  {
    "id": 1420,
    "city_name": "İstanbul",
    "city_slug": "istanbul",
    "serve_date": "2026-09-13",
    "meal_type": "breakfast",
    "source_type": "table",
    "status": "approved",
    "bot_commentary": null,
    "calorie_range_min": 750,
    "calorie_range_max": 900,
    "calorie_range": "750 - 900 kcal",
    "calculated_calories": 820,
    "vote_count": 48,
    "rating_sum": 32,
    "comment_count": 5,
    "my_vote": null,
    "items": [
      {
        "order_index": 0,
        "raw_name": "Patates Kızartması",
        "is_alternative": false,
        "amount": "150 g",
        "calories": 312,
        "price": 35.0,
        "category": "Kahvaltılık Sıcak",
        "master_data": {
          "dish_id": 8101,
          "name": "Patates Kızartması",
          "is_celiac": false,
          "is_vegan": true,
          "is_vegetarian": true,
          "estimated_calories": 310,
          "total_votes": 120,
          "positive_votes": 105,
          "negative_votes": 15,
          "like_ratio": 0.875,
          "dislike_ratio": 0.125
        }
      },
      {
        "order_index": 1,
        "raw_name": "Haşlanmış Yumurta",
        "is_alternative": false,
        "amount": "1 adet L boy",
        "calories": 78,
        "price": 10.0,
        "category": "Kahvaltılık Protein",
        "master_data": null
      }
    ],
    "takeaways": [
      {
        "name": "1. Soğuk Sandviç Paketi",
        "items": [
          {
            "order_index": 0,
            "raw_name": "Kaşarlı Soğuk Sandviç",
            "is_alternative": false,
            "amount": "1 Adet",
            "calories": 320,
            "price": 40.0,
            "category": "Sandviç",
            "master_data": null
          }
        ]
      }
    ]
  }
]
```

### Tekil Menü Detayı
Tek bir menü kaydının detaylarını döner.

```http
GET /api/v1/menus/:menu_id
```

### Arşiv Yılları
Bir şehirde onaylı menü kaydı bulunan yılları listeler.

```http
GET /api/v1/menus/archive/years?city=:city_slug
```


## 2. Kullanıcı Etkileşim Uç Noktaları

### Menüye Oy Verme
Menüye olumlu veya olumsuz oy vermeyi sağlar. İstek başlığında kullanıcı oturum jetonu bulunmalıdır.

```http
POST /api/v1/menus/:menu_id/vote
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "sentiment": "positive"
}
```

- `sentiment`: `"positive"` (beğendim) veya `"negative"` (beğenmedim).
- Hız Sınırı: Kullanıcı veya IP başına dakikada en fazla 10 oy.

### Menü Yorumlarını Okuma
Menüye ait onaylanmış kullanıcı yorumlarını sayfalanmış olarak döner. Herkese açıktır.

```http
GET /api/v1/comments?menu_id=:menu_id&page=1&limit=20
```

### Yorum Ekleme
Menüye yorum gönderir. İstek başlığında onaylanmış kullanıcı oturum jetonu bulunmalıdır.

```http
POST /api/v1/comments
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "menu_id": 1420,
  "content": "Bugünkü çorba oldukça lezzetliydi."
}
```

- Hız Sınırı: Kullanıcı veya IP başına dakikada en fazla 5 yorum.


## 3. Şehir Uç Noktaları

### Aktif Şehirleri Listele
Sistemde aktif olarak taranan ve menüsü bulunan illeri döner. Herkese açıktır ve önbelleğe alınır.

```http
GET /api/v1/public/cities
```

#### Örnek Yanıt (`200 OK`)
```json
[
  {
    "id": 34,
    "name": "İstanbul",
    "slug": "istanbul",
    "has_celiac": true
  },
  {
    "id": 6,
    "name": "Ankara",
    "slug": "ankara",
    "has_celiac": false
  }
]
```

### Konum ve IP Tabanlı Şehir Tespiti
Cloudflare IP başlığından (`CF-IPCity`) ziyaretçinin bulunduğu ili tespit eder. Eşleşme yoksa varsayılan şehri döner.

```http
GET /api/v1/public/cities/detect
```


## 4. Site Haritası ve Dizin Uç Noktaları

Bu uç noktalar arama motoru indekslemesi, takvim gezinimi ve arşiv sayfaları için hafif veri sağlar.

### Menü Bulunan Aylar
Arşivde menü kaydı bulunan tüm ayları listeler (`YYYY-MM`).

```http
GET /api/v1/menus/months
```

### Aylık Menü Günleri Dizini
Belirtilen aydaki tüm onaylı günleri ve şehir adlarını yemek detaylarını dahil etmeden döner.

```http
GET /api/v1/menus/days?month=YYYY-MM
```

> [!NOTE]
> Geliştirici anahtarı (`X-API-Key`), tüm `/api/v1/menus/*` uç noktalarında istek başlığına eklenerek doğrudan kullanılabilir. Anahtar eklendiğinde istekler IP hız sınırından muaf tutulur ve günlük geliştirici kotanızdan düşülür. Geriye dönük uyumluluk adına `/api/v1/public/menus/*` yolları da aynı şekilde hizmet vermeyi sürdürür.
