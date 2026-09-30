//! Yerel menü ingest akışı (drop-zone) ve karar motoru.
//!
//! Temel ilkeler (yerel menü akışı sağlamlaştırma planı):
//! 1. **Sessiz başarı yoktur.** Bir dosya `vault`'a ancak tam ve tutarlı
//!    çıkarıldığında gider.
//! 2. **Şüpheli veri veritabanına asla yazılmaz.** Karar, yazma işleminden ÖNCE
//!    verilir; hayalet kayıt oluşamaz.
//! 3. **Operatör dosya tamir etmez, karar verir.** Şüpheli dosya `_karantina/`
//!    altına düşer, Telegram'dan tek dokunuşla `/onayla` veya `/reddet` denir.
//! 4. **Operatör kararı yoksa zaman aşımı karar verir.** Kuyruk TTL'i dolar ve
//!    gürültülü şekilde sonuçlanır.
//! 5. **Tarih yorumu deterministiktir.** LLM yalnızca ham metni taşır.

use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use shared::entities::cities;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::env;
use std::path::{Path, PathBuf};

use crate::parser::core::{
    DateOrderResolution, DeclaredMonth, ParseDiagnostics, extract_declared_month,
};
use crate::parser::models::MenuDatabase;
use crate::tasks::quarantine::{self, QuarantineDetail, ReasonCode};

/// Ağ/servis kaynaklı **geçici** hataları ayırt eder.
///
/// Geçici hatada dosya `bekleyen` klasöründe bırakılır ve sonraki tarama
/// döngüsünde yeniden denenir. Kalıcı hatada ise `hatali` klasörüne taşınır.
/// (Örn. Gemini 503 "high demand" / 429 "rate limit" geçicidir; tek denemede
/// kalıcı sayılıp `hatali` klasörüne atılması veri kaybına yol açar.)
fn is_transient_error(err_msg: &str) -> bool {
    const TRANSIENT_MARKERS: [&str; 13] = [
        "timeout",
        "geçici api hatası",
        "istek atılamadı",
        "service unavailable",
        "service_unavailable",
        "503",
        "429",
        "too many requests",
        "rate limit",
        "rate_limit",
        "high demand",
        "overloaded",
        "temporarily",
    ];
    TRANSIENT_MARKERS
        .iter()
        .any(|marker| err_msg.contains(marker))
}

// ---------------------------------------------------------------------------
// Kapı yapılandırması (K-7, K-8, K-9 + geri alma anahtarı)
// ---------------------------------------------------------------------------

/// Tamlık/kapsam kapısının eşikleri. Ortam değişkenleriyle ayarlanabilir.
#[derive(Debug, Clone)]
pub struct GateConfig {
    /// `WORKER_INGEST_STRICT=0` eski davranışa döner (kapı yok, yalnızca uyarı).
    pub strict: bool,
    /// `WORKER_MIN_DAYS_RATIO`: bu oranın üstü "kısmi", altı "şüpheli" (K-7).
    pub min_days_ratio: f64,
    /// `WORKER_ALLOW_MISSING_DAYS`: tam aydan düşülebilecek gün toleransı (K-9).
    pub allow_missing_days: u32,
    /// `WORKER_REQUIRE_DECLARED_MONTH=1`: beyan edilmemiş ay şüphelidir (K-8).
    pub require_declared_month: bool,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            strict: true,
            min_days_ratio: 0.6,
            allow_missing_days: 0,
            require_declared_month: false,
        }
    }
}

impl GateConfig {
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = env::var("WORKER_INGEST_STRICT") {
            cfg.strict = v.trim() != "0";
        }
        if let Some(r) = env::var("WORKER_MIN_DAYS_RATIO")
            .ok()
            .and_then(|v| v.trim().parse::<f64>().ok())
            .filter(|r| *r > 0.0 && *r <= 1.0)
        {
            cfg.min_days_ratio = r;
        }
        if let Some(d) = env::var("WORKER_ALLOW_MISSING_DAYS")
            .ok()
            .and_then(|v| v.trim().parse::<u32>().ok())
        {
            cfg.allow_missing_days = d;
        }
        if let Ok(v) = env::var("WORKER_REQUIRE_DECLARED_MONTH") {
            cfg.require_declared_month = v.trim() == "1";
        }
        cfg
    }
}

// ---------------------------------------------------------------------------
// Karar motoru (saf, G/Ç ve veritabanı yok — Faz 0.2)
// ---------------------------------------------------------------------------

/// Karar motorunun girdisi: ayrıştırılmış dosyanın saf özeti.
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub file_name: String,
    /// Çıkarılan tüm geçerli tarihler (sıralı, tekrarsız).
    pub dates: BTreeSet<NaiveDate>,
    pub breakfast: usize,
    pub lunch: usize,
    pub dinner: usize,
    /// Dosya adından çıkarılan beyan edilen ay (yalnızca çapraz doğrulama sinyali).
    pub declared_month: Option<DeclaredMonth>,
    pub diagnostics: ParseDiagnostics,
}

/// Karar motorunun çıktısı (Faz 0.1).
#[derive(Debug, Clone, PartialEq)]
pub enum IngestOutcome {
    /// Tam ve tutarlı: `vault`'a gider, veritabanına yazılır.
    Complete,
    /// Kısmi çıkarım: karar gerektirir, `_karantina/`, YAZIM YOK.
    Partial { reason: ReasonCode },
    /// Şüpheli veri: karar gerektirir, `_karantina/`, YAZIM YOK.
    Suspect { reason: ReasonCode },
    /// Kalıcı hata: `hatali/`, bir daha denenmez.
    Permanent { reason: ReasonCode },
    /// Geçici hata (ağ/API): dosya `bekleyen`'de kalır.
    Transient { reason: ReasonCode },
}

/// Ay kapsamı ve tamlık denetiminin ara sonucu.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScopeDecision {
    /// Yazılacak ay (çoğunluk ayı; onayda kapsam dışı tarihler düşürülür).
    pub scope_month: Option<(i32, u32)>,
    /// Veride görülen tüm aylar.
    pub detected_months: Vec<(i32, u32)>,
    /// Kapsam ayı dışına düşen tarihler (asla yazılmaz).
    pub stray_dates: Vec<NaiveDate>,
    /// Kapsam ayının takvim günü sayısı.
    pub expected_days: Option<u32>,
    /// Kapsam ayına düşen çıkarılmış gün sayısı.
    pub in_scope_days: usize,
}

/// `MenuDatabase` + dosya adı + tanılama sinyallerinden karar girdisini kurar.
pub fn build_parsed_file(
    file_name: &str,
    db: &MenuDatabase,
    diagnostics: ParseDiagnostics,
) -> ParsedFile {
    let mut dates = BTreeSet::new();
    let (mut breakfast, mut lunch, mut dinner) = (0usize, 0usize, 0usize);
    for (date_str, day) in db {
        if let Ok(d) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            dates.insert(d);
        }
        if !day.normal.breakfast.is_empty() || !day.colyak.breakfast.is_empty() {
            breakfast += 1;
        }
        if !day.normal.lunch.is_empty() || !day.colyak.lunch.is_empty() {
            lunch += 1;
        }
        if !day.normal.dinner.is_empty() || !day.colyak.dinner.is_empty() {
            dinner += 1;
        }
    }
    ParsedFile {
        file_name: file_name.to_string(),
        dates,
        breakfast,
        lunch,
        dinner,
        declared_month: extract_declared_month(file_name),
        diagnostics,
    }
}

fn month_key(d: NaiveDate) -> (i32, u32) {
    (d.year(), d.month())
}

fn calendar_days_in_month(year: i32, month: u32) -> u32 {
    let first = NaiveDate::from_ymd_opt(year, month, 1);
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    match (first, next) {
        (Some(a), Some(b)) => (b - a).num_days().max(0) as u32,
        _ => 0,
    }
}

