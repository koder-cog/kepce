# Hata Yönetimi ve Durum Kodları

Kepçe API uç noktaları standart HTTP durum kodları ve öngörülebilir JSON hata yanıtları üretir.

## 1. HTTP Durum Kodları

| Durum Kodu | Anlamı | Açıklama |
| :--- | :--- | :--- |
| `200 OK` | Başarılı | İstek başarıyla karşılandı ve veri döndürüldü. |
| `304 Not Modified` | Değişmedi | `If-None-Match` başlığındaki ETag değeri eşleştiğinde veri güncel kabul edilir ve yanıt gövdesi boş döner. |
| `400 Bad Request` | Geçersiz İstek | Eksik veya hatalı sorgu parametresi (örnek: geçersiz tarih biçimi). |
| `401 Unauthorized` | Yetkisiz | Geçersiz API anahtarı veya eksik oturum jetonu. |
| `403 Forbidden` | Yasaklı | Yetkisiz kaynak erişimi veya moderasyon kısıtı. |
| `404 Not Found` | Bulunamadı | İstenen şehir, gün veya menü kaydı bulunamadı. |
| `429 Too Many Requests` | Kota Aşıldı | İstek hız sınırı aşıldı. `Retry-After` başlığı kontrol edilmelidir. |
| `500 Internal Server Error` | Sunucu Hatası | Beklenmeyen bir sunucu veya veritabanı hatası oluştu. |

## 2. Hata Yanıtı Biçimi

Hata durumlarında dönen JSON nesnesi standart olarak `error` alanı içerir:

```json
{
  "error": "Şehir bulunamadı."
}
```

Doğrulama hatalarında:

```json
{
  "error": "Geçersiz tarih formatı. Beklenen biçim: YYYY-MM-DD"
}
```

## 3. Örnek Hata Yakalama (JavaScript Fetch)

```javascript
async function fetchMenu(city) {
  const url = `https://kepce.org/api/v1/public/menus/today/${city}`;
  const response = await fetch(url, {
    headers: { "Accept": "application/json" }
  });

  if (response.status === 404) {
    console.warn(`Şehir için menü kaydı bulunamadı: ${city}`);
    return null;
  }

  if (response.status === 429) {
    const retrySec = response.headers.get("Retry-After") || 5;
    console.warn(`İstek kotası aşıldı, ${retrySec} saniye sonra tekrar denenmeli.`);
    return null;
  }

  if (!response.ok) {
    const err = await response.json().catch(() => ({ error: "Bilinmeyen hata" }));
    throw new Error(`API Hatası (${response.status}): ${err.error}`);
  }

  return await response.json();
}
```
