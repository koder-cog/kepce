# Cloudflare Tunnel ve Origin İzolasyonu

Bu belge, `kepce.org` origin'ini dünyaya kapatıp tüm trafiği Cloudflare Tunnel
üzerinden yayınlamanın ve dahili ingest uç noktasını Access service token ile bot
challenge'ından muaf tutmanın operasyon rehberidir.

Belge kalıcıdır ve git ile sürümlenir. Token, secret veya proxy kimlik bilgisi
buraya **yazılmaz**, yalnızca ortam değişkeni adı verilir.

## 1. Mimari

```
İnternet
  └─> Cloudflare edge (WAF / Bot / Access)
        └─> Cloudflare Tunnel (şifreli, yalnızca dışa açılan bağlantı)
              └─> cloudflared (Docker, kepce-cloudflared)
                    └─> Caddy (kepce-caddy, https://caddy:443)
                          ├─> api:8000
                          └─> webapp:3000
```

Origin sunucusu dışarıdan **hiçbir bağlantı kabul etmez**, SSH (22) hariç.
cloudflared içeriden dışarıya bağlanır.

### Neden gerekli oldu

GitHub Actions runner'ının (Azure/Fastly egress) ve datacenter IP'lerinin
`/api/v1/internal/ingest/kykyemek` isteği Cloudflare tarafından "managed
challenge" ile engelleniyordu. WAF custom rule datacenter proxy'lerde çalıştı ama
runner egress katmanını kapsamadı. Tünel, origin portlarını kapatır ve Access
service token, ingest isteğini bot korumasından muaf tutar. İki mekanizma birlikte
kurulur, challenge'ı asıl aşan mekanizma **Access service token**'dır.

## 2. Mevcut durum

Kurulum 29 Eylül 2026'da tamamlandı ve ölçümle doğrulandı.

| Bileşen | Durum | Kanıt |
|---------|-------|-------|
| cloudflared | Çalışıyor | `docker ps` içinde `kepce-cloudflared`, log satırı `Registered tunnel connection` |
| Ingress | 3 hostname | `kepce.org`, `ara.kepce.org`, `analitik.kepce.org` için servis `https://caddy:443`, `noTLSVerify: true` |
| DNS | Tünele dönük | `https://kepce.org/` HTTP 200, `ara.kepce.org` 200, `analitik.kepce.org` 200 |
| Access uygulaması | Aktif | Token'sız POST yanıtı `403`, başlıklarda `cf-access-domain: kepce.org` ve `cf-access-aud`, gövde `Cloudflare Access` hata sayfası |
| Origin portları | Kapalı | `docker port kepce-caddy` çıktısı `127.0.0.1:80` ve `127.0.0.1:443`, dışarıdan 80/443 bağlantı reddi |
| Güvenlik duvarı | Yalnızca SSH | `ufw status verbose` içinde yalnızca `22/tcp` ALLOW |

## 3. Kurulum adımları (sıfırdan)

Sıra önemlidir. Önce tünel ve DNS, sonra port kapatma. Portları tünel yayına
girmeden kapatmak siteyi tamamen düşürür.

| Faz | Ne | Port durumu | Risk |
|-----|----|-------------|------|
| 1 | Tüneli oluştur, cloudflared başlat | Açık | Düşük |
| 2 | DNS'i tünele çevir | Açık | Orta |
| 3 | Access service token ve ingest testi | Açık | Düşük |
| 4 | Portları kapat | Kapanır | Yüksek |

Her fazdan sonra doğrula, bir sonraki faza ancak önceki yeşilse geç.

### Faz 1: Tünel ve cloudflared

1. Cloudflare panelinde **Zero Trust > Networks > Tunnels > Create a tunnel**.
   Bağlayıcı `Cloudflared`, isim `kepce-origin`. Çıkan token'ı sunucu `.env`
   dosyasına `CLOUDFLARE_TUNNEL_TOKEN` olarak yaz.
2. Public hostname'leri ekle. Her biri için **Service** alanı Caddy'ye işaret
   eder ve **No TLS Verify** açık olur.

   | Hostname | Service | No TLS Verify |
   |----------|---------|---------------|
   | `kepce.org` | `https://caddy:443` | Açık |
   | `www.kepce.org` | `https://caddy:443` | Açık |
   | `ara.kepce.org` | `https://caddy:443` | Açık |
   | `analitik.kepce.org` | `https://caddy:443` | Açık |

   `No TLS Verify` açık olmalı, çünkü Caddy Cloudflare Origin Certificate kullanır
   ve cloudflared bu sertifikayı doğrulayamaz.

