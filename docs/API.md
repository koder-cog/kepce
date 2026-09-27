# Kepçe REST API Referansı

Kepçe REST API, KYK yurtlarında servis edilen günlük menüleri, tavan fiyat tarifelerini, Al Götür paket bileşenlerini ve besin değerlerini harici istemcilere ve geliştiricilere JSON formatında sunar.

Tüm istekler `https://kepce.org/api/v1` taban adresi üzerinden karşılanır.

## Modüler Dokümantasyon Haritası

API ile ilgili ayrıntılı teknik referanslar ve veri şemaları `docs/api/` dizini altında modüler olarak tutulur:

| Kılavuz | Açıklama | Bağlantı |
| :--- | :--- | :--- |
| **Kimlik Doğrulama ve Kotalar** | `X-API-Key` kotaları, tarayıcı çerez mimarisi (`credentials: "include"`) ve Bearer belirteç akışı. | [docs/api/authentication.md](api/authentication.md) |
| **Uç Noktalar** | Menü filtreleme, tekil menü detayları, gün dizinleri, oy verme ve yorum gönderme istekleri. | [docs/api/endpoints.md](api/endpoints.md) |
| **Veri Modelleri** | DTO yapıları, yemek eşleştirme modeli (`master_data`), Al Götür paketleri ve JSON alanları. | [docs/api/models.md](api/models.md) |
| **Hata Yönetimi** | HTTP durum kodları, hata yanıt formatı ve `Retry-After` hız sınırı yönetimi. | [docs/api/errors.md](api/errors.md) |

## Hızlı Başlangıç

Genel okuma uç noktaları herhangi bir kayıt veya anahtar gerektirmeden herkese açıktır.

### Aktif Şehirleri Listeleme
```bash
curl -s "https://kepce.org/api/v1/public/cities"
```

### İstanbul İçin Bugünün Menüsünü Alma
```bash
curl -s "https://kepce.org/api/v1/menus/today/istanbul"
```

## Temel Protokol Kuralları

- **Veri Biçimi:** İstek ve yanıt gövdeleri standart `application/json` biçimindedir (dosya yüklemeleri hariç). Karakter kodlaması `UTF-8`'dir.
- **Önbellek ve ETag:** Okuma uç noktaları `Cache-Control` ve `ETag` başlıkları içerir. Veri değişmediğinde sunucu `304 Not Modified` yanıtı döner.
- **Hız Sınırları:** Anonim çağrılarda IP başına dakikada 240, saniyede en fazla 10 istek yapılabilir. Yüksek hacimli entegrasyonlar için geliştirici anahtarı (`X-API-Key`) kullanılır.
- **Oturum Yönetimi:** Tarayıcı istemcileri `credentials: "include"` ile HttpOnly çerezleri (`kepce_token`) iletirken harici istemciler `Authorization: Bearer <token>` başlığını kullanabilir.

Ayrıntılı parametreler, cURL ve JavaScript örnekleri için [docs/api/](api/README.md) kılavuzlarını inceleyebilirsiniz.
