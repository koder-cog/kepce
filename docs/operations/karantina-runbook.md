# Karantina Kuyruğu Operatör Kılavuzu (Runbook)

Yerel menü ingest akışında (`bekleyen/` drop-zone) şüpheli dosyaların karantinaya
alınması, Telegram üzerinden karara bağlanması ve zaman aşımı davranışı.

İlgili plan: `.scratch/docs/planning/yerel_menu_akis_saglamlastirma_plani.md`
İlgili kod: [`worker/src/tasks/quarantine.rs`](../../worker/src/tasks/quarantine.rs),
[`worker/src/tasks/file_ingest.rs`](../../worker/src/tasks/file_ingest.rs),
[`worker/src/tasks/telegram_bot.rs`](../../worker/src/tasks/telegram_bot.rs)

## Temel ilkeler

1. **Sessiz başarı yoktur.** Dosya `vault/`'a ancak tam ve tutarlı çıkarıldığında gider.
2. **Şüpheli veri veritabanına asla yazılmaz.** Karar, yazma işleminden ÖNCE verilir.
3. **Operatör dosya tamir etmez, karar verir.** Tek dokunuşla `/onayla` veya `/reddet`.
4. **Operatör kararı yoksa zaman aşımı karar verir.** TTL (varsayılan 14 gün) dolar,
   dosya `hatali/`'ya taşınır ve kırmızı alarm üretilir.
5. **Tarih yorumu deterministiktir.** LLM yalnızca ham metni taşır.

## Klasör sözleşmesi

```
data/menuler/<rol>/                      # rol: admin | kullanici | anonim
├── bekleyen/<sehir>/                    # İŞLENME KUYRUĞU (worker buradan okur)
├── _karantina/                          # KARAR BEKLİYOR (DB'ye hiçbir şey yazılmadı)
│   ├── <dosya>
│   └── <dosya>.karantina.json           # karar meta verisi (yan dosya)
├── hatali/                              # KALICI HATA (bir daha denenmez)
└── (vault) data/menuler/vault/<uzanti>/<rol>/<sehir>/   # TAM ve ONAYLI
```

## Karar matrisi (özet)

| Durum | Sebep kodu | Hedef | DB yazımı |
|-------|-----------|-------|-----------|
| Geçici ağ/API hatası | — | `bekleyen` (kalır) | Yok |
| Kalıcı ayrıştırma hatası | PARSE_ERROR | `hatali` | Yok |
| 0 gün çıkarıldı | `NO_DATES` | `hatali` | Yok |
| Tarih sırası çelişkisi | `AMBIGUOUS_DATE_ORDER` | `_karantina` | Yok |
| Zayıf sıra kanıtı (tek tarih/tek biçim) | `WEAK_DATE_ORDER` | `_karantina` | Yok |
| LLM ISO ↔ ham tarih uyuşmazlığı | `DATE_ORDER_MISMATCH` | `_karantina` | Yok |
| Birden fazla aya ait tarih | `MULTI_MONTH_SCOPE` | `_karantina` | Yok |
| Dosya adı ile veri ayı uyuşmuyor | `DECLARED_MONTH_MISMATCH` | `_karantina` | Yok |
| Katı modda beyan edilmemiş ay | `UNDECLARED_MONTH` | `_karantina` | Yok |
| Ay eksik kapsanıyor (oran >= eşik) | `LOW_COVERAGE` (Partial) | `_karantina` | Yok |
| Ay eksik kapsanıyor (oran < eşik) | `LOW_COVERAGE` (Suspect) | `_karantina` | Yok |
| Dosya `bekleyen/` kökünde | `NO_CITY` | `_karantina` | Yok |
| Bilinmeyen şehir klasörü | `UNKNOWN_CITY` | `_karantina` | Yok |
| Tam ve tutarlı | — | `vault` | Var + ay atomikliği |

## Telegram komutları

Bot yalnızca `TELEGRAM_ADMIN_CHAT_ID` ile doğrulanan yöneticiye yanıt verir.

| Komut | İşlev |
|-------|-------|
| `/karantina` | Kuyruğu listeler: id, şehir, dosya, sebep, **bekleme yaşı** |
| `/karantina detay <id>` | Meta dosyanın tamamı: gün/beklenen gün, şüpheli tarihler, teşhis |
| `/onayla <id>` | Dosyayı **kapsam içi** tarihlerle işler. Kapsam dışı tarihler yazılmaz ve raporlanır |
| `/reddet <id>` | Dosyayı `hatali/` altına taşır, meta'ya `rejected` işler |
| `/ata <id> <sehir>` | Şehirsiz öğeye şehir atar, dosyayı `bekleyen/<sehir>/` altına alır ve işler |

**Önemli:** `/onayla` yalnızca kapsam ayına (meta dosyadaki `scope_month`) düşen
tarihleri yazar. Ay dışı tarihler (ör. Haziran dosyasındaki hayali 4 Mayıs) hiçbir
koşulda veritabanına girmez. Onay sonrası ay atomikliği çalışır: aynı kaynak ve ay
kapsamında yeni dosyada olmayan eski `kepce-%` kayıtları sert DELETE ile silinir.

## Onay akışı ve sağlayıcı kesintileri (LLM'e bağımlılık YOK)

Karantinaya alınırken **karar anındaki çıkarımın anlık görüntüsü** yan dosyaya
(`<dosya>.karantina.json` → `parsed_days`) gömülür. `/onayla` bu görüntüyü esas alır:

