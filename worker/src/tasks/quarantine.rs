//! Karantina kuyruğu: karar bekleyen yerel ingest dosyaları.
//!
//! Tasarım, endüstrideki **dead-letter queue (DLQ)** ve **quarantine-then-classify**
//! kalıplarının dosya sistemine uyarlanmış halidir:
//! - Her şüpheli dosya önce `_karantina/` altına düşer, veritabanına HİÇBİR ŞEY
//!   yazılmaz. Karar (onayla/reddet) operatör tarafından Telegram üzerinden verilir.
//! - Kuyruk iki metrikle izlenir: bekleyen öğe sayısı ve EN ESKİ öğenin yaşı.
//! - Otomatik yeniden deneme YOKTUR; sonlu bir TTL ve gürültülü sonuç vardır.
//!   TTL dolduğunda dosya `hatali/` altına taşınır ve kırmızı alarm üretilir.
//!
//! Klasör sözleşmesi:
//! ```text
//! data/menuler/<rol>/_karantina/<dosya>
//! data/menuler/<rol>/_karantina/<dosya>.karantina.json   (karar meta verisi)
//! ```

use crate::parser::models::MenuDatabase;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Karantina klasörü adı (rol dizini altında).
pub const QUARANTINE_DIR: &str = "_karantina";
/// Karar meta verisi yan dosyasının uzantısı.
pub const SIDECAR_SUFFIX: &str = ".karantina.json";
/// Yerel ingest rol klasörleri.
pub const ROLES: [&str; 3] = ["admin", "kullanici", "anonim"];
/// Günlük özet hatırlatma durum dosyası (base dir altında).
const REMINDER_STATE_FILE: &str = ".karantina_hatirlatma.json";

/// Karantina sebebi kodları (karar matrisi, bkz. plan 4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReasonCode {
    /// Hiç tarih çıkarılamadı (kalıcı hata, `hatali/`).
    NoDates,
    /// Dosya `bekleyen/` kökünde, şehir klasörü yok.
    NoCity,
    /// Şehir klasörü adı `cities` tablosunda yok.
    UnknownCity,
    /// Dosya birden fazla aya ait tarih içeriyor.
    MultiMonthScope,
    /// Bildirilen ay ile veriden çıkan kapsam uyuşmuyor.
    DeclaredMonthMismatch,
    /// Katı mod: dosya adı/sayfa adı ay beyan etmiyor.
    UndeclaredMonth,
    /// Ay eksik kapsanıyor (kısmi veya şüpheli).
    LowCoverage,
    /// Tarih sırası çelişkisi (T1/T2/T4).
    AmbiguousDateOrder,
    /// Tarih sırası kanıtı zayıf (T3).
    WeakDateOrder,
    /// LLM ISO tarihi ile ham tarih (`date_raw`) uyuşmuyor.
    DateOrderMismatch,
    /// Sütun başlığı gün adı ile tarihin haftanın günü uyuşmuyor.
    WeekdayMismatch,
    /// Karantina TTL'i doldu.
    TtlExpired,
    /// Resmi fiyat ve gramaj cetveli (tarihsiz onay belgesi).
    OfficialPricingDocument,
    /// Al Götür menü paketi / slot listesi.
    TakeawayDocument,
}

impl ReasonCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReasonCode::NoDates => "NO_DATES",
            ReasonCode::NoCity => "NO_CITY",
            ReasonCode::UnknownCity => "UNKNOWN_CITY",
            ReasonCode::MultiMonthScope => "MULTI_MONTH_SCOPE",
            ReasonCode::DeclaredMonthMismatch => "DECLARED_MONTH_MISMATCH",
            ReasonCode::UndeclaredMonth => "UNDECLARED_MONTH",
            ReasonCode::LowCoverage => "LOW_COVERAGE",
            ReasonCode::AmbiguousDateOrder => "AMBIGUOUS_DATE_ORDER",
            ReasonCode::WeakDateOrder => "WEAK_DATE_ORDER",
            ReasonCode::DateOrderMismatch => "DATE_ORDER_MISMATCH",
            ReasonCode::WeekdayMismatch => "WEEKDAY_MISMATCH",
            ReasonCode::TtlExpired => "TTL_EXPIRED",
            ReasonCode::OfficialPricingDocument => "OFFICIAL_PRICING_DOCUMENT",
            ReasonCode::TakeawayDocument => "TAKEAWAY_DOCUMENT",
        }
    }
}

/// Karar meta verisi (`<dosya>.karantina.json`). Operatörün Telegram mesajını
/// besler; teşhis cümlenin içindedir, operatörün Excel'i açması gerekmez.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineMeta {
    pub id: String,
    pub file: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    pub reason_code: ReasonCode,
    pub reason_tr: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub detected_months: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_month: Option<String>,
    #[serde(default)]
    pub day_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_days: Option<u32>,
    #[serde(default)]
    pub has_colyak: bool,
    #[serde(default)]
    pub colyak_day_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stray_dates: Vec<String>,
    /// Ek teşhis ayrıntıları (çelişen tarihler, çözülemeyen hücreler vb.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<String>,
    #[serde(default)]
    pub sha256: String,
    /// Karantinaya alınırken yapılmış ayrıştırmanın anlık görüntüsü.
    ///
    /// `/onayla` bu görüntüyü esas alır: sağlayıcı (LLM/ağ) erişilemez olduğunda
    /// bile operatör kararı uygulanabilir ve dosya karantinada kilitli kalmaz.
    /// Onay öncesi `sha256` ile dosyanın değişmediği doğrulanır.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parsed_days: Option<MenuDatabase>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parsed_pricing: Option<crate::parser::models::OfficialPricingData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parsed_takeaway: Option<crate::parser::models::TakeawayData>,
    pub first_seen_at: String,
    #[serde(default)]
    pub notify_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_notified_at: Option<String>,
    /// Öğenin nasıl sonuçlandığı: approved | rejected | ttl_expired | assigned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}

