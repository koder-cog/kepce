# Kepçe - Önüne Hazır Konanı Yeme Sanatı

Ön Uyarı: Bu proje henüz geliştirme aşamasında olup bazı eksiklikler barındırmaktadır. Planlanan geliştirmelere ve güncel duruma [görev listemizden](/.scratch/docs/planning/task.md) ulaşabilirsiniz.

[![Web Sitesi](https://img.shields.io/badge/Web_Sitesi-kepce.org-blue?style=flat-square)](https://kepce.org)
[![Lisans: AGPL v3](https://img.shields.io/badge/Lisans-AGPL%20v3-blue.svg?style=flat-square)](https://www.gnu.org/licenses/agpl-3.0#license-text)

Açık kaynaklı, şeffaf ve güvenilir KYK yemekhane veri ağı. Kepçe, dağınık KYK tabela listelerini toplayan ve temiz bir REST API ile sunan bir topluluk projesidir.

## Ekran Görüntüleri

<div align="center">
<table width="100%">
<tr>
<td align="center" width="76.5%">
<img src="docs/images/masaüstü-koyu.png" alt="Kepçe Masaüstü (Karanlık Tema)" width="100%"/>
</td>
<td align="center" width="23.5%">
<img src="docs/images/mobil-açık.png" alt="Kepçe Mobil (Açık Tema)" width="100%"/>
</td>
</tr>
</table>
</div>

## Özellikler ve Teknolojiler

- **Backend:** Rust, Axum, SeaORM, PostgreSQL (Veritabanı ilişkileri ve hızlı API sunumu, Redis bağımlılığı yoktur)
- **Frontend:** SvelteKit 2 (Svelte 5) + `adapter-static` ve saf (Vanilla) CSS, Vite
- **Veri Madenciliği & Ayrıştırma:** Otomatik Excel/PDF ayrıştırıcıları (Rust tabanlı worker)
- **Veri Bütünlüğü:** Hash Chain mimarisi ile değiştirilemez (immutable) ve doğrulanabilir veri geçmişi (Her yeni menü bir önceki menünün hash'ini referans alarak zincirlenir)
- **Yapay Zeka (Opsiyonel):** Gemini API (`gemini-flash-lite-latest`) ile PDF tabanlı akıllı menü özetleme/çıkarma (Eğer `GEMINI_API_KEY` tanımlanmamışsa PDF menü ayrıştırma devre dışı kalır)

## Kurulum ve Çalıştırma

Gereksinimler: Rust (cargo), Node.js 20+ ve Podman (veya Docker).

### 1. Yerel Geliştirme (Local Development)

Sistemi tek komutla yerel olarak başlatmak için kök dizindeki `manage.sh` betiğini kullanabilirsiniz:

```bash
# Tüm servisleri (Veritabanı, API, Web, Worker) yerel makinede ayağa kaldır
./manage.sh start

# Sistem durumunu gör
./manage.sh status

# Logları takip et
./manage.sh logs api
./manage.sh logs web
./manage.sh logs db

# Durdur
./manage.sh stop
```

### 2. Konteynerli Dağıtım (Containerized Deployment)

Sistemdeki tüm servisleri izole konteynerler halinde çalıştırmak için `podman-compose` kullanabilirsiniz:

```bash
# Konteynerleri build et ve arka planda çalıştır
podman-compose up -d --build

# Konteyner loglarını gör
podman-compose logs -f
```

## Canlı Ortam (Production) Süreçleri

- **CI/CD Kurulumu:** GitHub Actions ile otomatik test (`cargo test`, `cargo clippy`) ve SvelteKit static build (`npm run build`) süreçleri otomatize edilmelidir.
- **Staging Ortamı:** Tüm değişiklikler production öncesinde `staging.kepce.org` gibi izole bir ortamda test edilmelidir.
- **Monitoring:** Sistem sağlığını takip etmek için Uptime Kuma kurulmalıdır (CPU/RAM, API endpoint yanıt süreleri, SSL geçerliliği).
- **Otomatik Yedekleme:** Veritabanı yedekleme betiği (`scripts/backup_db.sh`) sunucuda cron job olarak tanımlanmalıdır.

## Katkıda Bulunma

Kepçe, topluluk odaklı bir projedir. Yeni özellikler eklemek, hata bildirmek veya dokümantasyonu geliştirmek isterseniz bir Issue açın veya Pull Request gönderin. Her türlü katkıya açığız.

## Lisans & Telif Hakkı

Copyright (C) 2026 Kepçe Contributors.

Bu proje **GNU Affero General Public License v3.0 (AGPL-3.0)** lisansı ile korunmaktadır. Detaylar için [LICENSE](LICENSE) dosyasına bakınız.

### AGPL Network-Source Yükümlülük Notu

AGPL-3.0 lisansı gereği, bu yazılımı bir ağ (network) üzerinden sunan (sağlayan) her kuruluş, kullanıcılara yazılımın **kaynak koduna erişim sağlama yükümlülüğüne** sahiptir. Bu doğrultuda, Kepçe arayüzünde (web uygulamasında) projenin açık kaynak kodlu repository adresine (örneğin GitHub bağlantısı) yönlendiren bir link barındırılması yasal bir zorunluluktur.