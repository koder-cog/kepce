# Kepçe Sistem Mimarisi

Bu belge, Kepçe platformunun ana servislerini, veri akışını ve temel kurallarını özetler.

## Genel Bakış

```mermaid
graph TD
    User["Kullanıcı / Tarayıcı"] -->|HTTPS| Caddy["Caddy Reverse Proxy"]
    Caddy -->|/api/*| API["Kepçe REST API (Rust/Axum)"]
    Caddy -->|/*| Webapp["Web Uygulaması (SvelteKit)"]
    
    Source["Yemekhane Listeleri / Dosyalar"] --> Worker["Worker & Ayrıştırıcı (Rust)"]
    Worker --> DB[("PostgreSQL")]
    API --> DB
```

## Servisler

1. `api` (Rust / Axum):
   - REST API uç noktalarını sunar.
   - Kimlik doğrulama (JWT ve Magic Link), oylama ve yorum işlerini yönetir.
   - Şehre ve tarihe göre resmi fiyat/porsiyon hesaplamalarını yapar.
   - Hız sınırlama (Rate Limiting) ve önbellek başlıklarını yönetir.

2. `worker` (Rust):
   - Excel (`calamine`) tablolarını ve harici panolardan gelen menü verilerini ayrıştırıp veritabanına işler.
   - Taranmış, yamuk veya bozuk resmi belgelerden yapılandırılmış veri çıkarma.
   - Eksik veya kopya verileri temizler.

3. `webapp` (SvelteKit):
   - Kullanıcıların menüleri incelediği, arama ve filtreleme yaptığı mobil uyumlu web arayüzü.

4. `db` (PostgreSQL):
   - Şehir, menü, yemek, fiyat tarifesi ve kullanıcı verilerini saklayan ilişkisel veritabanı.

## Veri Kuralları

- Kaynak Önceliği: Doğrulanmış veya yönetici onaylı menü kayıtları, otomatik taranan ham verilerin önüne geçer.
- Şehir İzolasyonu: Yemek fiyatları ve porsiyonlar yalnızca o şehrin geçerli resmi tarifesi üzerinden hesaplanır; tarifesi olmayan şehirlerde tahmini fiyat gösterilmez.
- Denetim Günlüğü: Onaylanan menülerin değişiklik geçmişi kayıt altında tutulur.
