# Kepçe - Önüne Hazır Konanı Yeme Sanatı

Ön Uyarı: Bu proje henüz geliştirme aşamasında olup eksiklikler barındırmaktadır. Öneri veya hata bildirimleri için [Issues](https://github.com/kepce-app/kepce/issues) sekmesini kullanabilirsiniz.

[![Web Sitesi](https://img.shields.io/badge/Web_Sitesi-kepce.org-blue?style=flat-square)](https://kepce.org)
[![AGPL v3](https://img.shields.io/badge/Lisans-AGPL%20v3-blue.svg?style=flat-square)](https://www.gnu.org/licenses/agpl-3.0#license-text)

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

## Teknolojiler

- **Backend:** Rust (Axum, SeaORM) ve PostgreSQL
- **Frontend:** SvelteKit 2 (Svelte 5) ve CSS
- **Veri Madenciliği:** Rust tabanlı otomatik Excel ve PDF ayrıştırıcı worker
- **Veri Bütünlüğü:** Hash Chain mimarisi ile değiştirilemez geçmiş kaydı
- **Yapay Zeka (Opsiyonel):** Gemini Flash ile PDF menü özetleme

## Kurulum ve Çalıştırma

Gereksinimler: Rust (cargo), Node.js 20+ ve Docker (veya Podman).  
*Not: Windows ortamında betikleri çalıştırmak için WSL2 (Windows Subsystem for Linux) önerilir.*

### 1. Yerel Geliştirme (Local Development)

Sistemi tek komutla yerel olarak başlatmak için kök dizindeki `manage.sh` betiğini kullanabilirsiniz:

```bash
# Tüm servisleri yerel makinede ayağa kaldır
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

Sistemdeki tüm servisleri izole konteynerler halinde çalıştırmak için:

```bash
# Konteynerleri derle ve arka planda çalıştır
docker compose up -d --build

# Konteyner loglarını takip et
docker compose logs -f
```

## Katkıda Bulunma

Kepçe, topluluk odaklı bir projedir. Yeni özellikler eklemek, hata bildirmek veya dokümantasyonu geliştirmek isterseniz bir Issue açabilir veya Pull Request gönderebilirsiniz.

## Lisans & Telif Hakkı

Telif Hakkı (C) 2026 Kepçe Katkıda Bulunanları.

Bu proje **GNU Affero General Public License v3.0 (AGPL-3.0)** ile lisanslanmıştır. Detaylar için [LICENSE](LICENSE) dosyasına bakabilirsiniz.