/// Kuyruğa düşerken doldurulan teşhis alanları.
#[derive(Debug, Default, Clone)]
pub struct QuarantineDetail {
    pub detected_months: Vec<String>,
    pub scope_month: Option<String>,
    pub day_count: usize,
    pub expected_days: Option<u32>,
    pub has_colyak: bool,
    pub colyak_day_count: usize,
    pub stray_dates: Vec<String>,
    pub details: Vec<String>,
    /// Karar anındaki çıkarım. Onay sırasında yeniden ayrıştırmayı (ve dolayısıyla
    /// LLM bağımlılığını) ortadan kaldırır.
    pub parsed_days: Option<MenuDatabase>,
    pub parsed_pricing: Option<crate::parser::models::OfficialPricingData>,
    pub parsed_takeaway: Option<crate::parser::models::TakeawayData>,
}

/// Kuyruktaki bir öğe: meta + dosya ve yan dosya yolları.
#[derive(Debug, Clone)]
pub struct QueueItem {
    pub meta: QuarantineMeta,
    pub file_path: PathBuf,
    pub meta_path: PathBuf,
}

impl QueueItem {
    /// Öğenin bekleme yaşı (gün). `first_seen_at` okunamazsa dosya mtime'ı.
    pub async fn age_days(&self) -> i64 {
        let now = Utc::now();
        if let Ok(first) = DateTime::parse_from_rfc3339(&self.meta.first_seen_at) {
            return (now - first.with_timezone(&Utc)).num_days();
        }
        tokio::fs::metadata(&self.file_path)
            .await
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|mtime| {
                let mt: DateTime<Utc> = mtime.into();
                (now - mt).num_days()
            })
            .unwrap_or(0)
    }
}

/// Yan dosyaya gömülecek çıkarım görüntüsünün üst sınırı (KB). Varsayılan 512.
pub fn snapshot_max_kb() -> usize {
    std::env::var("WORKER_QUARANTINE_SNAPSHOT_MAX_KB")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .filter(|v: &usize| *v > 0)
        .unwrap_or(512)
}

/// Karantina TTL'i (gün). Varsayılan 14 (K-6).
pub fn ttl_days() -> u64 {
    std::env::var("WORKER_QUARANTINE_TTL_DAYS")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .filter(|d: &u64| *d > 0)
        .unwrap_or(14)
}

/// Kırmızı alarm eşiği (gün). Varsayılan 7.
pub fn escalate_days() -> u64 {
    std::env::var("WORKER_QUARANTINE_ESCALATE_DAYS")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .filter(|d: &u64| *d > 0)
        .unwrap_or(7)
}

/// Yerel ingest kök dizini (`file_ingest` ile aynı çözüm).
pub fn menu_base_dir() -> String {
    std::env::var("WORKER_MENU_DIR").unwrap_or_else(|_| "../data/menuler".to_string())
}

pub fn quarantine_root(base_dir: &str, role: &str) -> PathBuf {
    PathBuf::from(base_dir).join(role).join(QUARANTINE_DIR)
}

use std::fmt::Write;

/// Dosyanın sha256 özeti (karantina yan dosyasındaki kaydı doğrulamak için).
pub async fn sha256_of(path: &Path) -> Result<String> {
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|e| anyhow::anyhow!("dosya okunamadı {:?}: {}", path, e))?;
    let digest = Sha256::digest(&bytes);
    Ok(digest.iter().fold(String::with_capacity(64), |mut s, b| {
        let _ = write!(s, "{:02x}", b);
        s
    }))
}

/// Dosyayı karantinaya taşır, yan meta dosyasını yazar ve operatöre bildirir.
///
/// Taşıma rename -> copy+remove zinciriyle yapılır; ikisi de başarısızsa dosya
/// yerinde bırakılır ve hata döner (sessiz kayıp yok).
pub async fn quarantine_file(
    base_dir: &str,
    role: &str,
    city: Option<&str>,
    src: &Path,
    reason: ReasonCode,
    reason_tr: String,
    detail: QuarantineDetail,
) -> Result<QuarantineMeta> {
    let file_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("karantina: dosya adı çözülemedi: {:?}", src))?
        .to_string();

    let q_root = quarantine_root(base_dir, role);
    tokio::fs::create_dir_all(&q_root).await?;

    let sha = sha256_of(src).await?;
    let dest = move_to_unique_dest(src, &q_root, &file_name).await?;

    // Yan dosya küçük bir karar kaydıdır: aşırı büyük çıkarımlar gömülmez
    // (listeleme/sweep maliyeti ve disk şişmesi engellenir).
    let parsed_days = detail.parsed_days.filter(|snapshot| {
        let limit = snapshot_max_kb() * 1024;
        match serde_json::to_string(snapshot) {
            Ok(json) if json.len() <= limit => true,
            Ok(json) => {
                tracing::warn!(
                    "Karantina: çıkarım görüntüsü çok büyük ({} KB > {} KB), yan dosyaya gömülmedi: {}",
                    json.len() / 1024,
                    snapshot_max_kb(),
                    file_name
                );
                false
            }
            Err(e) => {
                tracing::warn!(
                    "Karantina: çıkarım görüntüsü serileştirilemedi ({}): {}",
                    file_name,
                    e
                );
                false
            }
        }
    });

    let id = format!(
        "k_{}",
        uuid::Uuid::new_v4().simple().to_string()[..12].to_uppercase()
    );
    let now = Utc::now().to_rfc3339();
    let meta = QuarantineMeta {
        id: id.clone(),
        file: dest
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&file_name)
            .to_string(),
        role: role.to_string(),
        city: city.map(|s| s.to_string()),
        reason_code: reason,
        reason_tr,
        detected_months: detail.detected_months,
        scope_month: detail.scope_month,
        day_count: detail.day_count,
        expected_days: detail.expected_days,
        has_colyak: detail.has_colyak,
        colyak_day_count: detail.colyak_day_count,
        stray_dates: detail.stray_dates,
        details: detail.details,
        sha256: sha,
        parsed_days,
        parsed_pricing: detail.parsed_pricing,
        parsed_takeaway: detail.parsed_takeaway,
        first_seen_at: now.clone(),
        notify_count: 0,
        last_notified_at: None,
        resolution: None,
    };

    let meta_path = sidecar_path(&dest);
    tokio::fs::write(&meta_path, serde_json::to_string_pretty(&meta)?).await?;

    let mut item = QueueItem {
        meta: meta.clone(),
        file_path: dest,
        meta_path,
    };
    notify_item(&mut item, NotifyKind::New).await;

    Ok(item.meta)
}

