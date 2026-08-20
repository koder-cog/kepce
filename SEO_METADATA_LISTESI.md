# Kepçe (kepce.org) - Tüm Sayfaların SEO Başlık ve Meta Açıklama Listesi

Bu belgede, arama motorlarında (Google, Bing, Yandex vb.) çıkacak olan güncel başlık (Title), meta açıklama (Description) ve yapısal veri (Schema) detayları listelenmiştir.

---

## 1. Ana Sayfa & Şehir Sayfaları

### 🏠 Ana Sayfa (`/`)
* **Title:** `Bugün KYK'da Ne Yemek Var? | Kepçe`
* **Description:** `Bugün KYK yurtlarında çıkan sabah kahvaltısı ve akşam yemeği menüsü. Reklamsız, güncel yemekhane listeleri ve öğrenci değerlendirmeleri.`
* **Canonical:** `https://kepce.org`
* **Schema (JSON-LD):** `WebSite`, `Organization`, `Menu` (Günlük yemek listesi)

### 📍 81 Şehir Sayfası (Örn: `/istanbul` veya `/ankara`)
* **Title:** `[Şehir Adı] KYK Yemek Menüsü | Kepçe` (Örn: `İstanbul KYK Yemek Menüsü | Kepçe`)
* **Description:** `[Şehir Adı] KYK yurtlarında bugün çıkan sabah kahvaltısı ve akşam yemeği menüsü. Reklamsız, güncel yemek listeleri ve öğrenci yorumları.`
* **Canonical:** `https://kepce.org/[sehir_slug]`
* **Schema (JSON-LD):** `Menu` (İlgili şehrin güncel tabldot menüsü)

---

## 2. Arşiv & Menü Detay Sayfaları

### 📅 Geçmiş Menü Arşivi (`/arsiv`)
* **Title:** `KYK Yemek Menüsü Arşivi | Kepçe`
* **Description:** `Geçmiş aylara ait KYK yurt yemekhane menüleri. Şehrinizi ve tarihi seçerek eski günlerde ne çıktığını inceleyin.`
* **Canonical:** `https://kepce.org/arsiv`

### 🍲 Tekil Menü Detayı (`/menu/[id]`)
* **Title:** `[Tarih] [Şehir] KYK Yemek Menüsü | Kepçe` (Örn: `20 Ağustos 2026 Perşembe İstanbul KYK Yemek Menüsü | Kepçe`)
* **Description:** `[Tarih] tarihli [Şehir] KYK yurt yemek menüsü detayları, besin değerleri ve öğrenci yorumları.`
* **Canonical:** `https://kepce.org/menu/[id]`
* **Schema (JSON-LD):** `Menu`, `MenuItem`, `AggregateRating` (öğrenci oylamaları varsa)

---

## 3. Keşfet & Rehber Sayfaları

### ⏰ KYK Yemek Saatleri (`/kyk-yemek-saatleri`)
* **Title:** `KYK Yemek Saatleri | Kepçe`
* **Description:** `KYK yurtlarında sabah kahvaltısı ve akşam yemeği saatleri kaçta başlıyor, kaçta bitiyor? Hafta içi ve hafta sonu yemekhane saat tablosu.`
* **Canonical:** `https://kepce.org/kyk-yemek-saatleri`
* **Schema (JSON-LD):** `Article`

### 💳 KYK Beslenme Yardımı (`/kyk-beslenme-yardimi`)
* **Title:** `KYK Beslenme Yardımı | Kepçe`
* **Description:** `KYK yurtlarında günlük yemek yardımı ne kadar? Sabah ve akşam beslenme yardımı kullanım kuralları ve limit aşımı detayları.`
* **Canonical:** `https://kepce.org/kyk-beslenme-yardimi`
* **Schema (JSON-LD):** `Article`

### ❓ Sıkça Sorulabilecek Sorular (`/sss`)
* **Title:** `Sıkça Sorulabilecek Sorular | Kepçe`
* **Description:** `Kepçe platformu, KYK yurt yemek menüleri, verilerin doğruluğu ve topluluk kuralları hakkında sıkça sorulan sorular.`
* **Canonical:** `https://kepce.org/sss`
* **Schema (JSON-LD):** `FAQPage`

