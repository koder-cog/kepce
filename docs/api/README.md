# Kepçe API Dokümantasyonu

Kepçe REST API, KYK yurtlarında servis edilen günlük ve aylık menüleri, Al Götür paketlerini, yemek bazlı porsiyon gramajı veya kalori bilgilerini ve resmi tavan fiyat karşılıklarını geliştiricilere JSON formatında sunar.

Bu doküman, Discord veya Telegram botu yazarları, üniversite kulüpleri, kampüs panoları ve mobil widget geliştiricileri için sıfırdan entegrasyon rehberidir.

## Modüller ve Kılavuzlar

- [Erişim ve Kimlik Doğrulama](authentication.md): Ücretsiz ve ticari API kotaları, IP hız sınırları ve jeton yönetimi.
- [Uç Noktalar](endpoints.md): Menü sorgulama, tekil menü detayları, oy verme, yorum yapma ve site haritası dizinleri.
- [Veri Modelleri](models.md): Menü (`MenuResponseDto`), yemek (`MenuItemDto`), Al Götür paketleri ve merkezi yemek eşleşme şemaları.
- [Hata Yönetimi](errors.md): HTTP durum kodları, hata yanıt formatı ve `Retry-After` başlığı.

## Hızlı Başlangıç

### Taban Adres
```http
https://kepce.org/api/v1
```

### Aktif Şehirleri Listele
```bash
curl -X GET "https://kepce.org/api/v1/public/cities" \
  -H "Accept: application/json"
```

### İstanbul İçin Bugünün Menüsü
```bash
curl -X GET "https://kepce.org/api/v1/menus/today/istanbul" \
  -H "Accept: application/json"
```

## Temel Özellikler

- Ham metinlerdeki OCR hataları, bitişik kelimeler ve yazım yanlışları `normalizer` ve `ContentGuard` filtreleriyle temizlenir.
- Her yemek için resmi KYK tavan fiyat listesinden porsiyon tutarı (`price`) dinamik olarak hesaplanır.
- Tekil yemekler için bülten veya katalog kalorisi (`calories`), menü için ise resmi kalori aralığı (`calorie_range`) sunulur.
- Al Götür paketleri (`takeaways`) kapalı bir metin olarak bırakılmaz, resmi paket kataloğundaki gerçek bileşenlerine ayrılarak döner.
- Standart ve glutensiz menüler bağımsız olarak etiketlenir.
- Veri değişmediğinde gereksiz ağ trafiğini engellemek için `ETag` ve `304 Not Modified` protokolü desteklenir.
