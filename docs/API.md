# Kepçe REST API Referansı (API Specification)

Kepçe REST API, `https://kepce.org/api/v1` taban URL'si üzerinden hizmet verir.

---

## 🍽️ Menü Endpoint'leri

### 1. Günün Menüsü
```http
GET /api/v1/menus/today/:city_slug
```
* **Açıklama:** Belirtilen şehir için bugünün onaylanmış menülerini getirir.
* **Örnek:** `/api/v1/menus/today/istanbul`

### 2. Tarih Aralığı veya Şehir Menüleri
```http
GET /api/v1/menus?city=:city_slug&start_date=YYYY-MM-DD&end_date=YYYY-MM-DD
```
* **Parametreler:**
  * `city` (string, zorunlu): Şehir kısa adı (örn: `istanbul`, `ankara`)
  * `start_date` (date, opsiyonel): Başlangıç tarihi
  * `end_date` (date, opsiyonel): Bitiş tarihi
  * `date` (date, opsiyonel): Tekil tarih sorgusu

### 3. Menü Detayı
```http
GET /api/v1/menus/:menu_id
```

### 4. Menüye Oy Verme
```http
POST /api/v1/menus/:menu_id/vote
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "sentiment": "positive" | "negative"
}
```

---

## 🏙️ Şehir Endpoint'leri

### Tüm Şehirleri Listele
```http
GET /api/v1/public/cities
```
* **Dönen Alanlar:** `id`, `name`, `slug`, `plate_code`, `dormitory_count`, `active_menu_count`.

---

## 💬 Yorumlar

### Menü Yorumlarını Getir
```http
GET /api/v1/comments?menu_id=:menu_id&page=1&limit=20
```

### Yorum Ekle
```http
POST /api/v1/comments
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "menu_id": 1050,
  "content": "Bugünkü yemek gayet lezzetliydi."
}
```

---

## 📊 İstatistikler & Şeffaflık

### Genel Platform Metrikleri
```http
GET /api/v1/statistics/overview
```
* Toplam menü sayısı, kapsanan şehirler, oylama dağılımları ve veri madenciliği durumları.