### 📊 Yemek İstatistikleri (`/istatistikler/yemekler`)
* **Title:** `Yemek İstatistikleri ve Analizleri | Kepçe`
* **Description:** `KYK yurtlarında en çok sevilen ve en az beğenilen yemekler, puanlamalar ve öğrenci oylama istatistikleri.`
* **Canonical:** `https://kepce.org/istatistikler/yemekler`

### 💬 Yorum İstatistikleri (`/istatistikler/yorumlar`)
* **Title:** `En Beğenilen Öğrenci Yorumları | Kepçe`
* **Description:** `KYK yemekhaneleri hakkında en çok oy alan, öne çıkan öğrenci yorumları ve değerlendirmeleri.`
* **Canonical:** `https://kepce.org/istatistikler/yorumlar`

### 🤝 İnsaniyet Endeksi (`/istatistikler/insaniyet`)
* **Title:** `İnsaniyet ve Topluluk Tablosu | Kepçe`
* **Description:** `Kepçe topluluğu insaniyet metrikleri, yardımseverlik ve etkileşim istatistikleri.`
* **Canonical:** `https://kepce.org/istatistikler/insaniyet`

### 🛡️ Denetim & Şeffaflık (`/istatistikler/denetim`)
* **Title:** `Denetim ve Moderasyon İstatistikleri | Kepçe`
* **Description:** `Kepçe platformundaki moderasyon hareketleri, şikayetler ve sistem şeffaflık raporları.`
* **Canonical:** `https://kepce.org/istatistikler/denetim`

---

## 4. Bağlantılar & Araçlar

### ℹ️ Hakkında (`/hakkinda`)
* **Title:** `Hakkında | Kepçe`
* **Description:** `Kepçe; KYK yurtlarında kalan öğrencilerin günlük yemek menülerine şeffaf, reklamsız ve hızlıca ulaşması için geliştirilmiş açık kaynaklı ve bağımsız bir platformdur.`
* **Canonical:** `https://kepce.org/hakkinda`

### ⚡ Sistem Durumu (`/durum`)
* **Title:** `Sistem Durumu | Kepçe`
* **Description:** `Kepçe API, web uygulaması, veri tabanı ve arka plan servislerinin anlık çalışma durumu ve geçmiş kesinti raporları.`
* **Canonical:** `https://kepce.org/durum`

### 📤 Menü Gönder (`/menu-gonder`)
* **Title:** `Menü Gönder | Kepçe`
* **Description:** `Yurdunuzun yemekhane menüsünü veya tabldot fotoğrafını yükleyin, Kepçe veritabanına katkıda bulunun.`
* **Canonical:** `https://kepce.org/menu-gonder`

### 📡 RSS Akışları (`/rss`)
* **Title:** `RSS Akışları | Kepçe`
* **Description:** `KYK yemekhane menülerini RSS veya JSON feed ile takip edin. Otomasyon ve bildirim entegrasyonları.`
* **Canonical:** `https://kepce.org/rss`

---

## 5. Yasal Sayfalar

### 📜 Kullanım Koşulları (`/kullanim-kosullari`)
* **Title:** `Kullanım Koşulları | Kepçe`
* **Description:** `Kepçe platformunun kullanım şartları, sorumluluk reddi ve kuralları.`
* **Canonical:** `https://kepce.org/kullanim-kosullari`

### 🔒 Gizlilik Politikası (`/gizlilik-politikasi`)
* **Title:** `Gizlilik Politikası | Kepçe`
* **Description:** `Kepçe platformunun veri güvenliği, gizlilik ve çerez politikası.`
* **Canonical:** `https://kepce.org/gizlilik-politikasi`

### ✉️ İletişim / Künye (`/iletisim`)
* **Title:** `İletişim / Künye | Kepçe`
* **Description:** `Kepçe ekibiyle iletişime geçin. Geri bildirim, menü düzeltme ve iş birliği talepleri.`
* **Canonical:** `https://kepce.org/iletisim`
