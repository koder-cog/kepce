# Veritabanı Mimarisi ve Şema Yapısı

Bu dizin, Kepçe PostgreSQL veritabanının güncel şemasını etki alanlarına göre ayrılmış SQL dosyalarıyla belgeler.

## Makine ve Geliştirici Ayrımı

Veritabanı yapısı iki farklı ihtiyaca göre iki ayrı dizinde tutulur:

| Dizin | Amaç | Hedef | Yapı |
| :--- | :--- | :--- | :--- |
| `db/migrations/` | Sürüm takibi ve sırayla yürütme. | Veritabanı motoru (`schema_migrations`), testler ve dağıtım araçları. | `0001_...` ile `0026_...` arası sıralı yama dosyaları. |
| `docs/database/` | Sistemin güncel halini inceleme. | Geliştiriciler ve mimari planlama. | Yamalardan arındırılmış, etki alanlarına bölünmüş SQL dosyaları. |

## Etki Alanları

1. [01_core.sql](01_core.sql)
   - `cities`: 81 il ve plaka ile slug eşleştirmeleri.
   - `dishes`: Ana yemek kataloğu, diyet nitelikleri (`is_vegan`, `is_celiac`), kalori ve kategori bilgisi.
   - `dish_aliases`: Farklı KYK kaynaklarından gelen yemek adlarının ana yemek kaydıyla eşleştirilmesi.
   - `tags` ve `dish_tags`: Yemek etiketleri ve sınıflandırma tablosu.

2. [02_identity.sql](02_identity.sql)
   - `users`: Kullanıcı hesapları, karma puanı, bildirim tercihleri ve seviye sistemi.
   - `user_sessions`: JWT oturumları, cihaz ve IP kayıtları.
   - `used_tokens`: Çıkış yapılmış veya süresi dolmuş belirteçlerin kara listesi.
   - `user_favorites` ve `user_pinned_dishes`: Beğenilen veya profile sabitlenen yemekler.
   - `user_blocks` ve `user_warnings`: Kullanıcı engelleri ve hesap uyarıları.
   - `badges` ve `user_badges`: Rozet kataloğu ve kullanıcılara verilen rozetler.

3. [03_menus.sql](03_menus.sql)
   - `menus`: Günlük tabldot menüleri, öğün tipi, veri kaynağı, onay durumu ve kalori aralığı.
   - `menu_dishes`: Menüdeki yemek yuvaları (`order_index`), alternatifler ve paket adı.
   - `menu_history`: Menülerin önceki sürümleri ve kaynak geçmişi.
   - `menu_submissions`: Kullanıcıların ilettiği menü bildirimleri.
   - `pricing_periods` ve `meal_category_prices`: Şehir ve tarih bazlı tabldot fiyat dönemleri.

4. [04_social.sql](04_social.sql)
   - `comments`: Menü ve yemeklere yapılan hiyerarşik yorumlar ile silinme gerekçeleri.
   - `vote_reactions`: Yorumlara verilen beğeni ve karşıt oylar.
   - `menu_votes` ve `dish_votes`: Menü ve yemek bazlı duygu değerlendirmeleri.

5. [05_moderation.sql](05_moderation.sql)
   - `reports`: Menü, yorum ve kullanıcı şikayetleri.
   - `contact_messages`: İletişim formu üzerinden gelen mesajlar.
   - `notifications`: Kullanıcılara gönderilen site içi bildirimler.
   - `push_subscriptions`: Tarayıcı bildirim abonelikleri ve gönderim saatleri.

6. [06_platform.sql](06_platform.sql)
   - `projects` ve `api_keys`: Geliştirici projeleri ve yetkilendirme anahtarları.
   - `api_usage_logs`: Günlük istek ve hata sayaçları.
   - `system_incidents`: Altyapı kesintileri ve durum kayıtları.

7. [07_indexes.sql](07_indexes.sql)
   - Veri bütünlüğü tetikleyicisi: `trg_menu_dishes_unique_dish` ile aynı menü yuvasında mükerrer yemek kaydı engellenir.
   - Kısmi tekillik indeksi: `uq_menu_dishes_primary_slot` ile her menü yuvasına yalnızca tek bir ana yemek tanımlanabilir.
   - Yabancı anahtar ve sorgu indeksleri: Tablolar arası ilişkiler ve tarih filtreleri için tanımlanan performans indeksleri.