pub fn format_month_tr(key: (i32, u32)) -> String {
    const MONTHS: [&str; 12] = [
        "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim",
        "Kasım", "Aralık",
    ];
    let name = MONTHS
        .get(key.1.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("?");
    format!("{} {}", name, key.0)
}

/// Kapsam ayını seçer: çoğunluk ayı; eşitlikte beyan edilen ay, o da yoksa en erken ay.
fn pick_scope_month(
    counts: &BTreeMap<(i32, u32), usize>,
    declared: Option<&DeclaredMonth>,
) -> Option<(i32, u32)> {
    let max = counts.values().max().copied()?;
    let candidates: Vec<(i32, u32)> = counts
        .iter()
        .filter(|&(_, &c)| c == max)
        .map(|(&k, _)| k)
        .collect();
    if candidates.len() == 1 {
        return Some(candidates[0]);
    }
    if let Some(dm) = declared
        && let Some(&hit) = candidates
            .iter()
            .find(|k| k.1 == dm.month && dm.year.is_none_or(|y| y == k.0))
    {
        return Some(hit);
    }
    candidates.first().copied()
}

/// Çıkarılan tarihlerin kesintisiz bir haftalık/iki haftalık blok oluşturup oluşturmadığını kontrol eder.
/// Ay geçişine denk gelen (ör. 28 Eylül - 4 Ekim) meşru haftalık menülerin MultiMonthScope hatasına
/// düşmesini engeller.
pub fn is_contiguous_weekly_span(dates: &BTreeSet<NaiveDate>) -> bool {
    if dates.len() < 3 || dates.len() > 14 {
        return false;
    }
    let Some(min_d) = dates.first() else {
        return false;
    };
    let Some(max_d) = dates.last() else {
        return false;
    };
    (max_d.signed_duration_since(*min_d)).num_days() + 1 == dates.len() as i64
}

/// Saf karar fonksiyonu (Faz 0.2, karar matrisi 4.4).
///
/// Hiçbir G/Ç veya veritabanı çağrısı içermez; sıralı zincir:
/// 1. `day_count == 0` -> `Permanent(NO_DATES)`
/// 2. Tarih sırası çelişkisi/çözülemeyen tarih -> `Suspect(AMBIGUOUS_DATE_ORDER)`
/// 3. LLM ham tarih uyuşmazlığı -> `Suspect(DATE_ORDER_MISMATCH)`
/// 4. Sütun başlığı gün adı uyuşmazlığı -> `Suspect(WEEKDAY_MISMATCH)`
/// 5. Zayıf sıra kanıtı -> `Suspect(WEAK_DATE_ORDER)`
/// 6. Birden fazla ay -> `Suspect(MULTI_MONTH_SCOPE)` (kesintisiz haftalık listeler hariç)
/// 7. Beyan edilen ay uyuşmuyor -> `Suspect(DECLARED_MONTH_MISMATCH)`
/// 8. Katı modda beyan yok -> `Suspect(UNDECLARED_MONTH)`
/// 9. Eksik <= tolerans -> `Complete`
/// 10. oran >= eşik -> `Partial(LOW_COVERAGE)`
/// 11. oran < eşik -> `Suspect(LOW_COVERAGE)`
pub fn classify_ingest(parsed: &ParsedFile, cfg: &GateConfig) -> (IngestOutcome, ScopeDecision) {
    let mut scope = ScopeDecision::default();

    // Satır 3: hiç tarih yok
    if parsed.dates.is_empty() {
        return (
            IngestOutcome::Permanent {
                reason: ReasonCode::NoDates,
            },
            scope,
        );
    }

    // Ay kapsamı (4.2): kapsam VERİDEN çıkar, dosya adına asla tek başına güvenilmez
    let mut counts: BTreeMap<(i32, u32), usize> = BTreeMap::new();
    for d in &parsed.dates {
        *counts.entry(month_key(*d)).or_insert(0) += 1;
    }
    scope.detected_months = counts.keys().copied().collect();
    let scope_month = pick_scope_month(&counts, parsed.declared_month.as_ref());
    scope.scope_month = scope_month;
    if let Some(sm) = scope_month {
        scope.expected_days = Some(calendar_days_in_month(sm.0, sm.1));
        scope.in_scope_days = counts.get(&sm).copied().unwrap_or(0);
        scope.stray_dates = parsed
            .dates
            .iter()
            .filter(|d| month_key(**d) != sm)
            .copied()
            .collect();
    }

    // Satır 4: tarih sırası çelişkisi veya çözülemeyen tarih hücreleri
    if let Some(DateOrderResolution::Conflict(_)) = parsed.diagnostics.date_order {
        return (
            IngestOutcome::Suspect {
                reason: ReasonCode::AmbiguousDateOrder,
            },
            scope,
        );
    }
    if !parsed.diagnostics.unparseable_dates.is_empty() {
        return (
            IngestOutcome::Suspect {
                reason: ReasonCode::AmbiguousDateOrder,
            },
            scope,
        );
    }
    // Satır 4 (LLM çapraz kontrolü): ISO çıktı ile ham tarih uyuşmuyor
    if !parsed.diagnostics.date_raw_mismatches.is_empty() {
        return (
            IngestOutcome::Suspect {
                reason: ReasonCode::DateOrderMismatch,
            },
            scope,
        );
    }
    // Satır 4b: Sütun başlığı gün adı ile tarihin haftanın günü uyuşmuyor
    if !parsed.diagnostics.weekday_mismatches.is_empty() {
        return (
            IngestOutcome::Suspect {
                reason: ReasonCode::WeekdayMismatch,
            },
            scope,
        );
    }
    // Satır 4c: zayıf sıra kanıtı (T3)
    if let Some(DateOrderResolution::Weak(_)) = parsed.diagnostics.date_order {
        return (
            IngestOutcome::Suspect {
                reason: ReasonCode::WeakDateOrder,
            },
            scope,
        );
    }

    // Satır 5: birden fazla aya ait tarih (Haziran vakası: 46146 -> 4 Mayıs).
    // İstisna: Kesintisiz haftalık/iki haftalık ay geçişi menüleri (ör. 28 Eylül - 4 Ekim).
    let is_weekly = is_contiguous_weekly_span(&parsed.dates);
    if is_weekly && scope.detected_months.len() > 1 {
        scope.expected_days = Some(parsed.dates.len() as u32);
        scope.in_scope_days = parsed.dates.len();
        scope.stray_dates.clear();
    } else if scope.detected_months.len() > 1 {
        return (
            IngestOutcome::Suspect {
                reason: ReasonCode::MultiMonthScope,
            },
            scope,
        );
    }

    // Satır 6: beyan edilen ay ile veri kapsamı uyuşmuyor (haftalık ay geçişi değilse)
    if !is_weekly
        && let Some(dm) = &parsed.declared_month
        && let Some(sm) = scope.scope_month
        && (dm.month != sm.1 || dm.year.is_some_and(|y| y != sm.0))
    {
        return (
            IngestOutcome::Suspect {
                reason: ReasonCode::DeclaredMonthMismatch,
            },
            scope,
        );
    }
    // Satır 6b: katı modda beyan edilmemiş ay
    if cfg.require_declared_month && parsed.declared_month.is_none() {
        return (
            IngestOutcome::Suspect {
                reason: ReasonCode::UndeclaredMonth,
            },
            scope,
        );
    }

    // Satır 7-9: tamlık (4.3). Eski `span` sezgisi KULLANILMAZ; kapsam ve
    // tamlık ayrık iki adımdır (D-3b'nin arka kapısı kapalı).
    let expected = scope.expected_days.unwrap_or(0);
    let missing = expected.saturating_sub(scope.in_scope_days as u32);
    if missing <= cfg.allow_missing_days {
        return (IngestOutcome::Complete, scope);
    }
    let ratio = scope.in_scope_days as f64 / expected.max(1) as f64;
    if ratio >= cfg.min_days_ratio {
        (
            IngestOutcome::Partial {
                reason: ReasonCode::LowCoverage,
            },
            scope,
        )
    } else {
        (
            IngestOutcome::Suspect {
                reason: ReasonCode::LowCoverage,
            },
            scope,
        )
    }
}

/// Karantina meta dosyası ve Telegram mesajı için Türkçe teşhis cümlesi.
pub fn reason_message(reason: ReasonCode, parsed: &ParsedFile, scope: &ScopeDecision) -> String {
    let scope_tr = scope
        .scope_month
        .map(format_month_tr)
        .unwrap_or_else(|| "bilinmeyen ay".to_string());
    match reason {
        ReasonCode::NoDates => "Dosyadan hiç geçerli tarih çıkarılamadı (0 gün).".to_string(),
        ReasonCode::MultiMonthScope => format!(
            "Dosya birden fazla aya ait tarih içeriyor ({}). Kaynak dosyada yazım hatası olabilir.",
            scope
                .detected_months
                .iter()
                .map(|m| format!("{}-{:02}", m.0, m.1))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ReasonCode::DeclaredMonthMismatch => format!(
            "Dosya adı '{}' ayını beyan ediyor ama veriler {} ayına ait.",
            parsed
                .declared_month
                .map(|dm| format_month_tr((dm.year.unwrap_or(0), dm.month)))
                .unwrap_or_else(|| "?".to_string()),
            scope_tr
        ),
        ReasonCode::UndeclaredMonth => {
            "Dosya adı ay beyan etmiyor ve katı mod (WORKER_REQUIRE_DECLARED_MONTH=1) açık."
                .to_string()
        }
        ReasonCode::LowCoverage => format!(
            "{} eksik kapsanıyor: {} gün çıkarıldı, beklenen {} gün.",
            scope_tr,
            scope.in_scope_days,
            scope.expected_days.unwrap_or(0)
        ),
        ReasonCode::AmbiguousDateOrder => format!(
            "Tarih sırası (gün.ay mı ay.gün mü) deterministik olarak çözülemedi. {}",
            parsed
                .diagnostics
                .date_order
                .as_ref()
                .and_then(|r| match r {
                    DateOrderResolution::Conflict(d) => Some(d.as_str()),
                    _ => None,
                })
                .unwrap_or("Bazı tarih hücreleri seçilen sırayla geçerli tarih üretmiyor.")
        ),
        ReasonCode::WeakDateOrder => {
            "Tarih sırası kanıtı zayıf: tek tarihli veya tek biçimli dosyada gün/ay sırası kanıtlanamıyor."
                .to_string()
        }
        ReasonCode::DateOrderMismatch => {
            "LLM'in ürettiği ISO tarihleri, belgedeki ham yazılı tarihlerin deterministik çözümüyle uyuşmuyor."
                .to_string()
        }
        ReasonCode::WeekdayMismatch => {
            if parsed.diagnostics.weekday_mismatches.is_empty() {
                "Sütun başlığındaki gün adı ile tarihin haftanın günü uyuşmuyor.".to_string()
            } else {
                let sample: Vec<&str> = parsed
                    .diagnostics
                    .weekday_mismatches
                    .iter()
                    .take(3)
                    .map(|m| m.as_str())
                    .collect();
                format!(
                    "Sütun başlığındaki gün adı ile tarihin günü uyuşmuyor ({} uyuşmazlık: {}).",
                    parsed.diagnostics.weekday_mismatches.len(),
                    sample.join("; ")
                )
            }
        }
        ReasonCode::NoCity => {
            "Dosya bekleyen/ köküne bırakılmış, bir şehir klasöründe değil. /ata <id> <sehir> ile şehir atayın."
                .to_string()
        }
        ReasonCode::UnknownCity => {
            "Şehir klasörü adı cities tablosunda yok. /ata <id> <sehir> ile doğru şehri atayın."
                .to_string()
        }
        ReasonCode::TtlExpired => "Karantina TTL'i doldu.".to_string(),
    }
}

/// Tanılama sinyallerini karantina meta ayrıntılarına çevirir.
fn detail_from(parsed: &ParsedFile, scope: &ScopeDecision) -> QuarantineDetail {
    let mut details = Vec::new();
    if let Some(DateOrderResolution::Conflict(d)) = &parsed.diagnostics.date_order {
        details.push(format!("Tarih sırası çelişkisi: {}", d));
    }
    if let Some(DateOrderResolution::Weak(_)) = &parsed.diagnostics.date_order {
        details.push("Tarih sırası kararı zayıf (T3: tek tarih/tek biçim).".to_string());
    }
    for u in &parsed.diagnostics.unparseable_dates {
        details.push(format!("Çözülemeyen tarih hücresi: '{}'", u));
    }
    for w in &parsed.diagnostics.weekday_mismatches {
        details.push(format!("Gün adı uyuşmazlığı: {}", w));
    }
    for m in &parsed.diagnostics.date_raw_mismatches {
        details.push(format!("Ham tarih uyuşmazlığı: {}", m));
    }
    QuarantineDetail {
        detected_months: scope
            .detected_months
            .iter()
            .map(|m| format!("{}-{:02}", m.0, m.1))
            .collect(),
        scope_month: scope.scope_month.map(|m| format!("{}-{:02}", m.0, m.1)),
        day_count: parsed.dates.len(),
        expected_days: scope.expected_days,
        stray_dates: scope
            .stray_dates
            .iter()
            .map(|d| d.format("%Y-%m-%d").to_string())
            .collect(),
        details,
    }
}

/// Tek öğün tipi uyarısı için minimum gün sayısı (gözlemlenebilirlik, bloklamaz).
const MEAL_MIX_MIN_DAYS: usize = 10;

/// Çok günlük belgede yalnızca tek öğün tipi varsa uyarı üretir (WARN; bloklamaz).
///
/// Eski `low_days_warning`'ın `span` tabanlı sezgisi KALDIRILDI (plan 4.3.7):
/// kapsam ve tamlık artık `classify_ingest` içinde ayrık iki adımdır.
fn meal_mix_warning(parsed: &ParsedFile) -> Option<String> {
    let days = parsed.dates.len();
    if days >= MEAL_MIX_MIN_DAYS {
        let meal_types_present = [parsed.breakfast, parsed.lunch, parsed.dinner]
            .iter()
            .filter(|&&c| c > 0)
            .count();
        if meal_types_present == 1 {
            return Some(format!(
                "{} günlük belgede yalnızca tek öğün tipi bulundu (kahvaltı {}, öğle {}, akşam {}). Çoklu-tablo atlanmış olabilir.",
                days, parsed.breakfast, parsed.lunch, parsed.dinner
            ));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tek taşıma noktası (Faz 0.3)
// ---------------------------------------------------------------------------

/// Dosyanın son durağı.
pub enum FinalDest<'a> {
    /// Tam ve onaylı: `vault/<uzanti>/<rol>/<sehir>/`
    Vault { ext: &'a str, city_slug: &'a str },
    /// Kalıcı hata: `<rol>/hatali/`
    Hatali,
}

/// Tek taşıma fonksiyonu: vault ve hatali hedefleri buradan yönetilir.
///
/// rename -> copy+remove zinciri korunur; ikisi de başarısızsa sonsuz döngüyü
/// önlemek için dosya uzantısı `.failed` / `.processed` yapılır.
pub async fn finalize_file(base_dir: &str, folder: &str, path: &Path, dest: FinalDest<'_>) {
    let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
        tracing::error!("Dosya adı çözülemedi: {:?}", path);
        return;
    };
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let (target_dir, dest_name, escape_ext) = match dest {
        FinalDest::Vault { ext, city_slug } => (
            PathBuf::from(base_dir)
                .join("vault")
                .join(ext.to_lowercase())
                .join(folder)
                .join(city_slug),
            format!("{}_{}", Local::now().format("%Y%m%d_%H%M%S"), filename),
            "processed",
        ),
        FinalDest::Hatali => (
            PathBuf::from(base_dir).join(folder).join("hatali"),
            filename.to_string(),
            "failed",
        ),
    };

    let _ = tokio::fs::create_dir_all(&target_dir).await;
    let dest_path = target_dir.join(&dest_name);

    if let Err(e) = tokio::fs::rename(path, &dest_path).await {
        tracing::warn!("Dosya taşınamadı ({:?}). Kopyalama + silme deneniyor...", e);
        if let Err(copy_err) = tokio::fs::copy(path, &dest_path).await {
            tracing::error!(
                "Kopyalama başarısız ({:?}). Sonsuz döngüyü önlemek için dosya uzantısı .{} yapılıyor...",
                copy_err,
                escape_ext
            );
            let failed_dest = path.with_extension(format!("{}.{}", ext, escape_ext));
            if let Err(rename_err) = tokio::fs::rename(path, &failed_dest).await {
                tracing::error!(
                    "Dosya .{} olarak yeniden adlandırılamadı: {:?}",
                    escape_ext,
                    rename_err
                );
            }
        } else if let Err(remove_err) = tokio::fs::remove_file(path).await {
            tracing::error!(
                "Kaynak dosya silinemedi ({:?}): {:?}. Yeniden işlenmemesi için .{} yapılıyor...",
                path,
                remove_err,
                escape_ext
            );
            let _ = tokio::fs::rename(path, path.with_extension(format!("{}.{}", ext, escape_ext)))
                .await;
        }
    }
}

// ---------------------------------------------------------------------------
// Ayrıştırma (veritabanına YAZMADAN)
// ---------------------------------------------------------------------------

/// Dosyayı veritabanına DOKUNMADAN ayrıştırır.
///
/// `Ok(None)` = desteklenmeyen uzantı veya LLM devre dışı; dosya yerinde kalır.
/// Karar yazımdan ÖNCE verildiği için hiçbir dal burada `save_menu_database`
/// çağırmaz (plan ilke 2).
pub async fn parse_local_file(
    path: &Path,
    city_slug: &str,
    reqwest_client: &reqwest::Client,
    gemini_api_key: Option<&str>,
) -> Result<Option<(MenuDatabase, ParseDiagnostics)>> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let path_str = path.to_string_lossy().to_string();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    if ext == "xlsx" {
        let mut file_db = MenuDatabase::new();
        let diag = crate::parser::excel::parse_excel_with_diagnostics(&path_str, &mut file_db)
            .map_err(|e| anyhow::anyhow!("Excel parse hatası: {}", e))?;
        for day_data in file_db.values_mut() {
            crate::parser::validation::finalize_day_metadata(day_data);
        }
        return Ok(Some((file_db, diag)));
    }

    if ext == "json" {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| anyhow::anyhow!("JSON dosyası okunamadı: {}", e))?;
        let (mut file_db, mismatches) =
            crate::parser::json::parse_json_str_with_diagnostics(&content, file_name)
                .map_err(|e| anyhow::anyhow!("JSON parse hatası: {}", e))?;
        for day_data in file_db.values_mut() {
            crate::parser::validation::finalize_day_metadata(day_data);
        }
        let diag = ParseDiagnostics {
            date_raw_mismatches: mismatches,
            ..Default::default()
        };
        return Ok(Some((file_db, diag)));
    }

    if matches!(
        ext.as_str(),
        "pdf" | "png" | "jpg" | "jpeg" | "webp" | "heic" | "heif"
    ) {
        if !crate::parser::llm::llm_available(gemini_api_key) {
            tracing::warn!(
                "{}: LLM parsing devre dışı - hiçbir sağlayıcı anahtarı (OPENROUTER_API_KEY/GEMINI_API_KEY) ayarlanmamış, atlanıyor.",
                file_name
            );
            return Ok(None);
        }
        let (file_db, diag) =
            crate::parser::llm::parse_document_with_llm(reqwest_client, gemini_api_key, path)
                .await
                .map_err(|e| anyhow::anyhow!("Belge/Görsel LLM parse hatası: {}", e))?;
        let _ = city_slug;
        return Ok(Some((file_db, diag)));
    }

    Ok(None)
}

/// Yazılacak (tarih, öğün) ikilileri kümesi (ay atomikliği için).
fn build_keep_set(db: &MenuDatabase) -> HashSet<(NaiveDate, String)> {
    let mut keep = HashSet::new();
    for (date_str, day) in db {
        let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
            continue;
        };
        if !day.normal.breakfast.is_empty() || !day.colyak.breakfast.is_empty() {
            keep.insert((date, "breakfast".to_string()));
        }
        if !day.normal.lunch.is_empty() || !day.colyak.lunch.is_empty() {
            keep.insert((date, "lunch".to_string()));
        }
        if !day.normal.dinner.is_empty() || !day.colyak.dinner.is_empty() {
            keep.insert((date, "dinner".to_string()));
        }
    }
    keep
}