/// Yan meta dosyasının yolu (`<dosya>.karantina.json`).
pub fn sidecar_path(file: &Path) -> PathBuf {
    let mut s = file.as_os_str().to_os_string();
    s.push(SIDECAR_SUFFIX);
    PathBuf::from(s)
}

/// Çakışmasız ve atomik taşıma (TOCTOU yarış durumunu önler).
pub async fn move_to_unique_dest(src: &Path, dir: &Path, file_name: &str) -> Result<PathBuf> {
    tokio::fs::create_dir_all(dir).await?;
    let (stem, ext) = match file_name.rsplit_once('.') {
        Some((s, e)) => (s.to_string(), format!(".{}", e)),
        None => (file_name.to_string(), String::new()),
    };

    let first = dir.join(file_name);
    if !first.exists() && move_file(src, &first).await.is_ok() {
        return Ok(first);
    }

    for i in 1..1000 {
        let candidate = dir.join(format!("{}_{}{}", stem, i, ext));
        if !candidate.exists() && move_file(src, &candidate).await.is_ok() {
            return Ok(candidate);
        }
    }

    let fallback = dir.join(format!(
        "{}_{}_{}{}",
        stem,
        Utc::now().timestamp_micros(),
        uuid::Uuid::new_v4().simple(),
        ext
    ));
    move_file(src, &fallback).await?;
    Ok(fallback)
}

/// Rename -> copy+remove zinciriyle dosya taşıma (mevcut ingest davranışıyla aynı).
pub async fn move_file(src: &Path, dest: &Path) -> Result<()> {
    if let Err(e) = tokio::fs::rename(src, dest).await {
        tracing::warn!("Dosya taşınamadı ({:?}). Kopyalama + silme deneniyor...", e);
        tokio::fs::copy(src, dest).await?;
        tokio::fs::remove_file(src).await?;
    }
    Ok(())
}

/// Tüm rollerin karantina kuyruğunu listeler (yaşa göre sıralı, en eski önce).
pub async fn list_queue(base_dir: &str) -> Vec<QueueItem> {
    let mut items = Vec::new();
    for role in ROLES {
        let q_root = quarantine_root(base_dir, role);
        let Ok(mut rd) = tokio::fs::read_dir(&q_root).await else {
            continue;
        };
        while let Ok(Some(entry)) = rd.next_entry().await {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if !name.ends_with(SIDECAR_SUFFIX) {
                continue;
            }
            let file_path = PathBuf::from(&name[..name.len() - SIDECAR_SUFFIX.len()]);
            let file_path = path.parent().unwrap_or(Path::new(".")).join(&file_path);
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => match serde_json::from_str::<QuarantineMeta>(&content) {
                    Ok(meta) => items.push(QueueItem {
                        meta,
                        file_path,
                        meta_path: path,
                    }),
                    Err(e) => tracing::error!(
                        "Karantina meta dosyası okunamadı ({:?}): {} — ÖKSÜZ KAYIT, elle inceleyin",
                        path,
                        e
                    ),
                },
                Err(e) => tracing::error!("Karantina meta dosyası okunamadı ({:?}): {}", path, e),
            }
        }
    }
    items.sort_by(|a, b| a.meta.first_seen_at.cmp(&b.meta.first_seen_at));
    items
}

/// ID (tam veya önek) ile kuyruk öğesi bulur.
pub async fn find_item(base_dir: &str, id: &str) -> Option<QueueItem> {
    let clean_query = id
        .trim()
        .trim_start_matches("k_")
        .trim_start_matches("K_")
        .to_uppercase();
    let id_upper = id.trim().to_uppercase();
    let items = list_queue(base_dir).await;
    items.into_iter().find(|i| {
        let clean_item = i
            .meta
            .id
            .trim_start_matches("k_")
            .trim_start_matches("K_")
            .to_uppercase();
        i.meta.id.eq_ignore_ascii_case(&id_upper)
            || i.meta.id.to_uppercase().starts_with(&id_upper)
            || clean_item.starts_with(&clean_query)
    })
}