3. Sunucuda başlat:

   ```bash
   cd ~/kepce
   docker compose -f docker-compose.yml -f docker-compose.prod.yml \
     -f docker-compose.analytics.yml -f docker-compose.search.yml \
     -f docker-compose.tunnel.yml --profile tunnel up -d cloudflared
   docker logs kepce-cloudflared 2>&1 | tail -20
   ```

   Logda `Registered tunnel connection` görünmeli, panelde tünel `Healthy` olmalı.

**Not:** `www.kepce.org` şu an ingress listesinde yok. `www` adresi apex'e `301`
dönüyor, yani yönlendirme Cloudflare edge'inde bir Redirect Rule ile yapılıyor.
İşlevsel sorun yok. Yönlendirmeyi origin'e taşımak istersen hostname'i ingress'e
ekle.

### Faz 2: DNS

Public hostname eklediğinde Cloudflare, ilgili DNS kaydını otomatik olarak
`<tunnel-id>.cfargotunnel.com` CNAME'ine çevirir (proxied). Mevcut A kayıtları
varsa silinir.

```bash
curl -sS -o /dev/null -w "kepce.org -> HTTP %{http_code}\n" https://kepce.org/
curl -sS -o /dev/null -w "ara -> HTTP %{http_code}\n" https://ara.kepce.org/
```

İkisi de `200` dönmeli. Dönmezse DNS yayılımını bekle ve panelde hostname
eşleşmesini kontrol et.

### Faz 3: Access service token

1. **Zero Trust > Access > Service Auth > Create Service Token**, isim
   `github-actions-ingest`. Verilen Client ID ve Client Secret'ı kaydet, secret
   bir daha görüntülenemez.
2. **Zero Trust > Access > Applications > Add an application > Self-hosted**.
   İsim `kepce-ingest`, domain `kepce.org`, path `api/v1/internal/ingest`.
   Policy: Action `Service Auth`, Include `Selector = Service Token` ve az önce
   oluşturulan token.
3. Repo **Settings > Secrets and variables > Actions** altına `CF_ACCESS_CLIENT_ID`
   ve `CF_ACCESS_CLIENT_SECRET` ekle.
4. Sunucudan test et. Proxy adresini `.env` içindeki `KYKYEMEK_PROXY_TOOL`
   değişkeninden oku, kimlik bilgisini komut satırına veya belgeye yazma:

   ```bash
   cd ~/kepce
   get() { grep "^$1=" .env | head -1 | cut -d= -f2- | tr -d '"' | tr -d "'" | tr -d '\r'; }
   SECRET=$(get INTERNAL_INGEST_SECRET)
   PROXY=$(get KYKYEMEK_PROXY_TOOL | cut -d, -f1)

   curl -sS -o /dev/null -w "Access token ile -> HTTP %{http_code}\n" \
     --max-time 20 -x "$PROXY" \
     -X POST https://kepce.org/api/v1/internal/ingest/kykyemek \
     -H "X-Internal-Token: $SECRET" \
     -H "CF-Access-Client-Id: $(get CF_ACCESS_CLIENT_ID)" \
     -H "CF-Access-Client-Secret: $(get CF_ACCESS_CLIENT_SECRET)" \
     -H "Content-Type: application/json" \
     -d '{"menus":[]}'
   ```

   Beklenen `HTTP 200`. `403` ise Access policy eşleşmiyor (path veya token
   hatası). `302` ise Access login sayfasına yönlendiriyor, yani policy yok.
   `401` ise Access geçildi ama `X-Internal-Token` yanlış.

5. Uçtan uca doğrulama: `scrape-kykyemek.yml` çalıştır ve logda
   `Aktarıldı -> Alınan: N` satırını gör.

### Faz 4: Portları kapat

1. [`docker-compose.prod.yml`](../../docker-compose.prod.yml) içinde Caddy
   portlarını loopback'e bağla:

   ```yaml
   ports:
     - "127.0.0.1:80:80"
     - "127.0.0.1:443:443"
   ```

2. Güvenlik duvarı:

   ```bash
   sudo ufw allow 22/tcp
   sudo ufw delete allow 80/tcp
   sudo ufw delete allow 443/tcp
   sudo ufw status verbose
   ```

3. Doğrula: dışarıdan origin IP'ye doğrudan erişim kapalı olmalı, site tünel
   üzerinden çalışmaya devam etmeli.

   ```bash
   curl -sS --max-time 5 -o /dev/null -w "%{http_code}\n" http://130.110.247.87/ || echo "kapali (beklenen)"
   curl -sS -o /dev/null -w "kepce.org -> HTTP %{http_code}\n" https://kepce.org/
   ```

## 4. Önemli: Docker, ufw'yi baypas eder

