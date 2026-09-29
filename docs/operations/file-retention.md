# Dosya Saklama (Retention) Politikası

Karantina havuzu ve reddedilen menü arşivi için saklama süreleri, temizlik görevi
ve doğrulama komutları.

## Neden gerekli

Google Cloud OLM (nesne yaşam döngüsü) ve KVKK veri minimizasyonu ilkeleri gereği
ihtiyaç duyulmayan kullanıcı içerikleri süresiz saklanamaz. Buna karşılık hatalı
moderasyon kararlarının geri alınabilmesi için anında silme (hard delete) yerine
süreli bir bekleme havuzu tutulur.

## Dosya yaşam döngüsü

```
karantina (uploads/quarantine/menus/<sehir>/<yil>/<uuid>/, TTL: 30 gün)
    ├→ onay → bekleyen (data/menuler/<anonim|admin>/bekleyen/<sehir>/)
    │         → worker işler → vault (data/menuler/vault/<uzanti>/<rol>/<sehir>/<zaman_damgasi>_<dosya>)
    └→ red  → reddedilen (data/menuler/reddedilen/<sehir>/<uuid>/, TTL: 90 gün)
```

### Yerel ingest karar kapısı (29 Eylül 2026)

`bekleyen/` drop-zone akışında dosya artık doğrudan `vault`'a gitmez; önce
**karar motorundan** geçer. Şüpheli veya kısmi çıkarım veritabanına YAZILMAZ,
`_karantina/` kuyruğuna düşer ve operatör kararı bekler:

```
bekleyen/<sehir>/<dosya>
    ├→ tam ve tutarlı     → vault (DB'ye yazılır + ay atomikliği)
    ├→ kısmi/şüpheli      → _karantina/<dosya> + <dosya>.karantina.json (TTL: 14 gün)
    │                        ├→ /onayla → kapsam içi tarihlerle vault
    │                        ├→ /reddet → hatali/
    │                        └→ TTL dolar → hatali/ + 🔴 alarm
    └→ kalıcı hata / 0 gün → hatali/
```

Karantina kuyruğunun operatör akışı, karar matrisi ve ortam değişkenleri için
bkz. [`karantina-runbook.md`](karantina-runbook.md). Kuyruk TTL taraması her
`process_local_files` döngüsünde çalışır (`WORKER_QUARANTINE_TTL_DAYS`,
varsayılan 14 gün; kırmızı alarm eşiği `WORKER_QUARANTINE_ESCALATE_DAYS`,
varsayılan 7 gün).

## Süreler ve yapılandırma

| Değişken | Varsayılan | Kapsam |
|----------|-----------|--------|
| `RETENTION_QUARANTINE_DAYS` | 30 | Karantina havuzu (`MENU_QUARANTINE_DIR`) |
| `RETENTION_REJECTED_DAYS` | 90 (çeyrek dönem) | `reddedilen/` arşivi (`MENU_INGEST_DIR/reddedilen`) |
| `MENU_QUARANTINE_DIR` | `/app/uploads/quarantine/menus` | Karantina kökü |
| `MENU_INGEST_DIR` | `/app/data/menuler` | Onaylanan dosyaların hedef kökü |
| `WORKER_QUARANTINE_TTL_DAYS` | 14 | İngest karar kuyruğu (`_karantina/`) TTL'i |
| `WORKER_QUARANTINE_ESCALATE_DAYS` | 7 | Kuyrukta kırmızı alarm eşiği |

Süre ölçütü dosyanın `mtime` değeridir. Boş dizinler de yaş ölçütüyle temizlenir,
sembolik bağlar dizin dışına sızmayı önlemek için izlenmez.

## Temizlik görevi

[`worker/src/tasks/cleanup.rs`](../../worker/src/tasks/cleanup.rs) içindeki
`clean_expired_files(dry_run)` fonksiyonu periyodik olarak çalışır ve
`[RETENTION-CLEANUP]` önekiyle özet loglar.

```bash
# Temizlik özeti
docker logs kepce-worker --since 24h | grep RETENTION-CLEANUP

# Karantina ve reddedilen arşivdeki dosya sayısı
docker exec kepce-worker find /app/uploads -type f | wc -l
docker exec kepce-worker find /app/data/menuler/reddedilen -type f 2>/dev/null | wc -l
```

**Dikkat:** Karantina bir Docker volume içindedir (`kepce_uploads`, konteyner içi
yolu `/app/uploads`). Sunucu host'unda `uploads/` yolu yoktur, bu yüzden sayım
host'tan değil konteyner içinden yapılmalıdır.

## Doğrulama kaydı

- 28 Eylül 2026: temizlikçi dry-run ve gerçek modda çalıştı. TTL dolmadığı için
  silme yapmadı (`Karantina: 0 dosya`), tasarım gereği.
- 29 Eylül 2026: karantinada 3 dosya mevcut (Çorum 2 JPEG, Bursa 1 PDF), mtime
  20 Eylül 2026. 30 günlük TTL 20 Ekim 2026'da dolacak. Bu dosyalar silinmiş
  gönderimlere ait yetim dosyalardır, veritabanında karşılıkları yoktur.