/// Meta dosyasını yeniden yazar (notify_count/resolution güncellemeleri).
pub async fn write_meta(item: &QueueItem) -> Result<()> {
    tokio::fs::write(&item.meta_path, serde_json::to_string_pretty(&item.meta)?).await?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotifyKind {
    New,
    Reminder,
    Escalation,
    TtlExpired,
}

/// Karantina öğesi için standart inline buton klavyesini üretir.
pub fn item_inline_keyboard(id: &str) -> serde_json::Value {
    serde_json::json!({
        "inline_keyboard": [
            [
                { "text": "Onayla", "callback_data": format!("q:approve:{}", id) },
                { "text": "Reddet", "callback_data": format!("q:reject_confirm:{}", id) }
            ],
            [
                { "text": "Detay", "callback_data": format!("q:detail:{}", id) },
                { "text": "Dosyayı Gönder", "callback_data": format!("q:file:{}", id) }
            ]
        ]
    })
}

/// Bildirim mesajını plan 5.1 biçiminde üretir.
pub fn format_item_alert(item: &QueueItem, age_days: i64, kind: NotifyKind) -> String {
    let ttl = ttl_days();
    let head = match kind {
        NotifyKind::New => format!("KARANTİNA: {}", item.meta.id),
        NotifyKind::Reminder => format!("KARANTİNA HATIRLATMA: {}", item.meta.id),
        NotifyKind::Escalation => format!(
            "KARANTİNA KRİTİK: {} (bu öğe {} gündür bekliyor, {} gün eşiği aşıldı)",
            item.meta.id,
            age_days,
            escalate_days()
        ),
        NotifyKind::TtlExpired => format!(
            "KARANTİNA TTL DOLDU: {} ({} gün bekledi, dosya hatali/ altına taşındı)",
            item.meta.id, age_days
        ),
    };
    let city = item.meta.city.as_deref().unwrap_or("şehirsiz");
    let mut lines = vec![
        head,
        format!("{} · {}", city, item.meta.file),
        format!("Sebep: {}", item.meta.reason_code.as_str()),
        item.meta.reason_tr.clone(),
    ];
    if !item.meta.detected_months.is_empty() {
        lines.push(format!(
            "Tespit edilen aylar: {}",
            item.meta.detected_months.join(", ")
        ));
    }
    if !item.meta.stray_dates.is_empty() {
        lines.push(format!(
            "Şüpheli tarihler: {}",
            item.meta.stray_dates.join(", ")
        ));
    }
    if item.meta.day_count > 0 || item.meta.expected_days.is_some() {
        lines.push(format!(
            "Gün: {} / beklenen {}",
            item.meta.day_count,
            item.meta
                .expected_days
                .map(|d| d.to_string())
                .unwrap_or_else(|| "?".to_string())
        ));
    }
    if item.meta.has_colyak || item.meta.colyak_day_count > 0 {
        lines.push(format!(
            "Çölyak Menüsü: Var ({} gün)",
            item.meta.colyak_day_count
        ));
    }
    if let Some(ref pricing) = item.meta.parsed_pricing {
        let kahvalti = pricing
            .items
            .iter()
            .filter(|i| i.meal_type == "breakfast")
            .count();
        let yemek = pricing
            .items
            .iter()
            .filter(|i| i.meal_type != "breakfast")
            .count();
        lines.push(format!(
            "Fiyat Listesi: {} Kahvaltı, {} Yemek/Öğle/Akşam kalemi",
            kahvalti, yemek
        ));
        if let Some(ref year) = pricing.academic_year {
            lines.push(format!("Akademik Dönem: {}", year));
        }
    }
    if let Some(ref takeaway) = item.meta.parsed_takeaway {
        lines.push(format!(
            "Al Götür: {} paket tespit edildi",
            takeaway.packages.len()
        ));
        for pkg in &takeaway.packages {
            lines.push(format!(
                "• {}: {} seçim slotu",
                pkg.package_name,
                pkg.slots.len()
            ));
        }
    }
    for d in &item.meta.details {
        lines.push(format!("• {}", d));
    }
    if kind != NotifyKind::TtlExpired {
        lines.push(format!(
            "Karar: /onayla {}  veya  /reddet {}{}",
            item.meta.id,
            item.meta.id,
            if item.meta.city.is_none() {
                format!("  (şehirsiz öğe için önce /ata {} <sehir>)", item.meta.id)
            } else {
                String::new()
            }
        ));
        lines.push(format!(
            "(TTL: {} gün · bu öğe {} gündür bekliyor)",
            ttl, age_days
        ));
    }
    lines.join("\n")
}

/// Karantina bildirimini gönderir ve sayaçları meta dosyasına işler.
/// Uyarı kanalı tanımlı değilse ERROR loglar (plan 5.4: operatörsüz karantina
/// sessiz kaybın yeni adı olur).
pub async fn notify_item(item: &mut QueueItem, kind: NotifyKind) {
    let age = item.age_days().await;
    let message = format_item_alert(item, age, kind);

    if !shared::services::alerting::AlertingService::alert_channel_configured() {
        tracing::error!(
            "[KARANTİNA] Uyarı kanalı TANIMLI DEĞİL (TELEGRAM_ADMIN_CHAT_ID / ALERT_WEBHOOK_URL yok). \
             Karantinaya düşen öğe operatöre ULAŞMAYACAK: {} ({}) — {}",
            item.meta.id,
            item.meta.reason_code.as_str(),
            item.meta.file
        );
    }

    let kb = if kind != NotifyKind::TtlExpired {
        item_inline_keyboard(&item.meta.id)
    } else {
        serde_json::json!({})
    };

    if item.file_path.exists() {
        if let Err(e) =
            shared::services::alerting::AlertingService::send_telegram_document_with_buttons(
                &message,
                &item.file_path,
                kb.clone(),
            )
            .await
        {
            tracing::error!(
                "[KARANTİNA] Dosya ekli bildirim gönderilemedi ({}): {:?}",
                item.meta.id,
                e
            );
            let _ =
                shared::services::alerting::AlertingService::send_telegram_message_with_buttons(
                    &message, kb,
                )
                .await;
        }
    } else if let Err(e) =
        shared::services::alerting::AlertingService::send_telegram_message_with_buttons(
            &message, kb,
        )
        .await
    {
        tracing::error!(
            "[KARANTİNA] Bildirim gönderilemedi ({}): {:?}",
            item.meta.id,
            e
        );
    }

    item.meta.notify_count += 1;
    item.meta.last_notified_at = Some(Utc::now().to_rfc3339());
    if let Err(e) = write_meta(item).await {
        tracing::warn!(
            "[KARANTİNA] meta güncellenemedi ({}): {:?}",
            item.meta.id,
            e
        );
    }
}

/// TTL ve yükseltme (escalation) taraması: her ingest döngüsünde çalışır.
///
/// - Yaş >= TTL: dosya `hatali/` altına taşınır, 🔴 son alarm gönderilir.
/// - Yaş >= eşik (7 gün): 🔴 kırmızı alarm (24 saatte bir yinelenir).
/// - Kuyruk boş değilse günde bir özet hatırlatma gönderilir.
#[derive(Debug, Default)]
pub struct SweepReport {
    pub expired: Vec<String>,
    pub escalated: Vec<String>,
    pub summary_sent: bool,
}

pub async fn sweep(base_dir: &str) -> Result<SweepReport> {
    let mut report = SweepReport::default();
    let mut items = list_queue(base_dir).await;
    if items.is_empty() {
        return Ok(report);
    }

    let ttl = ttl_days() as i64;
    let escalate = escalate_days() as i64;

    let mut remaining: Vec<&QueueItem> = Vec::new();
    for item in &mut items {
        let age = item.age_days().await;

        if age >= ttl {
            // TTL doldu: hatali/ altına taşı, sonucu meta'ya işle, kırmızı alarm.
            let err_dir = PathBuf::from(base_dir).join(&item.meta.role).join("hatali");
            let dest = match move_to_unique_dest(&item.file_path, &err_dir, &item.meta.file).await {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!(
                        "[KARANTİNA] TTL dolan dosya hatali/ altına taşınamadı ({:?}): {:?}",
                        item.file_path,
                        e
                    );
                    remaining.push(item);
                    continue;
                }
            };
            let mut meta = item.meta.clone();
            meta.resolution = Some("ttl_expired".to_string());
            let new_meta_path = sidecar_path(&dest);
            let _ = tokio::fs::write(&new_meta_path, serde_json::to_string_pretty(&meta)?).await;
            let _ = tokio::fs::remove_file(&item.meta_path).await;

            let mut moved_item = QueueItem {
                meta,
                file_path: dest,
                meta_path: new_meta_path,
            };
            notify_item(&mut moved_item, NotifyKind::TtlExpired).await;
            tracing::error!(
                "[KARANTİNA] TTL ({} gün) doldu: {} -> hatali/ ({} gün bekledi)",
                ttl,
                item.meta.file,
                age
            );
            report.expired.push(item.meta.id.clone());
            continue;
        }

        if age >= escalate {
            // Son 20 saatte bildirim gitmediyse kırmızı alarmı yinele.
            let stale_notify = item
                .meta
                .last_notified_at
                .as_deref()
                .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
                .map(|t| (Utc::now() - t.with_timezone(&Utc)).num_hours() >= 20)
                .unwrap_or(true);
            if stale_notify {
                notify_item(item, NotifyKind::Escalation).await;
                report.escalated.push(item.meta.id.clone());
            }
        }
        remaining.push(item);
    }

    // Günlük özet hatırlatma (yalnızca kuyruk boş değilse).
    if !remaining.is_empty() && should_send_daily_summary(base_dir).await {
        let oldest_age = remaining
            .iter()
            .fold(0i64, |acc, i| acc.max(meta_age_days(i)));
        let critical = oldest_age >= escalate;
        let icon = if critical { "🔴" } else { "🟠" };
        let mut lines = vec![format!(
            "{} KARANTİNA ÖZETİ: {} öğe karar bekliyor, en eskisi {} gündür bekliyor (TTL {} gün).",
            icon,
            remaining.len(),
            oldest_age,
            ttl_days()
        )];
        if critical {
            lines.push(format!(
                "⚠️ Bu kuyruk {} gündür temizlenmedi! /karantina ile listeleyin.",
                oldest_age
            ));
        }
        for i in remaining.iter().take(10) {
            lines.push(format!(
                "• {} {} {} [{}]",
                i.meta.id,
                i.meta.city.as_deref().unwrap_or("şehirsiz"),
                i.meta.file,
                i.meta.reason_code.as_str()
            ));
        }
        if remaining.len() > 10 {
            lines.push(format!("… ve {} öğe daha", remaining.len() - 10));
        }
        let msg = lines.join("\n");
        if let Err(e) = shared::services::alerting::AlertingService::send_webhook_alert(&msg).await
        {
            tracing::error!("[KARANTİNA] Özet hatırlatma gönderilemedi: {:?}", e);
        }
        mark_daily_summary(base_dir).await;
        report.summary_sent = true;
    }

    Ok(report)
}