Origin'i kapatan asıl mekanizma `127.0.0.1:` bağlama adresidir, ufw değil. Docker
port publish ederken iptables kurallarını doğrudan yazar ve ufw zincirini atlar.
Yani `docker-compose.prod.yml` bir gün `0.0.0.0:443` sürümüne dönerse
`sudo ufw delete allow 443/tcp` bunu engellemez.

Sonuç: `docker-compose.prod.yml` içindeki loopback bağlaması **git ile
sürümlenmek zorundadır.** Bu satır commit edilmezse bir sonraki deploy repo'daki
sürümü sunucuya gönderir ve portlar sessizce dünyaya açılır.

## 5. Deploy hattı entegrasyonu

[`deploy.yml`](../../.github/workflows/deploy.yml) tünel kurulumunu iki şekilde
korur.

1. **Profil yönetimi:** `CLOUDFLARE_TUNNEL_TOKEN` tanımlıysa `--profile tunnel`
   eklenir ve `cloudflared` hem `pull` hem `up -d` hedefine girer. Böylece
   `--remove-orphans` cloudflared'ı yetim sayıp durdurmaz. Token boşsa profil
   açılmaz, konteyner hata döngüsüne girmez.
2. **Sağlık kapısı:** Tünel modu bekleniyorsa (token var veya `kepce-cloudflared`
   konteyneri mevcut) deploy şu üç koşulu arar ve sağlanmazsa hata ile durur:
   `kepce-cloudflared` ayakta, Caddy `443` portu `127.0.0.1:443` üzerinde bağlı,
   `https://kepce.org/` yanıt veriyor. Hata durumunda mevcut Telegram alarmı
   devreye girer.

Deploy sonrası elle doğrulama:

```bash
ssh ubuntu@130.110.247.87 'docker ps --format "{{.Names}}" | grep -E "cloudflared|caddy"; docker port kepce-caddy; curl -sS -o /dev/null -w "kepce.org -> %{http_code}\n" https://kepce.org/'
```

## 6. Sorun giderme

| Belirti | Olası neden | Çözüm |
|---------|-------------|-------|
| `523` / `521` | Origin erişilemiyor, ingress hostname eksik | Ingress listesini ve `https://caddy:443` servisini kontrol et |
| `tls: internal error` | Caddy sunucu adı eşleşmiyor | İlgili hostname için `originServerName` ekle (analitik için bu şekilde çözüldü) |
| Ingest `403` + `cf-access-domain` başlığı | Access policy eşleşmiyor | Uygulama domain/path'ini `kepce.org` + `api/v1/internal/ingest` olarak kontrol et |
| Ingest `302` | Access policy yok | `Service Auth` policy'sini ekle |
| Ingest `401` | Access geçildi, dahili token yanlış | GitHub secret `INTERNAL_INGEST_SECRET` ile sunucu `.env` değerini eşitle |
| Site 200 ama alt alan adı düşük | Ingress hostname eksik | Panelde public hostname ekle |

## 7. Geri alma

1. **DNS:** Panelde hostname'i eski A kaydına (`130.110.247.87`, proxied) döndür.
2. **Portlar:** [`docker-compose.prod.yml`](../../docker-compose.prod.yml)
   içindeki portları `"80:80"` ve `"443:443"` yap, ardından
   `sudo ufw allow 80/tcp` ve `sudo ufw allow 443/tcp` ile aç.
3. **cloudflared:**
   `docker compose -f docker-compose.yml -f docker-compose.prod.yml -f docker-compose.tunnel.yml --profile tunnel down cloudflared`

## 8. İlgili dosyalar

- [`docker-compose.tunnel.yml`](../../docker-compose.tunnel.yml): cloudflared servisi, `profiles: [tunnel]` ile kapılı.
- [`docker-compose.prod.yml`](../../docker-compose.prod.yml): Caddy port bağlamaları.
- [`deploy.yml`](../../.github/workflows/deploy.yml): profil yönetimi ve sağlık kapısı.
- [`scrape-kykyemek.yml`](../../.github/workflows/scrape-kykyemek.yml): `CF_ACCESS_CLIENT_ID` ve `CF_ACCESS_CLIENT_SECRET` secret'larını binary'ye geçirir.
- [`remote_scraper.rs`](../../worker/src/bin/remote_scraper.rs): ingest isteğine `CF-Access-Client-Id` ve `CF-Access-Client-Secret` başlıklarını ekler.
- [`.env.example`](../../.env.example): `CLOUDFLARE_TUNNEL_TOKEN`, `CF_ACCESS_CLIENT_ID`, `CF_ACCESS_CLIENT_SECRET` belgeleri.
