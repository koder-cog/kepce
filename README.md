# Kepçe - Önüne Hazır Konanı Yeme Sanatı

[![Web Sitesi](https://img.shields.io/badge/Web_Sitesi-kepce.org-blue?style=flat-square)](https://kepce.org)
[![CI Pipeline](https://img.shields.io/github/actions/workflow/status/koder-cog/kepce/ci.yml?branch=main&label=CI&style=flat-square)](https://github.com/koder-cog/kepce/actions)
[![AGPL v3](https://img.shields.io/badge/Lisans-AGPL%20v3-blue.svg?style=flat-square)](https://www.gnu.org/licenses/agpl-3.0#license-text)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/Frontend-SvelteKit%202%20(Svelte%205)-ff3e00?style=flat-square&logo=svelte)](https://svelte.dev/)

Açık kaynaklı, şeffaf ve güvenilir KYK yemekhane veri ağı. Kepçe; dağınık yurt tabela listelerini, Excel tablolarını ve görsel menüleri otomatik toplayan, standartlaştıran ve temiz bir REST API ile öğrencilere sunan bir topluluk projesidir.

---

## ⚖️ Yasal Sorumluluk Reddi (Legal Disclaimer)

> **Önemli Not:** Kepçe, Gençlik ve Spor Bakanlığı (GSB) veya Kredi ve Yurtlar Kurumu (KYK) ile doğrudan veya dolaylı hiçbir kurumsal bağı bulunmayan, kâr amacı gütmeyen, bağımsız bir **açık kaynak topluluk projesidir**. Menü, gramaj ve fiyat verileri; kamusal yurt panoları, resmi duyurular ve açık kaynaklardan öğrencileri bilgilendirme ve şeffaflık amacıyla derlenmektedir.

---

## 🌟 Öne Çıkan Özellikler

* **Resmi Fiyat ve Porsiyon Motoru:** Bakanlık onaylı porsiyon gramajları ve öğün bazlı fiyatlandırma kuralları ile otomatik eşleştirme.
* **Akıllı Alternatif Yönetimi:** Seçenekli yemekleri (`/` veya `VEYA`) bağımsız porsiyon ve kalori değerleriyle ayrıştırma.
* **Al-Götür & Sahur Paketleri:** Standart tabldot haricindeki paket ve kumanya menülerini hiyerarşik paket yapısıyla sunma.
* **Çölyak ve Diyet Menüleri:** Glütensiz beslenen öğrenciler için ayrı onay ve menü akışı.
* **Kriptografik Veri Bütünlüğü:** Her menü kaydı SHA-256 hash zinciriyle doğrulanarak geçmişe dönük tahrifata karşı korunur.
* **Dönem & Şehir İzolasyonu:** Şehir bazlı fiyatlandırma ve tatil/nöbetçi yurt dönemlerinde (Temmuz & Ağustos) dinamik kurallar.

---

## 📱 Ekran Görüntüleri

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

---

## 📚 Dokümantasyon

* 🏛️ [Sistem Mimarisi ve Veri Akışı](docs/ARCHITECTURE.md)
* 📡 [REST API Referansı](docs/API.md)
* 🤝 [Katkıda Bulunma Rehberi](CONTRIBUTING.md)
* 🛡️ [Güvenlik Politikası](SECURITY.md)
* 📜 [Topluluk Kuralları](CODE_OF_CONDUCT.md)

---

## 🛠️ Kurulum ve Çalıştırma

Gereksinimler: Rust `1.80+` (cargo), Node.js `20+` ve Docker / Podman.  
*Windows ortamında betikleri çalıştırmak için WSL2 önerilir.*

### 1. Yerel Geliştirme (Local Dev)

Sistemi tek komutla yerel olarak başlatmak için `manage.sh` yönetim betiğini kullanabilirsiniz:

```bash
# Örnek ortam değişkenlerini hazırla
cp .env.example .env

# Tüm servisleri yerel makinede ayağa kaldır
./manage.sh start

# Servislerin durumunu gör
./manage.sh status

# Logları canlı takip et
./manage.sh logs api
./manage.sh logs web
./manage.sh logs db

# Durdur
./manage.sh stop
```

### 2. Konteynerli Dağıtım (Production Docker)

```bash
# Konteynerleri derle ve arka planda çalıştır
docker compose up -d --build

# Konteyner loglarını takip et
docker compose logs -f
```

---

## 🤝 Katkıda Bulunma

Kepçe, topluluk odaklı bir projedir. Yeni özellikler eklemek, hata bildirmek veya dokümantasyonu geliştirmek isterseniz bir [Issue](https://github.com/koder-cog/kepce/issues) açabilir veya [Pull Request](CONTRIBUTING.md) gönderebilirsiniz.

---

## 📄 Lisans & Telif Hakkı

Telif Hakkı (C) 2026 Kepçe Katkıda Bulunanları.

Bu proje **GNU Affero General Public License v3.0 (AGPLv3)** ile lisanslanmıştır. Detaylar için [LICENSE](LICENSE) dosyasına veya [gayriresmi Türkçe çevirisine](LICENSE_TR) bakabilirsiniz.