/// Senkron yaş hesabı (özet ve listeleme için; first_seen_at RFC3339).
fn meta_age_days(item: &QueueItem) -> i64 {
    DateTime::parse_from_rfc3339(&item.meta.first_seen_at)
        .map(|t| (Utc::now() - t.with_timezone(&Utc)).num_days())
        .unwrap_or(0)
}

#[derive(Debug, Serialize, Deserialize)]
struct ReminderState {
    last_summary_at: String,
}

async fn should_send_daily_summary(base_dir: &str) -> bool {
    let path = PathBuf::from(base_dir).join(REMINDER_STATE_FILE);
    let Ok(content) = tokio::fs::read_to_string(&path).await else {
        return true;
    };
    let Ok(state) = serde_json::from_str::<ReminderState>(&content) else {
        return true;
    };
    DateTime::parse_from_rfc3339(&state.last_summary_at)
        .map(|t| (Utc::now() - t.with_timezone(&Utc)).num_hours() >= 20)
        .unwrap_or(true)
}

async fn mark_daily_summary(base_dir: &str) {
    let path = PathBuf::from(base_dir).join(REMINDER_STATE_FILE);
    let state = ReminderState {
        last_summary_at: Utc::now().to_rfc3339(),
    };
    if let Ok(json) = serde_json::to_string(&state) {
        let _ = tokio::fs::write(&path, json).await;
    }
}