/// Kapsam ayı dışındaki tarihleri düşürür (plan 4.4: terfi kapsam dışı hiçbir
/// tarihi YAZMAZ; operatörün eliyle bile hayalet kayıt oluşturulamaz).
pub fn filter_to_scope(db: MenuDatabase, scope_month: (i32, u32)) -> (MenuDatabase, Vec<String>) {
    let mut kept = MenuDatabase::new();
    let mut dropped = Vec::new();
    for (date_str, day) in db {
        let in_scope = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map(|d| month_key(d) == scope_month)
            .unwrap_or(false);
        if in_scope {
            kept.insert(date_str, day);
        } else {
            dropped.push(date_str);
        }
    }
    dropped.sort();
    (kept, dropped)
}

/// Kapsam içi veriyi yazar + ay atomikliğini (Faz 5.2) uygular.
async fn write_scoped_menu(
    db: &DatabaseConnection,
    city_id: i32,
    source_type: &str,
    city_slug: &str,
    file_db: MenuDatabase,
    scope_month: Option<(i32, u32)>,
    filename: &str,
) -> Result<usize> {
    let (filtered, dropped) = match scope_month {
        Some(sm) => filter_to_scope(file_db, sm),
        None => (file_db, Vec::new()),
    };
    for d in &dropped {
        tracing::warn!(
            "{}: kapsam dışı tarih YAZILMADI ve düşürüldü: {} (kapsam: {})",
            filename,
            d,
            scope_month
                .map(|m| format!("{}-{:02}", m.0, m.1))
                .unwrap_or_else(|| "?".to_string())
        );
    }

    let keep = build_keep_set(&filtered);
    let written_days = filtered.len();
    crate::parser::save_menu_database(db, city_id, source_type, filtered, city_slug).await?;

    // Faz 5.2: ay atomikliği. Bu kaynak için (şehir, ay) kapsamında yeni dosyada
    // bulunmayan (tarih, öğün) ikilileri sert DELETE ile silinir.
    if let Some((y, m)) = scope_month
        && let Some(month_start) = NaiveDate::from_ymd_opt(y, m, 1)
    {
        match crate::tasks::scraper::delete_out_of_scope_menus(
            db,
            city_id,
            "kepce-",
            month_start,
            &keep,
        )
        .await
        {
            Ok(0) => {}
            Ok(n) => tracing::warn!(
                "{}: ay atomikliği — kapsamda yeni dosyada olmayan {} kayıt silindi.",
                filename,
                n
            ),
            Err(e) => tracing::error!(
                "{}: ay atomikliği temizliği başarısız (yazım tamam): {:?}",
                filename,
                e
            ),
        }
    }
    Ok(written_days)
}