1. Yan dosyada görüntü varsa ve dosyanın `sha256` özeti hâlâ eşleşiyorsa,
   **hiçbir ağ/LLM çağrısı yapılmadan** karar uygulanır (yanıtta `Kaynak:
   karantina anındaki çıkarım (önbellek, LLM'siz)` yazar).
2. Görüntü yoksa (ör. bu sürümden önce kuyruğa girmiş kayıtlar) veya dosya
   değişmişse yeniden ayrıştırılır: `xlsx`/`json` deterministik yolla, `pdf`/görsel
   ise LLM ile.

Bu sayede sağlayıcı 503/429 verdiğinde karantina kuyruğu kilitlenmez; dosya
operatör kararıyla çıkarılabilir. Onay sırasında yeniden ayrıştırma gerekirse ve
sağlayıcı geçici hata verirse Telegram yanıtı "öğe karantinada KALDI, birazdan
tekrar deneyin" der; kalıcı hatada `/reddet` önerilir.

LLM zinciri kesintilere karşı dayanıklıdır: sağlayıcı sırası (`LLM_PROVIDER_ORDER`,
varsayılan `openrouter,gemini`) tümüyle `LLM_CHAIN_PASSES` kez, artan bekleme ile
yeniden denenir; 429/5xx hatalarında model atlanır. Aşırı büyük çıkarımlar (>`WORKER_QUARANTINE_SNAPSHOT_MAX_KB`)
yan dosyaya gömülmez, uyarı loglanır.

## Bildirim ve yükseltme (escalation)

| Zaman | Davranış |
|-------|----------|
| Tespit anında | 🟠 anlık bildirim (sebep + teşhis + karar komutları) |
| Her 24 saatte | 🟠 özet hatırlatma (kuyruk boş değilse) |
| En eski öğe 7 günü geçince | 🔴 kırmızı alarm: "bu kuyruk 7 gündür temizlenmedi" |
| TTL (14 gün) dolunca | Dosya `hatali/`'ya taşınır, 🔴 son alarm gönderilir |

Uyarı kanalı (`TELEGRAM_ADMIN_CHAT_ID` veya `ALERT_WEBHOOK_URL`) tanımlı değilse
worker her döngüde `ERROR` seviyesinde log basar. **Operatörsüz karantina sessiz
kaybın yeni adıdır**; kanal mutlaka tanımlı olmalıdır.

## Ortam değişkenleri

| Değişken | Varsayılan | İş |
|----------|-----------|-----|
| `WORKER_INGEST_STRICT` | `1` | `0` = kapı kapalı, eski davranış (yaz + uyar). Geri alma anahtarı |
| `WORKER_MIN_DAYS_RATIO` | `0.6` | Kısmi/şüpheli ayrım eşiği |
| `WORKER_ALLOW_MISSING_DAYS` | `0` | Tam aydan düşülebilecek gün toleransı (bayram/tatil) |
| `WORKER_REQUIRE_DECLARED_MONTH` | `0` | `1` = dosya adı ay beyan etmiyorsa şüpheli say |
| `WORKER_QUARANTINE_TTL_DAYS` | `14` | Karantina TTL'i |
| `WORKER_QUARANTINE_ESCALATE_DAYS` | `7` | Kırmızı alarm eşiği |
| `WORKER_QUARANTINE_SNAPSHOT_MAX_KB` | `512` | Yan dosyaya gömülecek çıkarım görüntüsü üst sınırı |
| `LLM_PROVIDER_ORDER` | `openrouter,gemini` | Sağlayıcı sırası (birincil OpenRouter) |
| `LLM_CHAIN_PASSES` | `2` | Tüm zincirin yeniden denenme sayısı (1-5) |
| `LLM_CHAIN_RETRY_BASE_MS` | `2000` | Tur arası taban bekleme (ms) |
| `LLM_MODEL_SWITCH_DELAY_MS` | `1500` | Yoğunluk hatasında model değiştirmeden önceki bekleme (ms) |

## Rutin kontroller

```bash
# Karantina kuyruğu (konteyner içinden)
docker exec kepce-worker find /app/data/menuler -path '*_karantina*' -type f

# Karar bekleyen öğe sayısı ve en eski öğe
docker exec kepce-worker sh -c 'ls -lt /app/data/menuler/*/_karantina/ 2>/dev/null'

# Worker loglarında karantina hareketleri
docker logs kepce-worker --since 24h | grep KARANTİNA

# Karar önizlemesi (DB ve LLM olmadan, dosya taşımaz)
cargo run -p worker --example audit_bekleyen -- data/menuler/admin/bekleyen
```

## Geçmiş hayalet kayıtların temizliği (tek seferlik)

Kısmi dosyaların geçmişte sıçrattığı ay dışı kayıtlar için kaynak+ay kapsamlı
temizlik aracı:

```bash
# 1. VERİTABANI YEDEĞİ AL (zorunlu)
# 2. Dry-run ile silinecekleri incele
cargo run -p worker --example month_atomic_cleanup -- istanbul 2026-06 <dosya.xlsx>
# 3. Onaylıyorsan uygula
cargo run -p worker --example month_atomic_cleanup -- istanbul 2026-06 <dosya.xlsx> --apply
```

Araç yalnızca `kepce-%` kaynaklı kayıtlara dokunur (kykyemek ve diğer bağımsız
kaynaklar korunur) ve yalnızca verilen ay penceresinde çalışır. Silme serttir
(`ON DELETE CASCADE` ile bağlı yorum/oy/rapor verileri de silinir); `Rejected`
durumu bilinçli olarak kullanılmaz çünkü aynı kaynağın gelecekteki doğru
yazımını engeller.