/// Telegram `/karantina` liste çıktısı: id, şehir, dosya, sebep, bekleme yaşı.
pub async fn format_queue_listing(base_dir: &str) -> String {
    let items = list_queue(base_dir).await;
    if items.is_empty() {
        return "✅ Karantina kuyruğu boş. Karar bekleyen dosya yok.".to_string();
    }
    let mut lines = vec![format!(
        "🟠 *Karantina Kuyruğu* ({} öğe · TTL {} gün):",
        items.len(),
        ttl_days()
    )];
    for i in &items {
        let age = meta_age_days(i);
        lines.push(format!(
            "• `{}` {} · {} · {} · *{} gündür bekliyor*",
            i.meta.id,
            i.meta.city.as_deref().unwrap_or("şehirsiz"),
            i.meta.file,
            i.meta.reason_code.as_str(),
            age
        ));
    }
    lines.push(
        "\nKomutlar: `/karantina detay <id>`, `/onayla <id>`, `/reddet <id>`, `/ata <id> <sehir>`"
            .to_string(),
    );
    lines.join("\n")
}

/// Telegram `/karantina detay <id>` çıktısı.
pub async fn format_item_detail(item: &QueueItem) -> String {
    let age = item.age_days().await;
    let mut lines = vec![
        format!("[KARANTİNA DETAYI: `{}`]", item.meta.id),
        format!("• Dosya: `{}`", item.meta.file),
        format!("• Rol: `{}`", item.meta.role),
        format!(
            "• Şehir: `{}`",
            item.meta.city.as_deref().unwrap_or("şehirsiz")
        ),
        format!("• Sebep: `{}`", item.meta.reason_code.as_str()),
        format!("• Açıklama: {}", item.meta.reason_tr),
        format!(
            "• Gün: {} / beklenen {}",
            item.meta.day_count,
            item.meta
                .expected_days
                .map(|d| d.to_string())
                .unwrap_or_else(|| "?".to_string())
        ),
    ];
    if item.meta.has_colyak || item.meta.colyak_day_count > 0 {
        lines.push(format!("• Çölyak Menüsü: Var ({} gün)", item.meta.colyak_day_count));
    }
    if !item.meta.detected_months.is_empty() {
        lines.push(format!(
            "• Tespit edilen aylar: `{}`",
            item.meta.detected_months.join(", ")
        ));
    }
    if let Some(scope) = &item.meta.scope_month {
        lines.push(format!("• Onayda yazılacak kapsam: `{}`", scope));
    }
    if !item.meta.stray_dates.is_empty() {
        lines.push(format!(
            "• Kapsam dışı (YAZILMAYACAK) tarihler: `{}`",
            item.meta.stray_dates.join(", ")
        ));
    }
    for d in &item.meta.details {
        lines.push(format!("• {}", d));
    }
    if !item.meta.sha256.is_empty() {
        lines.push(format!(
            "• sha256: `{}…`",
            &item.meta.sha256[..16.min(item.meta.sha256.len())]
        ));
    }
    lines.push(format!(
        "• Yaş: {} gün (TTL {} gün) · bildirim sayısı: {}",
        age,
        ttl_days(),
        item.meta.notify_count
    ));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_meta(id: &str, first_seen: &str) -> QuarantineMeta {
        QuarantineMeta {
            id: id.to_string(),
            file: format!("{}.xlsx", id),
            role: "admin".to_string(),
            city: Some("istanbul".to_string()),
            reason_code: ReasonCode::MultiMonthScope,
            reason_tr: "test".to_string(),
            detected_months: vec!["2026-05".into(), "2026-06".into()],
            scope_month: Some("2026-06".into()),
            day_count: 31,
            expected_days: Some(30),
            has_colyak: false,
            colyak_day_count: 0,
            stray_dates: vec!["2026-05-04".into()],
            details: vec![],
            sha256: "abc".to_string(),
            parsed_days: None,
            parsed_pricing: None,
            parsed_takeaway: None,
            first_seen_at: first_seen.to_string(),
            notify_count: 0,
            last_notified_at: None,
            resolution: None,
        }
    }

    #[test]
    fn test_reason_code_serialization() {
        let json = serde_json::to_string(&ReasonCode::MultiMonthScope).unwrap();
        assert_eq!(json, "\"MULTI_MONTH_SCOPE\"");
        let back: ReasonCode = serde_json::from_str("\"WEAK_DATE_ORDER\"").unwrap();
        assert_eq!(back, ReasonCode::WeakDateOrder);
    }

    /// Anlık görüntü yan dosyaya yazılmalı ve okunabilmeli: `/onayla` bu veriyle
    /// LLM/sağlayıcı erişimi olmadan da ilerleyebilir.
    #[tokio::test]
    async fn test_snapshot_roundtrip() {
        let base = std::env::temp_dir().join(format!("kepce_q_snap_{}", uuid::Uuid::new_v4()));
        let bekleyen = base.join("admin").join("bekleyen").join("istanbul");
        tokio::fs::create_dir_all(&bekleyen).await.unwrap();
        let src = bekleyen.join("Haziran_Snapshot.xlsx");
        tokio::fs::write(&src, b"snapshot icerigi").await.unwrap();

        let mut snapshot = MenuDatabase::new();
        snapshot.insert(
            "2026-06-01".to_string(),
            crate::parser::models::DayData::default(),
        );

        let meta = quarantine_file(
            base.to_str().unwrap(),
            "admin",
            Some("istanbul"),
            &src,
            ReasonCode::LowCoverage,
            "test".to_string(),
            QuarantineDetail {
                parsed_days: Some(snapshot),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert!(
            meta.parsed_days.is_some(),
            "dönen meta anlık görüntüyü taşımalı"
        );

        let items = list_queue(base.to_str().unwrap()).await;
        assert_eq!(items.len(), 1);
        let stored = items[0]
            .meta
            .parsed_days
            .as_ref()
            .expect("anlık görüntü yan dosyada saklanmalı");
        assert!(stored.contains_key("2026-06-01"));

        let _ = tokio::fs::remove_dir_all(&base).await;
    }

    /// Anlık görüntü alanı olmayan eski yan dosyalar okunabilmeye devam etmeli.
    #[test]
    fn test_meta_without_snapshot_is_backward_compatible() {
        let legacy = r#"{
            "id": "k_LEGACY1",
            "file": "Haziran.xlsx",
            "role": "admin",
            "city": "istanbul",
            "reason_code": "LOW_COVERAGE",
            "reason_tr": "eski kayıt",
            "sha256": "deadbeef",
            "first_seen_at": "2026-01-01T00:00:00+00:00"
        }"#;

        let meta: QuarantineMeta =
            serde_json::from_str(legacy).expect("eski kayıt geriye dönük okunabilmeli");
        assert_eq!(meta.id, "k_LEGACY1");
        assert!(meta.parsed_days.is_none());
    }

    #[test]
    fn test_meta_roundtrip() {
        let meta = test_meta("k_TEST1", &Utc::now().to_rfc3339());
        let json = serde_json::to_string_pretty(&meta).unwrap();
        let back: QuarantineMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, meta.id);
        assert_eq!(back.reason_code, ReasonCode::MultiMonthScope);
        assert_eq!(back.stray_dates, vec!["2026-05-04".to_string()]);
    }

    #[tokio::test]
    async fn test_quarantine_file_and_list() {
        let base = std::env::temp_dir().join(format!("kepce_q_test_{}", uuid::Uuid::new_v4()));
        let bekleyen = base.join("admin").join("bekleyen").join("istanbul");
        tokio::fs::create_dir_all(&bekleyen).await.unwrap();
        let src = bekleyen.join("Haziran_Test.xlsx");
        tokio::fs::write(&src, b"test icerigi").await.unwrap();

        let meta = quarantine_file(
            base.to_str().unwrap(),
            "admin",
            Some("istanbul"),
            &src,
            ReasonCode::MultiMonthScope,
            "Dosya birden fazla aya ait tarih içeriyor.".to_string(),
            QuarantineDetail {
                detected_months: vec!["2026-05".into(), "2026-06".into()],
                scope_month: Some("2026-06".into()),
                day_count: 31,
                expected_days: Some(30),
                has_colyak: false,
                colyak_day_count: 0,
                stray_dates: vec!["2026-05-04".into()],
                details: vec![],
                parsed_days: None,
                parsed_pricing: None,
                parsed_takeaway: None,
            },
        )
        .await
        .unwrap();

        assert!(!src.exists(), "kaynak dosya karantinaya taşınmalı");
        let q_file = quarantine_root(base.to_str().unwrap(), "admin").join(&meta.file);
        assert!(q_file.exists(), "dosya karantinada olmalı");
        assert!(sidecar_path(&q_file).exists(), "yan meta dosyası olmalı");
        assert!(!meta.sha256.is_empty(), "sha256 hesaplanmalı");

        let items = list_queue(base.to_str().unwrap()).await;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].meta.id, meta.id);

        let found = find_item(base.to_str().unwrap(), &meta.id).await;
        assert!(found.is_some(), "id ile bulunabilmeli");
        // Önek ile arama da çalışmalı
        let found_prefix = find_item(base.to_str().unwrap(), &meta.id[2..6]).await;
        assert!(found_prefix.is_some(), "id öneki ile bulunabilmeli");

        let _ = tokio::fs::remove_dir_all(&base).await;
    }

    /// TTL'i dolan öğe `hatali/` altına taşınır ve meta `ttl_expired` işaretlenir.
    #[tokio::test]
    async fn test_sweep_moves_expired_to_hatali() {
        let base = std::env::temp_dir().join(format!("kepce_q_ttl_{}", uuid::Uuid::new_v4()));
        let q_root = quarantine_root(base.to_str().unwrap(), "admin");
        tokio::fs::create_dir_all(&q_root).await.unwrap();

        let file_path = q_root.join("k_OLD1.xlsx");
        tokio::fs::write(&file_path, b"eski icerik").await.unwrap();

        // 15 gün öncesine tarihlenmiş meta (varsayılan TTL 14 gün)
        let old = (Utc::now() - chrono::Duration::days(15)).to_rfc3339();
        let meta = test_meta("k_OLD1", &old);
        let meta_path = sidecar_path(&file_path);
        tokio::fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap())
            .await
            .unwrap();

        let report = sweep(base.to_str().unwrap()).await.unwrap();
        assert_eq!(report.expired, vec!["k_OLD1".to_string()]);

        assert!(!file_path.exists(), "dosya karantinadan çıkarılmalı");
        let hatali = base.join("admin").join("hatali").join("k_OLD1.xlsx");
        assert!(hatali.exists(), "dosya hatali/ altına taşınmalı");
        let moved_meta: QuarantineMeta = serde_json::from_str(
            &tokio::fs::read_to_string(sidecar_path(&hatali))
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(moved_meta.resolution.as_deref(), Some("ttl_expired"));

        let items = list_queue(base.to_str().unwrap()).await;
        assert!(items.is_empty(), "kuyruk boşalmalı");

        let _ = tokio::fs::remove_dir_all(&base).await;
    }

    /// Taze öğe TTL taramasında yerinde kalır.
    #[tokio::test]
    async fn test_sweep_keeps_fresh_items() {
        let base = std::env::temp_dir().join(format!("kepce_q_fresh_{}", uuid::Uuid::new_v4()));
        let q_root = quarantine_root(base.to_str().unwrap(), "admin");
        tokio::fs::create_dir_all(&q_root).await.unwrap();

        let file_path = q_root.join("Taze_Dosya.xlsx");
        tokio::fs::write(&file_path, b"taze icerik").await.unwrap();
        let meta = test_meta("k_FRESH", &Utc::now().to_rfc3339());
        tokio::fs::write(
            sidecar_path(&file_path),
            serde_json::to_string_pretty(&meta).unwrap(),
        )
        .await
        .unwrap();

        let report = sweep(base.to_str().unwrap()).await.unwrap();
        assert!(report.expired.is_empty());
        assert!(file_path.exists(), "taze dosya yerinde kalmalı");

        let _ = tokio::fs::remove_dir_all(&base).await;
    }

    #[test]
    fn test_format_item_alert_polymorphic() {
        let mut meta = test_meta("k_POLY1", &Utc::now().to_rfc3339());
        meta.parsed_pricing = Some(crate::parser::models::OfficialPricingData {
            city_slug: Some("istanbul".to_string()),
            academic_year: Some("2026-2027".to_string()),
            period_start: None,
            period_end: None,
            items: vec![
                crate::parser::models::PricingCategoryItem {
                    meal_type: "dinner".to_string(),
                    category_name: "1. GRUP YEMEKLER (ÇORBALAR)".to_string(),
                    portion_amount: Some("250 GR".to_string()),
                    price: sea_orm::prelude::Decimal::from(35),
                },
                crate::parser::models::PricingCategoryItem {
                    meal_type: "breakfast".to_string(),
                    category_name: "KAHVALTI KALEMİ".to_string(),
                    portion_amount: None,
                    price: sea_orm::prelude::Decimal::from(20),
                },
            ],
        });
        meta.parsed_takeaway = Some(crate::parser::models::TakeawayData {
            city_slug: Some("istanbul".to_string()),
            academic_year: Some("2026-2027".to_string()),
            packages: vec![crate::parser::models::TakeawayPackageData {
                package_name: "Paket A".to_string(),
                slots: vec![crate::parser::models::TakeawaySlotData {
                    slot_index: 1,
                    slot_title: Some("Ana Sandviç".to_string()),
                    is_required: true,
                    items: vec![],
                }],
            }],
        });

        let item = QueueItem {
            meta,
            file_path: PathBuf::from("/tmp/test.jpg"),
            meta_path: PathBuf::from("/tmp/test.karantina.json"),
        };

        let alert = format_item_alert(&item, 1, NotifyKind::New);
        assert!(alert.contains("Fiyat Listesi: 1 Kahvaltı, 1 Yemek/Öğle/Akşam kalemi"));
        assert!(alert.contains("Akademik Dönem: 2026-2027"));
        assert!(alert.contains("Al Götür: 1 paket tespit edildi"));
        assert!(alert.contains("Paket A: 1 seçim slotu"));
        assert!(alert.contains("/onayla k_POLY1"));
    }

    #[test]
    fn test_item_inline_keyboard() {
        let kb = item_inline_keyboard("k_TEST1");
        let rows = kb
            .get("inline_keyboard")
            .and_then(|r| r.as_array())
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0]["text"], "Onayla");
        assert_eq!(rows[0][0]["callback_data"], "q:approve:k_TEST1");
        assert_eq!(rows[0][1]["text"], "Reddet");
        assert_eq!(rows[0][1]["callback_data"], "q:reject_confirm:k_TEST1");
        assert_eq!(rows[1][0]["text"], "Detay");
        assert_eq!(rows[1][0]["callback_data"], "q:detail:k_TEST1");
        assert_eq!(rows[1][1]["text"], "Dosyayı Gönder");
        assert_eq!(rows[1][1]["callback_data"], "q:file:k_TEST1");
    }
}