// ---------------------------------------------------------------------------
// Ana döngü
// ---------------------------------------------------------------------------

static INGEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn process_local_files(
    db: &DatabaseConnection,
    reqwest_client: &reqwest::Client,
    gemini_api_key: Option<&str>,
) -> Result<()> {
    let Ok(_lock_guard) = INGEST_LOCK.try_lock() else {
        tracing::info!(
            "[LOKAL] Ingest işlemi şu anda başka bir iş parçacığı tarafından yürütülüyor, eşzamanlı çalıştırma atlandı."
        );
        return Ok(());
    };
    let base_dir = env::var("WORKER_MENU_DIR").unwrap_or_else(|_| "../data/menuler".to_string());
    let cfg = GateConfig::from_env();

    // Faz 0.6 / 5.4: uyarı kanalı zorunluluğu. Operatörsüz karantina sessiz
    // kaybın yeni adı olur; bu yüzden açılışta gürültülü şekilde işaretlenir.
    if !shared::services::alerting::AlertingService::alert_channel_configured() {
        tracing::error!(
            "[LOKAL] Uyarı kanalı TANIMLI DEĞİL (TELEGRAM_ADMIN_CHAT_ID veya ALERT_WEBHOOK_URL yok). \
             Karantinaya düşen dosyalar operatöre ULAŞMAYACAK; TTL sonunda hatali/ altına taşınacaktır."
        );
    }

    // Karantina kuyruğu TTL + hatırlatma taraması (plan 5.3): kuyruk kendi
    // kendine asla sessizce birikmez.
    match quarantine::sweep(&base_dir).await {
        Ok(report) => {
            if !report.expired.is_empty() || !report.escalated.is_empty() {
                tracing::warn!(
                    "[KARANTİNA] tarama: {} öğe TTL ile hatali/'ya taşındı, {} öğe kırmızı alarma yükseltildi.",
                    report.expired.len(),
                    report.escalated.len()
                );
            }
        }
        Err(e) => tracing::error!("[KARANTİNA] tarama hatası: {:?}", e),
    }

    let configs = ["admin", "kullanici", "anonim"];
    let mut processed = 0;

    for folder in configs.iter() {
        let bekleyen_path = PathBuf::from(&base_dir).join(folder).join("bekleyen");
        if !bekleyen_path.exists() {
            continue;
        }

        let mut cities_iter = match tokio::fs::read_dir(&bekleyen_path).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        while let Ok(Some(city_entry)) = cities_iter.next_entry().await {
            let city_path = city_entry.path();

            // Faz 2.1 (D-1): bekleyen/ KÖKÜNDEKİ dosyalar artık sessizce
            // atlanmaz; karantinaya düşer ve operatöre şehir sorulur.
            if !city_path.is_dir() {
                let entry_type = city_entry.file_type().await;
                if entry_type.map(|t| t.is_symlink()).unwrap_or(false) {
                    continue;
                }
                let fname = city_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                tracing::warn!(
                    "{}: dosya bekleyen/ kökünde bulundu (şehir klasörü yok), karantinaya alınıyor.",
                    fname
                );
                let parsed_stub = ParsedFile {
                    file_name: fname.clone(),
                    dates: BTreeSet::new(),
                    breakfast: 0,
                    lunch: 0,
                    dinner: 0,
                    declared_month: None,
                    diagnostics: ParseDiagnostics::default(),
                };
                let stub_scope = ScopeDecision::default();
                if let Err(e) = quarantine::quarantine_file(
                    &base_dir,
                    folder,
                    None,
                    &city_path,
                    ReasonCode::NoCity,
                    reason_message(ReasonCode::NoCity, &parsed_stub, &stub_scope),
                    QuarantineDetail::default(),
                )
                .await
                {
                    tracing::error!("{} karantinaya alınamadı: {:?}", fname, e);
                }
                continue;
            }

            let city_slug = city_entry.file_name().to_string_lossy().to_string();

            // Get city ID from slug
            let city_opt = match cities::Entity::find()
                .filter(cities::Column::Slug.eq(&city_slug))
                .one(db)
                .await
            {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!(
                        "Şehir sorgusu başarısız ('{}'), klasör bu döngüde atlanıyor: {:?}",
                        city_slug,
                        e
                    );
                    continue;
                }
            };

            let city_id = match city_opt {
                Some(c) => c.id,
                None => {
                    // Faz 2.2 (D-1b): bilinmeyen şehir klasörü artık sessizce
                    // bekletilmez; içindeki her dosya karantinaya düşer.
                    tracing::warn!(
                        "Lokal dosya taraması: '{}' adlı şehir veritabanında bulunamadı; klasördeki dosyalar karantinaya alınıyor.",
                        city_slug
                    );
                    if let Ok(mut inner) = tokio::fs::read_dir(&city_path).await {
                        while let Ok(Some(f)) = inner.next_entry().await {
                            let p = f.path();
                            if !p.is_file() {
                                continue;
                            }
                            let detail = QuarantineDetail {
                                details: vec![format!(
                                    "bekleyen/ altındaki klasör adı: '{}' (cities tablosunda yok)",
                                    city_slug
                                )],
                                ..Default::default()
                            };
                            if let Err(e) = quarantine::quarantine_file(
                                &base_dir,
                                folder,
                                Some(&city_slug),
                                &p,
                                ReasonCode::UnknownCity,
                                reason_message(
                                    ReasonCode::UnknownCity,
                                    &ParsedFile {
                                        file_name: p
                                            .file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy()
                                            .to_string(),
                                        dates: BTreeSet::new(),
                                        breakfast: 0,
                                        lunch: 0,
                                        dinner: 0,
                                        declared_month: None,
                                        diagnostics: ParseDiagnostics::default(),
                                    },
                                    &ScopeDecision::default(),
                                ),
                                detail,
                            )
                            .await
                            {
                                tracing::error!("{:?} karantinaya alınamadı: {:?}", p, e);
                            }
                        }
                    }
                    continue;
                }
            };

            let mut files_iter = match tokio::fs::read_dir(&city_path).await {
                Ok(f) => f,
                Err(_) => continue,
            };

            while let Ok(Some(file_entry)) = files_iter.next_entry().await {
                let path = file_entry.path();
                if !path.is_file() {
                    continue;
                }
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                tracing::info!("Lokal dosya ayrıştırılıyor: {}/{}", city_slug, filename);

                let source_type = format!("kepce-{}", folder);

                // 1. Ayrıştır (veritabanına HENÜZ yazılmıyor)
                let parsed_opt = match parse_local_file(
                    &path,
                    &city_slug,
                    reqwest_client,
                    gemini_api_key,
                )
                .await
                {
                    Ok(Some(v)) => Some(v),
                    Ok(None) => {
                        tracing::warn!(
                            "{}: desteklenmeyen uzantı veya LLM devre dışı, dosya bekleyen'de bırakıldı.",
                            filename
                        );
                        continue;
                    }
                    Err(e) => {
                        let err_msg = format!("{:?}", e).to_lowercase();
                        if is_transient_error(&err_msg) {
                            tracing::error!(
                                "{}: Geçici ağ/API hatası, dosya kuyrukta bekletilecek: {:?}",
                                filename,
                                e
                            );
                            continue; // Transient: dosya bekleyen'de kalır
                        }
                        tracing::error!("{}: Kalıcı ayrıştırma hatası: {:?}", filename, e);
                        let _ = shared::services::alerting::AlertingService::send_webhook_alert(
                            &format!(
                                "🔴 KALICI HATA  {}\n{} · {}\nSebep: PARSE_ERROR\n{:?}",
                                filename, city_slug, source_type, e
                            ),
                        )
                        .await;
                        finalize_file(&base_dir, folder, &path, FinalDest::Hatali).await;
                        continue;
                    }
                };
                let Some((file_db, diag)) = parsed_opt else {
                    continue;
                };

                // 2. Karar ver (saf karar motoru)
                let parsed = build_parsed_file(&filename, &file_db, diag);
                let (mut outcome, scope) = classify_ingest(&parsed, &cfg);

                if let Some(warning) = meal_mix_warning(&parsed) {
                    tracing::warn!("{}: {}", filename, warning);
                }

                // Geri alma anahtarı: WORKER_INGEST_STRICT=0 eski davranışa döner
                // (kapı yok, yalnızca uyarı loglanır).
                if !cfg.strict {
                    match &outcome {
                        IngestOutcome::Partial { reason }
                        | IngestOutcome::Suspect { reason }
                        | IngestOutcome::Permanent { reason } => {
                            tracing::warn!(
                                "{}: STRICT OLMAYAN mod — {} kararı yok sayılıyor, eski davranışla yazılacak.",
                                filename,
                                reason.as_str()
                            );
                            outcome = IngestOutcome::Complete;
                        }
                        _ => {}
                    }
                }

                // 3. Sonuca göre eyleme geç
                match outcome {
                    IngestOutcome::Complete => {
                        tracing::info!(
                            "{}: {} gün ({} kahvaltı + {} öğle + {} akşam) TAM bulundu (kaynak: {})",
                            filename,
                            parsed.dates.len(),
                            parsed.breakfast,
                            parsed.lunch,
                            parsed.dinner,
                            source_type
                        );
                        match write_scoped_menu(
                            db,
                            city_id,
                            &source_type,
                            &city_slug,
                            file_db,
                            scope.scope_month,
                            &filename,
                        )
                        .await
                        {
                            Ok(_) => {
                                processed += 1;
                                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                                finalize_file(
                                    &base_dir,
                                    folder,
                                    &path,
                                    FinalDest::Vault {
                                        ext,
                                        city_slug: &city_slug,
                                    },
                                )
                                .await;
                            }
                            Err(e) => {
                                let err_msg = format!("{:?}", e).to_lowercase();
                                if is_transient_error(&err_msg) {
                                    tracing::error!(
                                        "{}: Kaydetme sırasında geçici hata, dosya kuyrukta bekletilecek: {:?}",
                                        filename,
                                        e
                                    );
                                } else {
                                    tracing::error!(
                                        "{}: Kalıcı kaydetme hatası: {:?}",
                                        filename,
                                        e
                                    );
                                    finalize_file(&base_dir, folder, &path, FinalDest::Hatali)
                                        .await;
                                }
                            }
                        }
                    }
                    IngestOutcome::Partial { reason } | IngestOutcome::Suspect { reason } => {
                        // ŞÜPHELİ VERİ VERİTABANINA YAZILMAZ; karar operatörün.
                        let label = if matches!(outcome, IngestOutcome::Partial { .. }) {
                            "KISMİ"
                        } else {
                            "ŞÜPHELİ"
                        };
                        tracing::warn!(
                            "{}: {} — {} ({}). Karantina kuyruğuna alındı, veritabanına yazılmadı.",
                            filename,
                            label,
                            reason.as_str(),
                            reason_message(reason, &parsed, &scope)
                        );
                        if let Err(e) = quarantine::quarantine_file(
                            &base_dir,
                            folder,
                            Some(&city_slug),
                            &path,
                            reason,
                            reason_message(reason, &parsed, &scope),
                            detail_from(&parsed, &scope),
                        )
                        .await
                        {
                            tracing::error!("{} karantinaya alınamadı: {:?}", filename, e);
                        }
                    }
                    IngestOutcome::Permanent { reason } => {
                        tracing::error!(
                            "{}: KALICI HATA — {} ({}). hatali/ altına taşınıyor.",
                            filename,
                            reason.as_str(),
                            reason_message(reason, &parsed, &scope)
                        );
                        let _ = shared::services::alerting::AlertingService::send_webhook_alert(
                            &format!(
                                "🔴 KALICI HATA  {}\n{} · {}\nSebep: {}\n{}",
                                filename,
                                city_slug,
                                source_type,
                                reason.as_str(),
                                reason_message(reason, &parsed, &scope)
                            ),
                        )
                        .await;
                        finalize_file(&base_dir, folder, &path, FinalDest::Hatali).await;
                    }
                    IngestOutcome::Transient { reason } => {
                        tracing::warn!(
                            "{}: geçici durum ({}), dosya bekleyen'de bırakıldı.",
                            filename,
                            reason.as_str()
                        );
                    }
                }
            }
        }
    }

    tracing::info!(
        "Lokal dosya taraması tamamlandı. {} dosya işlendi.",
        processed
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Operatör komutları (Telegram botu buradan çağırır — Faz 0.5)
// ---------------------------------------------------------------------------

/// `/onayla <id>`: karantinadaki dosyayı KAPSAM İÇİ tarihlerle işler.
///
/// Terfi kapsam dışı hiçbir tarihi yazmaz: ay dışı tarihler sessizce değil,
/// log ve rapor eşliğinde düşürülür. Böylece hayalet kayıt operatörün eliyle
/// bile oluşturulamaz.
pub async fn approve_quarantine_item(
    db: &DatabaseConnection,
    reqwest_client: &reqwest::Client,
    gemini_api_key: Option<&str>,
    id: &str,
) -> Result<String> {
    let base = quarantine::menu_base_dir();
    let item = quarantine::find_item(&base, id)
        .await
        .ok_or_else(|| anyhow::anyhow!("'{}' kimlikli karantina öğesi bulunamadı.", id))?;

    let city_slug = item.meta.city.clone().ok_or_else(|| {
        anyhow::anyhow!(
            "Öğenin şehri yok ({}). Önce /ata {} <sehir> komutuyla şehir atayın.",
            item.meta.id,
            item.meta.id
        )
    })?;
    let city = cities::Entity::find()
        .filter(cities::Column::Slug.eq(&city_slug))
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("'{}' şehri veritabanında yok.", city_slug))?;

    let (file_db, diag) =
        parse_local_file(&item.file_path, &city_slug, reqwest_client, gemini_api_key)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Dosya ayrıştırılamadı (desteklenmeyen uzantı veya LLM kapalı): {}",
                    item.meta.file
                )
            })?;

    let parsed = build_parsed_file(&item.meta.file, &file_db, diag);
    let cfg = GateConfig::from_env();
    let (_, scope) = classify_ingest(&parsed, &cfg);

    // Kapsam ayı: meta dosyasındaki teşhis anındaki kapsam esas alınır; yoksa
    // yeniden hesaplanan kapsam kullanılır.
    let scope_month = item
        .meta
        .scope_month
        .as_deref()
        .and_then(|s| {
            let mut parts = s.split('-');
            let y = parts.next()?.parse::<i32>().ok()?;
            let m = parts.next()?.parse::<u32>().ok()?;
            Some((y, m))
        })
        .or(scope.scope_month)
        .ok_or_else(|| anyhow::anyhow!("Kapsam ayı belirlenemedi, onay iptal."))?;

    let source_type = format!("kepce-{}", item.meta.role);
    let written_days = write_scoped_menu(
        db,
        city.id,
        &source_type,
        &city_slug,
        file_db,
        Some(scope_month),
        &item.meta.file,
    )
    .await?;

    let dropped: Vec<String> = parsed
        .dates
        .iter()
        .filter(|d| month_key(**d) != scope_month)
        .map(|d| d.format("%Y-%m-%d").to_string())
        .collect();

    // Dosyayı vault'a taşı, yan meta dosyasını kaldır
    let ext = item
        .file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    finalize_file(
        &base,
        &item.meta.role,
        &item.file_path,
        FinalDest::Vault {
            ext,
            city_slug: &city_slug,
        },
    )
    .await;
    let _ = tokio::fs::remove_file(&item.meta_path).await;

    let _ = shared::services::alerting::AlertingService::send_webhook_alert(&format!(
        "✅ KARANTİNA ONAYLANDI  {}\n{} · {}\n{} gün yazıldı (kapsam {}-{:02}).{}",
        item.meta.id,
        city_slug,
        item.meta.file,
        written_days,
        scope_month.0,
        scope_month.1,
        if dropped.is_empty() {
            String::new()
        } else {
            format!(
                "\nKapsam dışı {} tarih YAZILMADI: {}",
                dropped.len(),
                dropped.join(", ")
            )
        }
    ))
    .await;

    Ok(format!(
        "✅ *Onaylandı* `{}`\n• {} · {}\n• {} gün yazıldı (kapsam `{}-{:02}`)\n• Kapsam dışı düşürülen tarih: {}",
        item.meta.id,
        city_slug,
        item.meta.file,
        written_days,
        scope_month.0,
        scope_month.1,
        if dropped.is_empty() {
            "yok".to_string()
        } else {
            dropped.join(", ")
        }
    ))
}

