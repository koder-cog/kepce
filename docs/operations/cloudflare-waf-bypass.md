# Cloudflare: Dahili Ingest Uç Noktası İçin Challenge Muafiyeti

Dahili ingest uç noktasının (`/api/v1/internal/ingest/kykyemek`) Cloudflare
tarafından challenge edilmesini, yalnızca o yola özel ve güvenlik açığı
yaratmayan bir ayarla çözmek için izlenen yol ve mevcut durum.

## Mevcut durum (29 Eylül 2026)

Bu belge iki katmanı ayırır ve ikisi birlikte kullanılır:

1. **WAF custom rule (bu belge):** Datacenter IP'ler için çalışır. Ölçümle
   doğrulandı, Webshare proxy havuzundaki 10 uç noktanın tamamı doğru token ile
   `HTTP 200`, token'sız `HTTP 401` döndü.
2. **Cloudflare Tunnel + Access service token (kalıcı çözüm):** GitHub-hosted
   runner'ın Azure/Fastly egress'i WAF kuralını aşamadığı için asıl çözüm budur.
   Kurulum ve doğrulama: [`cloudflare-tunnel.md`](cloudflare-tunnel.md).

WAF kuralı kaldırılmadı, ek katman olarak duruyor. Doğrudan sunucu çıkışı
(proxy'siz) zaten kullanılmıyor.

## Neden güvenli

- Uç nokta gizli `X-Internal-Token` başlığı ile korunur. API tarafında SHA-256
  sabit zamanlı karşılaştırma yapılır
  ([`api/src/routes/internal_ingest.rs`](../../api/src/routes/internal_ingest.rs)).
- Token'sız veya yanlış token'lı istek `401` alır. Yani CDN challenge'ını bu
  yolda kaldırmak veriyi dışarı açmaz, yalnızca bot koruması katmanını devre dışı
  bırakır.
- Ayar yalnızca `/api/v1/internal/ingest/*` yoluna uygulanır, sitenin geri kalanı
  etkilenmez.

## Adım 1: WAF Custom Rule (Skip), birincil yaklaşım

Cloudflare Dashboard, **Security > WAF > Custom rules > Create rule**

- **Name:** `internal-ingest-skip-challenge`
- **Expression (Edit expression):**
  ```
  (http.request.uri.path eq "/api/v1/internal/ingest/kykyemek"
   and http.request.headers["x-internal-token"][0] eq "<INTERNAL_INGEST_SECRET_DEGERI>")
  ```
  `<INTERNAL_INGEST_SECRET_DEGERI>` yerine sunucu `.env` dosyasındaki gerçek
  değeri yaz. Secret rotasyonunda bu kural da güncellenmelidir.
- **Action:** **Skip**. "WAF components to skip" altında şunları işaretle:
  - All remaining custom rules
  - All rate limiting rules
  - All managed rules
  - All Super Bot Fight Mode Rules
  - **Security Level** (More components to skip altında, managed challenge'ın en
    olası kaynağı budur)
  - Browser Integrity Check (opsiyonel, zararsız)
- **Deploy**

Kural, yalnızca doğru token'ı taşıyan isteğin challenge'ı atlamasını sağlar.
Token'ı bilmeyen istek challenge'a takılmaya devam eder.

## Adım 2: Challenge hâlâ geliyorsa

Cloudflare Dashboard, **Rules > Configuration Rules > Create rule**

- **Name:** `internal-ingest-security-level-off`
- **If:** `http.request.uri.path matches "^/api/v1/internal/ingest/"`
- **Then:** **Security Level = Essentially Off**
- **Deploy**

## Adım 3 (opsiyonel, defense in depth): Token'sız isteği WAF'ta blokla

- **Name:** `internal-ingest-block-without-token`
- **Expression:**
  ```
  (http.request.uri.path eq "/api/v1/internal/ingest/kykyemek"
   and http.request.headers["x-internal-token"][0] eq "")
  ```
- **Action:** **Block**

Uygulama katmanı zaten `401` döner, bu ek bir kalkandır.

## Alternatif (kullanılmıyor): Cloudflare'i tamamen atla

Ingest için Cloudflare'siz (grey cloud) ayrı bir alt alan adı kullanmak en
dayanıklı alternatiftir, ancak origin portlarının kapanmasıyla birlikte
uygulanabilir değildir:

1. DNS'te `ingest.kepce.org` kaydı oluştur, proxy kapalı olacak şekilde origin
   IP'ye yönlendir.
2. Origin'de (Caddy) bu alt alan adı için TLS sertifikası sağla.
3. `INTERNAL_INGEST_URL` değerini `https://ingest.kepce.org/api/v1/internal/ingest/kykyemek`
   yap.

Origin 80/443 portları dünyaya kapalı olduğu için bu yol ancak portlar açılırsa
işler. Yerine Cloudflare Tunnel + Access kullanılır.

## Doğrulama

1. `scrape-kykyemek.yml` iş akışını tetikle.
2. Logda şu satırı ara:
   ```
   [<sehir>] Aktarıldı -> Alınan: N, Eklenen: N, Güncellenen: N, Atlanan: N
   ```
3. `Alınan: 0` yerine pozitif bir sayı görürsen hat açılmıştır.
4. Hâlâ `403 Just a moment...` görürsen sorun WAF katmanında değil, egress
   yolundadır. [`cloudflare-tunnel.md`](cloudflare-tunnel.md) adımlarına geç.

## Sorun giderme

| Yanıt | Anlamı |
|-------|--------|
| `403` + `cf-access-domain` başlığı | Access uygulaması isteği yakaladı, policy veya service token eşleşmiyor |
| `403` + `Just a moment...` gövdesi | Bot challenge. WAF/Access muafiyeti bu egress yolunu kapsamıyor |
| `401` | Access geçildi, `X-Internal-Token` eksik veya yanlış |
| `404` | Yol yanlış, denetim sırasında yolu birebir doğrula |
| `200` | Hat açık |