/// `/reddet <id>`: dosyayı `hatali/` altına taşır ve sonucu meta'ya işler.
pub async fn reject_quarantine_item(id: &str) -> Result<String> {
    let base = quarantine::menu_base_dir();
    let item = quarantine::find_item(&base, id)
        .await
        .ok_or_else(|| anyhow::anyhow!("'{}' kimlikli karantina öğesi bulunamadı.", id))?;

    let err_dir = PathBuf::from(&base).join(&item.meta.role).join("hatali");
    tokio::fs::create_dir_all(&err_dir).await?;
    let dest = err_dir.join(&item.meta.file);
    quarantine::move_file(&item.file_path, &dest).await?;

    let mut meta = item.meta.clone();
    meta.resolution = Some("rejected".to_string());
    let new_meta_path = quarantine::sidecar_path(&dest);
    tokio::fs::write(&new_meta_path, serde_json::to_string_pretty(&meta)?).await?;
    let _ = tokio::fs::remove_file(&item.meta_path).await;

    let _ = shared::services::alerting::AlertingService::send_webhook_alert(&format!(
        "⛔ KARANTİNA REDDEDİLDİ  {}\n{} · {}\nDosya hatali/ altına taşındı.",
        item.meta.id,
        item.meta.city.as_deref().unwrap_or("şehirsiz"),
        item.meta.file
    ))
    .await;

    Ok(format!(
        "⛔ *Reddedildi* `{}`\n• {} hatali/ altına taşındı.",
        item.meta.id, item.meta.file
    ))
}

/// `/ata <id> <sehir>`: şehirsiz/bilinmeyen şehirli öğeye geçerli şehir atar
/// ve dosyayı `bekleyen/<sehir>/` altına geri taşır (sonraki döngü işler).
pub async fn assign_quarantine_item(
    db: &DatabaseConnection,
    id: &str,
    city_slug: &str,
) -> Result<String> {
    let base = quarantine::menu_base_dir();
    let item = quarantine::find_item(&base, id)
        .await
        .ok_or_else(|| anyhow::anyhow!("'{}' kimlikli karantina öğesi bulunamadı.", id))?;

    let slug = city_slug.trim().to_lowercase();
    cities::Entity::find()
        .filter(cities::Column::Slug.eq(&slug))
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("'{}' şehri cities tablosunda yok.", slug))?;

    let dest_dir = PathBuf::from(&base)
        .join(&item.meta.role)
        .join("bekleyen")
        .join(&slug);
    tokio::fs::create_dir_all(&dest_dir).await?;
    let dest = dest_dir.join(&item.meta.file);
    quarantine::move_file(&item.file_path, &dest).await?;
    let _ = tokio::fs::remove_file(&item.meta_path).await;

    Ok(format!(
        "🏙 *Şehir atandı* `{}`\n• {} → bekleyen/{}/\nDosya işleme kuyruğuna geri alındı; birazdan işlenecek.",
        item.meta.id, item.meta.file, slug
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::models::{DayData, MenuComponent, MenuItem};

    fn item(name: &str) -> MenuItem {
        MenuItem {
            takeaway_id: None,
            alternatives: vec![MenuComponent::from(name)],
        }
    }

    fn day(breakfast: usize, lunch: usize, dinner: usize) -> DayData {
        let mut d = DayData::default();
        d.normal.breakfast = (0..breakfast).map(|i| item(&format!("b{}", i))).collect();
        d.normal.lunch = (0..lunch).map(|i| item(&format!("l{}", i))).collect();
        d.normal.dinner = (0..dinner).map(|i| item(&format!("d{}", i))).collect();
        d
    }

    fn db_with_dates(
        dates: &[&str],
        breakfast: usize,
        lunch: usize,
        dinner: usize,
    ) -> MenuDatabase {
        let mut db = MenuDatabase::new();
        for dt in dates {
            db.insert((*dt).to_string(), day(breakfast, lunch, dinner));
        }
        db
    }

    fn full_month(year: i32, month: u32) -> Vec<String> {
        let days = calendar_days_in_month(year, month);
        (1..=days)
            .map(|d| format!("{:04}-{:02}-{:02}", year, month, d))
            .collect()
    }

    fn parsed(file_name: &str, db: &MenuDatabase) -> ParsedFile {
        build_parsed_file(file_name, db, ParseDiagnostics::default())
    }

    #[test]
    fn test_build_parsed_file_counts() {
        let db = db_with_dates(&["2026-09-01", "2026-09-02"], 2, 0, 3);
        let p = parsed("Eylul.xlsx", &db);
        assert_eq!(p.dates.len(), 2);
        assert_eq!(p.breakfast, 2);
        assert_eq!(p.lunch, 0);
        assert_eq!(p.dinner, 2);
    }

    /// Satır 3: 0 gün dönen dosya kalıcı hatadır (D-2).
    #[test]
    fn test_classify_no_dates_is_permanent() {
        let db = MenuDatabase::new();
        let (outcome, _) = classify_ingest(&parsed("Bos.xlsx", &db), &GateConfig::default());
        assert_eq!(
            outcome,
            IngestOutcome::Permanent {
                reason: ReasonCode::NoDates
            }
        );
    }

    /// Satır 5: Haziran vakası — 2026-06 + 2026-05 karışımı MULTI_MONTH_SCOPE.
    #[test]
    fn test_classify_multi_month_scope() {
        let mut dates = full_month(2026, 6);
        dates.push("2026-05-04".to_string());
        let refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let db = db_with_dates(&refs, 0, 0, 3);
        let (outcome, scope) = classify_ingest(
            &parsed("Haziran_Ayı_Menüleri.xlsx", &db),
            &GateConfig::default(),
        );
        assert_eq!(
            outcome,
            IngestOutcome::Suspect {
                reason: ReasonCode::MultiMonthScope
            }
        );
        assert_eq!(scope.scope_month, Some((2026, 6)));
        assert_eq!(
            scope.stray_dates,
            vec![NaiveDate::from_ymd_opt(2026, 5, 4).unwrap()]
        );
    }

    /// Satır 9: 5 günlük Nisan parçası (5/30 < 0.6) şüpheli düşük kapsama.
    #[test]
    fn test_classify_five_day_april_is_suspect_low_coverage() {
        let db = db_with_dates(
            &[
                "2026-04-01",
                "2026-04-02",
                "2026-04-03",
                "2026-04-04",
                "2026-04-05",
            ],
            0,
            0,
            3,
        );
        let (outcome, scope) = classify_ingest(
            &parsed("Nisan_2026_Akşam_Yemeği_LLM.xlsx", &db),
            &GateConfig::default(),
        );
        assert_eq!(
            outcome,
            IngestOutcome::Suspect {
                reason: ReasonCode::LowCoverage
            }
        );
        assert_eq!(scope.in_scope_days, 5);
        assert_eq!(scope.expected_days, Some(30));
    }

    /// Satır 8: 20/30 gün (>= 0.6) kısmi düşük kapsama — ikisi de karar ister.
    #[test]
    fn test_classify_partial_low_coverage() {
        let dates: Vec<String> = (1..=20).map(|d| format!("2026-09-{:02}", d)).collect();
        let refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let db = db_with_dates(&refs, 1, 0, 2);
        let (outcome, _) = classify_ingest(&parsed("menu.xlsx", &db), &GateConfig::default());
        assert_eq!(
            outcome,
            IngestOutcome::Partial {
                reason: ReasonCode::LowCoverage
            }
        );
    }

    /// Satır 7: 31 gün Mart tam -> Complete.
    #[test]
    fn test_classify_full_march_is_complete() {
        let dates = full_month(2026, 3);
        let refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let db = db_with_dates(&refs, 1, 0, 2);
        let (outcome, scope) = classify_ingest(
            &parsed("Mart_2026_Menüleri.xlsx", &db),
            &GateConfig::default(),
        );
        assert_eq!(outcome, IngestOutcome::Complete);
        assert_eq!(scope.scope_month, Some((2026, 3)));
        assert!(scope.stray_dates.is_empty());
    }

    /// Tatil toleransı: WORKER_ALLOW_MISSING_DAYS kadar eksik Complete sayılır.
    #[test]
    fn test_classify_allow_missing_days_tolerance() {
        let mut dates = full_month(2026, 9);
        dates.pop();
        dates.pop();
        let refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let db = db_with_dates(&refs, 1, 0, 2);
        let cfg = GateConfig {
            allow_missing_days: 2,
            ..Default::default()
        };
        let (outcome, _) = classify_ingest(&parsed("menu.xlsx", &db), &cfg);
        assert_eq!(outcome, IngestOutcome::Complete);
    }

    /// Satır 6: dosya adı Mayıs diyor, veri Haziran -> DECLARED_MONTH_MISMATCH.
    #[test]
    fn test_classify_declared_month_mismatch() {
        let dates = full_month(2026, 6);
        let refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let db = db_with_dates(&refs, 1, 0, 2);
        let (outcome, _) = classify_ingest(
            &parsed("Mayis_2026_Menüleri.xlsx", &db),
            &GateConfig::default(),
        );
        assert_eq!(
            outcome,
            IngestOutcome::Suspect {
                reason: ReasonCode::DeclaredMonthMismatch
            }
        );
    }

    /// K-5: ismi işe yaramaz dosyada kapsam VERİDEN çıkar; dosya adı engel değil.
    #[test]
    fn test_classify_scope_from_data_with_useless_filename() {
        let dates = full_month(2026, 9);
        let refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let db = db_with_dates(&refs, 1, 0, 2);
        let (outcome, scope) = classify_ingest(
            &parsed("YENI_LISTE_SON_V2(1).xlsx", &db),
            &GateConfig::default(),
        );
        assert_eq!(outcome, IngestOutcome::Complete);
        assert_eq!(scope.scope_month, Some((2026, 9)));
    }

    /// K-8 katı mod: beyan edilmemiş ay şüphelidir.
    #[test]
    fn test_classify_undeclared_month_strict() {
        let dates = full_month(2026, 9);
        let refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let db = db_with_dates(&refs, 1, 0, 2);
        let cfg = GateConfig {
            require_declared_month: true,
            ..Default::default()
        };
        let (outcome, _) = classify_ingest(&parsed("YENI_LISTE.xlsx", &db), &cfg);
        assert_eq!(
            outcome,
            IngestOutcome::Suspect {
                reason: ReasonCode::UndeclaredMonth
            }
        );
    }

    /// Satır 4: tarih sırası çelişkisi -> AMBIGUOUS_DATE_ORDER.
    #[test]
    fn test_classify_ambiguous_date_order() {
        let db = db_with_dates(&["2026-06-01", "2026-06-02"], 0, 0, 3);
        let mut p = parsed("menu.xlsx", &db);
        p.diagnostics.date_order = Some(DateOrderResolution::Conflict("test".to_string()));
        let (outcome, _) = classify_ingest(&p, &GateConfig::default());
        assert_eq!(
            outcome,
            IngestOutcome::Suspect {
                reason: ReasonCode::AmbiguousDateOrder
            }
        );
    }

    /// Satır 4: zayıf sıra kanıtı -> WEAK_DATE_ORDER.
    #[test]
    fn test_classify_weak_date_order() {
        let db = db_with_dates(&["2026-06-05"], 0, 0, 3);
        let mut p = parsed("menu.xlsx", &db);
        p.diagnostics.date_order = Some(DateOrderResolution::Weak(
            crate::parser::core::DateTokenOrder::DayMonth,
        ));
        let (outcome, _) = classify_ingest(&p, &GateConfig::default());
        assert_eq!(
            outcome,
            IngestOutcome::Suspect {
                reason: ReasonCode::WeakDateOrder
            }
        );
    }

    /// Satır 4: LLM ISO / ham tarih uyuşmazlığı -> DATE_ORDER_MISMATCH.
    #[test]
    fn test_classify_date_raw_mismatch() {
        let db = db_with_dates(&["2026-06-01", "2026-06-02"], 0, 0, 3);
        let mut p = parsed("menu.pdf", &db);
        p.diagnostics.date_raw_mismatches = vec!["01.06.2026 != 2026-01-06".to_string()];
        let (outcome, _) = classify_ingest(&p, &GateConfig::default());
        assert_eq!(
            outcome,
            IngestOutcome::Suspect {
                reason: ReasonCode::DateOrderMismatch
            }
        );
    }

    /// Onay terfisi kapsam dışı tarihi asla yazmaz (plan 4.4 dipnot).
    #[test]
    fn test_filter_to_scope_drops_stray_dates() {
        let db = db_with_dates(&["2026-06-01", "2026-06-02", "2026-05-04"], 0, 0, 3);
        let (kept, dropped) = filter_to_scope(db, (2026, 6));
        assert_eq!(kept.len(), 2);
        assert!(kept.contains_key("2026-06-01"));
        assert!(!kept.contains_key("2026-05-04"));
        assert_eq!(dropped, vec!["2026-05-04".to_string()]);
    }

    /// Çok günlük belgede yalnız tek öğün tipi varsa öğün-karışımı uyarısı verilmeli.
    #[test]
    fn test_meal_mix_warning_single_meal() {
        let dates: Vec<String> = (1..=20).map(|d| format!("2026-09-{:02}", d)).collect();
        let refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let db = db_with_dates(&refs, 0, 0, 3);
        let p = parsed("menu.xlsx", &db);
        let warning = meal_mix_warning(&p).expect("öğün-karışımı uyarısı bekleniyor");
        assert!(warning.contains("tek öğün"));
    }

    /// Tam ay çeşitliliğinde (kahvaltı + akşam) uyarı OLMAMALI.
    #[test]
    fn test_meal_mix_warning_absent_for_mixed_month() {
        let dates = full_month(2026, 9);
        let refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let db = db_with_dates(&refs, 1, 0, 2);
        let p = parsed("menu.xlsx", &db);
        assert!(meal_mix_warning(&p).is_none());
    }

    /// finalize_file vault hedefi: dosya vault/<ext>/<rol>/<sehir>/ altına taşınır.
    #[tokio::test]
    async fn test_finalize_file_moves_to_vault() {
        let base = std::env::temp_dir().join(format!("kepce_vault_{}", uuid::Uuid::new_v4()));
        let city_dir = base.join("admin").join("bekleyen").join("istanbul");
        tokio::fs::create_dir_all(&city_dir).await.unwrap();
        let src = city_dir.join("Mart_2026.xlsx");
        tokio::fs::write(&src, b"icerik").await.unwrap();

        finalize_file(
            base.to_str().unwrap(),
            "admin",
            &src,
            FinalDest::Vault {
                ext: "xlsx",
                city_slug: "istanbul",
            },
        )
        .await;

        assert!(!src.exists(), "kaynak taşınmış olmalı");
        let vault_dir = base
            .join("vault")
            .join("xlsx")
            .join("admin")
            .join("istanbul");
        let mut entries = tokio::fs::read_dir(&vault_dir).await.unwrap();
        let mut found = false;
        while let Ok(Some(e)) = entries.next_entry().await {
            if e.file_name().to_string_lossy().ends_with("Mart_2026.xlsx") {
                found = true;
            }
        }
        assert!(found, "dosya vault'ta zaman damgalı adla durmalı");
        let _ = tokio::fs::remove_dir_all(&base).await;
    }

    /// finalize_file hatali hedefi: dosya <rol>/hatali/ altına taşınır.
    #[tokio::test]
    async fn test_finalize_file_moves_to_hatali() {
        let base = std::env::temp_dir().join(format!("kepce_hatali_{}", uuid::Uuid::new_v4()));
        let city_dir = base.join("admin").join("bekleyen").join("istanbul");
        tokio::fs::create_dir_all(&city_dir).await.unwrap();
        let src = city_dir.join("Bozuk.xlsx");
        tokio::fs::write(&src, b"bozuk").await.unwrap();

        finalize_file(base.to_str().unwrap(), "admin", &src, FinalDest::Hatali).await;

        assert!(!src.exists());
        assert!(
            base.join("admin")
                .join("hatali")
                .join("Bozuk.xlsx")
                .exists()
        );
        let _ = tokio::fs::remove_dir_all(&base).await;
    }

    /// Faz 2.1 (D-1) entegrasyon: bekleyen/ kökündeki dosya karantinaya düşer.
    ///
    /// `DatabaseConnection::Disconnected` kalıbıyla veritabanı gerektirmeden
    /// çalışır; kök dosya şehir sorgusundan ÖNCE karantinaya alınır.
    #[tokio::test]
    async fn test_root_file_goes_to_quarantine_no_city() {
        let base = std::env::temp_dir().join(format!("kepce_root_{}", uuid::Uuid::new_v4()));
        let bekleyen = base.join("admin").join("bekleyen");
        tokio::fs::create_dir_all(&bekleyen).await.unwrap();
        let root_file = bekleyen.join("Sahipsiz_Dosya.xlsx");
        tokio::fs::write(&root_file, b"veri").await.unwrap();

        unsafe {
            env::set_var("WORKER_MENU_DIR", base.to_str().unwrap());
        }

        let db = DatabaseConnection::Disconnected;
        let client = reqwest::Client::new();
        // Şehir sorguları başarısız olur ama kök dosya karantinası db istemez.
        let _ = process_local_files(&db, &client, None).await;

        assert!(!root_file.exists(), "kök dosya karantinaya taşınmalı");
        let q = quarantine::quarantine_root(base.to_str().unwrap(), "admin");
        assert!(q.join("Sahipsiz_Dosya.xlsx").exists());
        let items = quarantine::list_queue(base.to_str().unwrap()).await;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].meta.reason_code, ReasonCode::NoCity);
        assert!(items[0].meta.city.is_none());

        unsafe {
            env::remove_var("WORKER_MENU_DIR");
        }
        let _ = tokio::fs::remove_dir_all(&base).await;
    }

    #[test]
    fn test_classify_weekday_mismatch() {
        let dates = full_month(2026, 5);
        let strs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let db = db_with_dates(&strs, 3, 0, 4);
        let mut p = parsed("Mayis.xlsx", &db);
        p.diagnostics.weekday_mismatches =
            vec!["2026-05-04 (Perşembe) != başlık Pazartesi".to_string()];
        let cfg = GateConfig::default();
        let (outcome, _) = classify_ingest(&p, &cfg);
        match outcome {
            IngestOutcome::Suspect { reason } => {
                assert_eq!(reason, ReasonCode::WeekdayMismatch);
            }
            other => panic!("beklenen WeekdayMismatch, alınan {:?}", other),
        }
    }

    #[test]
    fn test_classify_contiguous_weekly_span_cross_month() {
        // 28 Eylül - 4 Ekim (7 günlük kesintisiz ay geçişi haftalık menüsü)
        let dates = vec![
            "2026-09-28",
            "2026-09-29",
            "2026-09-30",
            "2026-10-01",
            "2026-10-02",
            "2026-10-03",
            "2026-10-04",
        ];
        let db = db_with_dates(&dates, 3, 0, 4);
        let p = parsed("Haftalik_Menu.xlsx", &db);
        let cfg = GateConfig::default();
        let (outcome, scope) = classify_ingest(&p, &cfg);
        assert_eq!(outcome, IngestOutcome::Complete);
        assert_eq!(scope.expected_days, Some(7));
        assert_eq!(scope.in_scope_days, 7);
        assert!(scope.stray_dates.is_empty());
    }
}
