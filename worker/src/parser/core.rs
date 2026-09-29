use crate::parser::models::{DayData, DayMetadata, MenuComponent, MenuDatabase, MenuItem};
use crate::parser::validation;
use crate::parser::validation::MealType;
use chrono::Datelike;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SheetGrid {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTokenOrder {
    /// p1 is Day (1..=31), p2 is Month (1..=12) -> Standart Türkiye formatı DD.MM.YYYY
    DayMonth,
    /// p1 is Month (1..=12), p2 is Day (1..=31) -> Amerikan formatı MM.DD.YYYY
    MonthDay,
}

/// Dosya bazlı tarih sırası çözümünün sonucu (deterministik, T1..T5 kuralları).
///
/// `Weak` karar "kesin değil" anlamına gelir: tek tarihli veya tek biçimli
/// dosyada sıra kanıtlanamamıştır ve çağıran taraf bunu şüphe olarak kaydeder.
/// `Conflict` durumunda dosya karantinaya alınmalıdır.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateOrderResolution {
    /// Sıra kanıtla belirlendi.
    Resolved(DateTokenOrder),
    /// Sıra varsayılan (DMY) ama kanıt zayıf; karantina kararı gerektirir.
    Weak(DateTokenOrder),
    /// Çelişki: hem gün hem ay konumu için kanıt var ya da çapraz doğrulama
    /// iki okumayı da destekliyor/desteklemiyor.
    Conflict(String),
}

impl DateOrderResolution {
    /// Çözümün temsil ettiği token sırası (Conflict'te Türkiye standardı DMY).
    pub fn order_or_default(&self) -> DateTokenOrder {
        match self {
            DateOrderResolution::Resolved(o) | DateOrderResolution::Weak(o) => *o,
            DateOrderResolution::Conflict(_) => DateTokenOrder::DayMonth,
        }
    }
}

/// Ayrıştırma sırasında toplanan şüphe sinyalleri.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ParseDiagnostics {
    /// Dosya bazlı tarih sırası çözümü (Excel yolu).
    pub date_order: Option<DateOrderResolution>,
    /// Tarih biçimine benzeyen ama seçilen sırayla geçerli tarih üretmeyen hücreler.
    pub unparseable_dates: Vec<String>,
    /// Sütun başlığındaki gün adı ile çıkarılan tarihin haftanın günü uyuşmazlıkları.
    pub weekday_mismatches: Vec<String>,
    /// LLM'in ISO tarihi ile ham yazılı tarihin (`date_raw`) deterministik uyuşmazlıkları.
    pub date_raw_mismatches: Vec<String>,
}

/// Dosya adı/sayfa adı gibi metinlerden çıkarılan beyan edilen ay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeclaredMonth {
    pub year: Option<i32>,
    pub month: u32,
}

/// Tarih benzeri metinleri yakalayan ortak regex (p1, p2, yıl).
fn date_like_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"(\d{1,2})[\./-](\d{1,2})[\./-](\d{4})").unwrap())
}

/// Türkçe metinleri büyük harfe ve ASCII varyantlarına normalleştiren yardımcı fonksiyon.
fn normalize_turkish_text(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'ç' | 'Ç' => 'C',
            'ğ' | 'Ğ' => 'G',
            'ı' | 'I' | 'İ' | 'i' => 'I',
            'ö' | 'Ö' => 'O',
            'ş' | 'Ş' => 'S',
            'ü' | 'Ü' => 'U',
            other => other.to_ascii_uppercase(),
        })
        .collect()
}

pub fn extract_turkish_month(text: &str) -> Option<u32> {
    let normalized = normalize_turkish_text(text);
    if normalized.contains("OCAK") {
        Some(1)
    } else if normalized.contains("SUBAT") {
        Some(2)
    } else if normalized.contains("MART") {
        Some(3)
    } else if normalized.contains("NISAN") {
        Some(4)
    } else if normalized.contains("MAYIS") {
        Some(5)
    } else if normalized.contains("HAZIRAN") {
        Some(6)
    } else if normalized.contains("TEMMUZ") {
        Some(7)
    } else if normalized.contains("AGUSTOS") {
        Some(8)
    } else if normalized.contains("EYLUL") {
        Some(9)
    } else if normalized.contains("EKIM") {
        Some(10)
    } else if normalized.contains("KASIM") {
        Some(11)
    } else if normalized.contains("ARALIK") {
        Some(12)
    } else {
        None
    }
}

/// Metinden beyan edilen ayı çıkarır (Türkçe ay adı + olası 20xx yılı).
pub fn extract_declared_month(text: &str) -> Option<DeclaredMonth> {
    let month = extract_turkish_month(text)?;
    static RE_YEAR: OnceLock<regex::Regex> = OnceLock::new();
    let re_year = RE_YEAR.get_or_init(|| regex::Regex::new(r"(20\d{2})").unwrap());
    let year = re_year
        .captures(text)
        .and_then(|c| c[1].parse::<i32>().ok());
    Some(DeclaredMonth { year, month })
}

/// Dosya bazlı, deterministik ve çelişki tespit edici tarih sırası çözümü.
/// Haftalık ay geçişleri (örn: 28.09 - 02.10) ve T1..T5 kurallarını destekler.
pub fn resolve_file_date_order(grids: &[SheetGrid], file_name_hint: &str) -> DateOrderResolution {
    let re = date_like_regex();

    let mut p1_gt_12: Option<String> = None;
    let mut p2_gt_12: Option<String> = None;
    let mut p1_values = std::collections::HashSet::new();
    let mut p2_values = std::collections::HashSet::new();
    let mut token_count = 0usize;

    for grid in grids {
        for row in &grid.rows {
            for cell in row {
                // Eğer hücrede birden fazla tarih varsa (tarih aralığı), atla
                if re.captures_iter(cell).count() > 1 {
                    continue;
                }

                if let Some(caps) = re.captures(cell)
                    && let (Ok(p1), Ok(p2)) = (caps[1].parse::<u32>(), caps[2].parse::<u32>())
                    && (1..=31).contains(&p1)
                    && (1..=31).contains(&p2)
                {
                    token_count += 1;
                    if p1 > 12 && p1_gt_12.is_none() {
                        p1_gt_12 = Some(cell.trim().to_string());
                    }
                    if p2 > 12 && p2_gt_12.is_none() {
                        p2_gt_12 = Some(cell.trim().to_string());
                    }
                    p1_values.insert(p1);
                    p2_values.insert(p2);
                }
            }
        }
    }

    if token_count == 0 {
        return DateOrderResolution::Resolved(DateTokenOrder::DayMonth);
    }

    // T1: Çapa kuralı (>12 olan sadece gün olabilir)
    match (&p1_gt_12, &p2_gt_12) {
        (Some(_), None) => return DateOrderResolution::Resolved(DateTokenOrder::DayMonth),
        (None, Some(_)) => return DateOrderResolution::Resolved(DateTokenOrder::MonthDay),
        (Some(a), Some(b)) => {
            return DateOrderResolution::Conflict(format!(
                "Hem gün-önde hem ay-önde kanıt var: '{}' (p1>12) ve '{}' (p2>12)",
                a, b
            ));
        }
        (None, None) => {}
    }

    // Beyan edilen ay bilgisi (T4 ve çoklu ay çözümü için)
    let declared = grids
        .iter()
        .find_map(|g| extract_declared_month(&g.name))
        .or_else(|| extract_declared_month(file_name_hint));

    // T2: Sabitlik kuralı & Ay Geçişleri
    let p1_multi = p1_values.len() > 1;
    let p2_multi = p2_values.len() > 1;

    match (p1_multi, p2_multi) {
        (true, false) => return DateOrderResolution::Resolved(DateTokenOrder::DayMonth),
        (false, true) => return DateOrderResolution::Resolved(DateTokenOrder::MonthDay),
        (true, true) => {
            // Hem p1 hem p2 birden fazla değere sahip.
            // Bu durum haftalık menülerdeki ay sonu/başı geçişlerinde (örn. 30.04 - 04.05) görülebilir.
            let p2_all_valid_months = p2_values.iter().all(|&m| (1..=12).contains(&m));
            let p1_all_valid_months = p1_values.iter().all(|&m| (1..=12).contains(&m));

            // Tipik ay geçişi: p2 en fazla 2 ardışık ay iken p1 belirgin şekilde gün dağılımı gösterir.
            if p2_all_valid_months && p2_values.len() <= 2 && p1_values.len() > p2_values.len() {
                return DateOrderResolution::Resolved(DateTokenOrder::DayMonth);
            }
            if p1_all_valid_months && p1_values.len() <= 2 && p2_values.len() > p1_values.len() {
                return DateOrderResolution::Resolved(DateTokenOrder::MonthDay);
            }

            // Eğer beyan edilen ay varsa, çelişkiyi çözmeyi dene
            if let Some(dm) = declared {
                let dmy_matches = p2_values.contains(&dm.month);
                let mdy_matches = p1_values.contains(&dm.month);
                if dmy_matches && !mdy_matches {
                    return DateOrderResolution::Resolved(DateTokenOrder::DayMonth);
                } else if !dmy_matches && mdy_matches {
                    return DateOrderResolution::Resolved(DateTokenOrder::MonthDay);
                }
            }

            return DateOrderResolution::Conflict(format!(
                "Hem p1 hem p2 çok değerli: p1={:?}, p2={:?}. Sıra kanıtlanamaz.",
                sorted(&p1_values),
                sorted(&p2_values)
            ));
        }
        (false, false) => {}
    }

    // T3: Tek tarih / Tek biçim
    let p1 = *p1_values.iter().next().unwrap_or(&0);
    let p2 = *p2_values.iter().next().unwrap_or(&0);
    if p1 == p2 {
        return DateOrderResolution::Resolved(DateTokenOrder::DayMonth);
    }

    // T4: Ay çapraz doğrulaması (tek değerli durumlar için)
    if let Some(dm) = declared {
        let dmy_matches = p2 == dm.month;
        let mdy_matches = p1 == dm.month;
        match (dmy_matches, mdy_matches) {
            (true, false) => return DateOrderResolution::Resolved(DateTokenOrder::DayMonth),
            (false, true) => return DateOrderResolution::Resolved(DateTokenOrder::MonthDay),
            (true, true) => {
                return DateOrderResolution::Conflict(format!(
                    "Beyan edilen ay {} her iki okumayla da uyuşuyor, sıra kanıtlanamaz.",
                    dm.month
                ));
            }
            (false, false) => {
                return DateOrderResolution::Conflict(format!(
                    "Beyan edilen ay {} hiçbir okumayla uyuşmuyor (p1={}, p2={}).",
                    dm.month, p1, p2
                ));
            }
        }
    }

    // T3 sonucu: Kesin değil, Türkiye standardı DMY ama Weak etiketli.
    DateOrderResolution::Weak(DateTokenOrder::DayMonth)
}

fn sorted(set: &std::collections::HashSet<u32>) -> Vec<u32> {
    let mut v: Vec<u32> = set.iter().copied().collect();
    v.sort_unstable();
    v
}

/// Metin tarihini verilen token sırasıyla ISO'ya çevirir.
/// Eğer hücrede birden fazla tarih varsa (tarih aralığı), None döner.
pub fn parse_date_with_order(s: &str, order: DateTokenOrder) -> Option<String> {
    let re = date_like_regex();
    if re.captures_iter(s).count() > 1 {
        return None;
    }

    let caps = re.captures(s)?;
    let p1: u32 = caps[1].parse().ok()?;
    let p2: u32 = caps[2].parse().ok()?;
    let year: i32 = caps[3].parse().ok()?;

    let (day, month) = match order {
        DateTokenOrder::DayMonth => (p1, p2),
        DateTokenOrder::MonthDay => (p2, p1),
    };

    chrono::NaiveDate::from_ymd_opt(year, month, day).map(|d| d.format("%Y-%m-%d").to_string())
}

/// Türkçe gün adı başlığından chrono haftanın günü eşlemesi.
fn weekday_from_header(text: &str) -> Option<chrono::Weekday> {
    use chrono::Weekday;
    let norm = normalize_turkish_text(text);

    // Uzun kalıplar önce kontrol edilmeli: PAZARTESI, PAZAR'dan önce gelmeli
    if norm.contains("PAZARTESI") {
        Some(Weekday::Mon)
    } else if norm.contains("SALI") {
        Some(Weekday::Tue)
    } else if norm.contains("CARSAMBA") {
        Some(Weekday::Wed)
    } else if norm.contains("PERSEMBE") {
        Some(Weekday::Thu)
    } else if norm.contains("CUMARTESI") {
        Some(Weekday::Sat)
    } else if norm.contains("CUMA") {
        Some(Weekday::Fri)
    } else if norm.contains("PAZAR") {
        Some(Weekday::Sun)
    } else {
        None
    }
}

pub fn parse_date_string(s: &str) -> Option<String> {
    parse_date_with_order(s, DateTokenOrder::DayMonth)
}

/// Hücrenin belirlenen token sırasına göre bir tarih içerip içermediğini kontrol eder.
fn is_date_cell(s: &str, order: DateTokenOrder) -> bool {
    parse_date_with_order(s, order).is_some()
}

/// Parantez içindeki ayıraçları yok sayarak string'i böler.
pub fn split_outside_parens(s: &str, sep: char) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut round_parens: usize = 0;
    let mut square_brackets: usize = 0;
    let mut curly_braces: usize = 0;

    for c in s.chars() {
        match c {
            '(' => {
                round_parens += 1;
                current.push(c);
            }
            ')' => {
                round_parens = round_parens.saturating_sub(1);
                current.push(c);
            }
            '[' => {
                square_brackets += 1;
                current.push(c);
            }
            ']' => {
                square_brackets = square_brackets.saturating_sub(1);
                current.push(c);
            }
            '{' => {
                curly_braces += 1;
                current.push(c);
            }
            '}' => {
                curly_braces = curly_braces.saturating_sub(1);
                current.push(c);
            }
            x if x == sep && round_parens == 0 && square_brackets == 0 && curly_braces == 0 => {
                result.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }
    result
}

/// Sayfa bazlı eski arayüzü koruyan geriye dönük uyumluluk fonksiyonu.
pub fn parse_grid(sheet: &SheetGrid, db: &mut MenuDatabase, file_name_hint: &str) {
    let resolution = resolve_file_date_order(std::slice::from_ref(sheet), file_name_hint);
    let mut diag = ParseDiagnostics::default();
    parse_grid_with_order(
        sheet,
        db,
        file_name_hint,
        resolution.order_or_default(),
        &mut diag,
    );
}

/// Verilen token sırasıyla sayfayı ayrıştırır ve şüphe sinyallerini `diag`'a yazar.
pub fn parse_grid_with_order(
    sheet: &SheetGrid,
    db: &mut MenuDatabase,
    file_name_hint: &str,
    date_order: DateTokenOrder,
    diag: &mut ParseDiagnostics,
) {
    static RE_KCAL: OnceLock<regex::Regex> = OnceLock::new();
    let re_kcal = RE_KCAL
        .get_or_init(|| regex::Regex::new(r"(?i)(\d+\s*-\s*\d+)\s*(?:kcal|kkal|kalori)").unwrap());
    static RE_NUMS: OnceLock<regex::Regex> = OnceLock::new();
    let re_nums = RE_NUMS.get_or_init(|| regex::Regex::new(r"(\d+)").unwrap());

    let is_colyak = validation::is_colyak_sheet(&sheet.name);
    let mut meal_type_opt = validation::detect_meal_type(&sheet.name, &[]);

    let height = sheet.rows.len();
    if height == 0 {
        return;
    }

    // Öğün türü sayfa adından çıkarılamadıysa ilk birkaç yemekten tespit et
    if meal_type_opt.is_none() {
        let mut sample_items: Vec<String> = Vec::new();
        'detect: for r in 0..height {
            for c in 0..sheet.rows[r].len() {
                let cell = &sheet.rows[r][c];
                if parse_date_with_order(cell, date_order).is_some() {
                    let mut item_row = r + 1;
                    let mut empty_count = 0;
                    while item_row < height && empty_count < 2 {
                        let item_row_len = sheet.rows[item_row].len();
                        if c >= item_row_len {
                            empty_count += 1;
                            item_row += 1;
                            continue;
                        }
                        let raw_str = &sheet.rows[item_row][c];
                        if is_date_cell(raw_str, date_order) {
                            break;
                        }
                        let item_name = raw_str
                            .trim()
                            .trim_start_matches('*')
                            .trim_start_matches('-')
                            .trim_start_matches('•')
                            .trim_start_matches('⁃')
                            .trim()
                            .to_string();
                        if item_name.is_empty() {
                            empty_count += 1;
                        } else {
                            empty_count = 0;
                            sample_items.push(item_name);
                            if sample_items.len() >= 10 {
                                break 'detect;
                            }
                        }
                        item_row += 1;
                    }
                }
            }
        }
        if !sample_items.is_empty() {
            meal_type_opt = validation::detect_meal_type("", &sample_items);
        }
    }

    let resolved_meal_type = match meal_type_opt {
        Some(mt) => mt,
        None => {
            tracing::warn!(
                "SKIP sheet '{}': could not determine meal type from name or content",
                sheet.name
            );
            return;
        }
    };

    let mut sheet_kcal: Option<String> = None;
    let mut fallback_sheet_kcal: Option<String> = None;

    let meal_target_kw = match resolved_meal_type {
        MealType::Breakfast => "KAHVALT",
        MealType::Lunch => "ÖĞLE",
        MealType::Dinner => "AKŞAM",
    };

    for r in 0..height {
        for c in 0..sheet.rows[r].len() {
            let text = &sheet.rows[r][c];
            if let Some(caps) = re_kcal.captures(text) {
                let upper = text.to_uppercase();
                let val = format!("{} kcal", caps[1].replace(' ', ""));
                if upper.contains(meal_target_kw) {
                    sheet_kcal = Some(val);
                    break;
                } else if fallback_sheet_kcal.is_none() {
                    fallback_sheet_kcal = Some(val);
                }
            }
        }
        if sheet_kcal.is_some() {
            break;
        }
    }
    if sheet_kcal.is_none() {
        sheet_kcal = fallback_sheet_kcal;
    }

    for r in 0..height {
        let row_len = sheet.rows[r].len();
        for c in 0..row_len {
            let cell = &sheet.rows[r][c];

            if let Some(date_str) = parse_date_with_order(cell, date_order) {
                // Sütun başlığındaki gün adı ile tarihin haftanın günü uyuşması kontrolü
                if let Ok(parsed_date) = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                    for hr in (1..=2usize).rev() {
                        if r < hr {
                            break;
                        }
                        let header_row = &sheet.rows[r - hr];
                        if c >= header_row.len() {
                            continue;
                        }
                        if let Some(expected_wd) = weekday_from_header(&header_row[c]) {
                            if expected_wd != parsed_date.weekday() {
                                let msg = format!(
                                    "{} ({}) için başlıkta '{}' gün adı var ama tarih {} günü",
                                    date_str,
                                    cell.trim(),
                                    header_row[c].trim(),
                                    parsed_date.weekday()
                                );
                                tracing::warn!("WEEKDAY_MISMATCH: {}", msg);
                                if !diag.weekday_mismatches.contains(&msg) {
                                    diag.weekday_mismatches.push(msg);
                                }
                            }
                            break;
                        }
                    }
                }

                if !validation::validate_date_range(&date_str) {
                    tracing::warn!("SKIP date: {} is outside valid range (±2 years)", date_str);
                    continue;
                }

                let mut has_gramaj = false;
                let mut has_enerji = false;

                if c + 1 < row_len {
                    let v = sheet.rows[r][c + 1].to_uppercase();
                    if v.contains("GRAMAJ") {
                        has_gramaj = true;
                    }
                    if v.contains("ENERJİ") || v.contains("ENERJI") {
                        has_enerji = true;
                    }
                }
                if c + 2 < row_len && has_gramaj {
                    let v = sheet.rows[r][c + 2].to_uppercase();
                    if v.contains("ENERJİ") || v.contains("ENERJI") {
                        has_enerji = true;
                    }
                }

                // Yatay (Long) Format Kontrolü: Tarih ve yemeklerin aynı satırda olduğu tablolar
                let mut long_items: Vec<MenuItem> = Vec::new();
                {
                    let mut cc = c + 1;
                    while cc < row_len {
                        let cell_text = sheet.rows[r][cc].trim();
                        if cell_text.is_empty() || is_date_cell(cell_text, date_order) {
                            cc += 1;
                            continue;
                        }
                        let upper = cell_text.to_uppercase();
                        if upper.contains("GRAMAJ")
                            || upper.contains("ENERJİ")
                            || upper.contains("ENERJI")
                        {
                            cc += 1;
                            continue;
                        }
                        if let Some(name) = validation::validate_item_name(cell_text) {
                            let mut amount = None;
                            if cc + 1 < row_len {
                                amount = validation::validate_numeric_value(&sheet.rows[r][cc + 1]);
                            }

                            let mut alternatives = Vec::new();
                            let names: Vec<String> =
                                crate::parser::normalizer::split_smart_alternatives(&name);
                            for n in names {
                                alternatives.push(MenuComponent {
                                    name: n,
                                    amount: amount.clone(),
                                    calories: None,
                                    category: None,
                                });
                            }
                            if alternatives.is_empty() {
                                alternatives.push(MenuComponent {
                                    name: crate::parser::normalizer::normalize_food_name(&name),
                                    amount,
                                    calories: None,
                                    category: None,
                                });
                            }
                            long_items.push(MenuItem {
                                takeaway_id: None,
                                alternatives,
                            });
                            cc += 2; // yemek + olası gramaj hücresini atla
                        } else {
                            cc += 1;
                        }
                    }
                }

                if !long_items.is_empty() {
                    let day_data = db.entry(date_str.clone()).or_insert_with(|| DayData {
                        metadata: Some(DayMetadata {
                            trust_score: 100,
                            anomaly_score: Some(0.0),
                            status: "approved".to_string(),
                            source_file: Some(file_name_hint.to_string()),
                        }),
                        ..Default::default()
                    });
                    let menu_ref = if is_colyak {
                        &mut day_data.colyak
                    } else {
                        &mut day_data.normal
                    };

                    let target_list = match resolved_meal_type {
                        MealType::Breakfast => {
                            if menu_ref.breakfast_kcal.is_none() {
                                menu_ref.breakfast_kcal = sheet_kcal.clone();
                            }
                            &mut menu_ref.breakfast
                        }
                        MealType::Lunch => {
                            if menu_ref.lunch_kcal.is_none() {
                                menu_ref.lunch_kcal = sheet_kcal.clone();
                            }
                            &mut menu_ref.lunch
                        }
                        MealType::Dinner => {
                            if menu_ref.dinner_kcal.is_none() {
                                menu_ref.dinner_kcal = sheet_kcal.clone();
                            }
                            &mut menu_ref.dinner
                        }
                    };
                    target_list.append(&mut long_items);
                    continue;
                }

                // Dikey (Wide) Format: Tarihin altındaki satırların taranması
                let mut item_row = r + 1;
                let mut empty_count = 0;
                let mut item_count: usize = 0;

                while item_row < height {
                    let item_row_len = sheet.rows[item_row].len();
                    if c >= item_row_len {
                        empty_count += 1;
                        item_row += 1;
                        if empty_count >= 2 {
                            break;
                        }
                        continue;
                    }

                    let raw_str = &sheet.rows[item_row][c];
                    let raw_upper = raw_str.to_uppercase();

                    let is_takeaway = raw_upper.contains("AL GÖTÜR")
                        || raw_upper.contains("ALGÖTÜR")
                        || raw_upper.contains("AL-GÖTÜR")
                        || raw_upper.contains("AL GÖTUR")
                        || raw_upper.contains("ALGÖTUR");

                    let mut item_name = raw_str
                        .trim()
                        .trim_start_matches('*')
                        .trim_start_matches('-')
                        .trim_start_matches('•')
                        .trim_start_matches('⁃')
                        .trim()
                        .to_string();

                    if item_name.contains("Cay") {
                        item_name = item_name.replace("Cay", "Çay");
                    } else if item_name.contains("cay") {
                        item_name = item_name.replace("cay", "çay");
                    }

                    let item_name = match validation::validate_item_name(&item_name) {
                        Some(name) => name,
                        None if !item_name.trim().is_empty() => {
                            tracing::warn!(
                                "SKIP item: name too long ({} chars): {}...",
                                item_name.len(),
                                &item_name[..50.min(item_name.len())]
                            );
                            item_row += 1;
                            continue;
                        }
                        None => item_name,
                    };

                    let mut amount_str = String::new();
                    if has_gramaj && c + 1 < item_row_len {
                        amount_str = sheet.rows[item_row][c + 1].clone();
                    }

                    let lower_name = item_name.to_lowercase();
                    let lower_amt = amount_str.to_lowercase();
                    if lower_name.contains("hazırlanıp")
                        || lower_name.contains("sunulacaktır")
                        || lower_name.contains("garnitür")
                        || lower_name.contains("ortalama")
                        || lower_amt.contains("hazırlanıp")
                        || lower_amt.contains("sunulacaktır")
                        || lower_amt.contains("garnitür")
                        || lower_amt.contains("ortalama")
                    {
                        item_row += 1;
                        continue;
                    }

                    if is_date_cell(raw_str, date_order) || empty_count >= 2 {
                        break;
                    }

                    if item_name.is_empty() {
                        empty_count += 1;
                        item_row += 1;
                        continue;
                    } else {
                        empty_count = 0;
                    }

                    item_count += 1;
                    if !validation::validate_meal_item_count(item_count) {
                        tracing::warn!(
                            "WARN: meal for {} exceeded max items ({}), truncating",
                            date_str,
                            item_count
                        );
                        break;
                    }

                    let day_data = db.entry(date_str.clone()).or_insert_with(|| DayData {
                        metadata: Some(DayMetadata {
                            trust_score: 100,
                            anomaly_score: Some(0.0),
                            status: "approved".to_string(),
                            source_file: Some(file_name_hint.to_string()),
                        }),
                        ..Default::default()
                    });
                    let menu_ref = if is_colyak {
                        &mut day_data.colyak
                    } else {
                        &mut day_data.normal
                    };

                    let target_list = match resolved_meal_type {
                        MealType::Breakfast => {
                            if menu_ref.breakfast_kcal.is_none() {
                                menu_ref.breakfast_kcal = sheet_kcal.clone();
                            }
                            &mut menu_ref.breakfast
                        }
                        MealType::Lunch => {
                            if menu_ref.lunch_kcal.is_none() {
                                menu_ref.lunch_kcal = sheet_kcal.clone();
                            }
                            &mut menu_ref.lunch
                        }
                        MealType::Dinner => {
                            if menu_ref.dinner_kcal.is_none() {
                                menu_ref.dinner_kcal = sheet_kcal.clone();
                            }
                            &mut menu_ref.dinner
                        }
                    };

                    if is_takeaway {
                        let mut has_nums = false;
                        for cap in re_nums.captures_iter(&item_name) {
                            has_nums = true;
                            let num = &cap[1];
                            let num_str = num.to_string();
                            if !target_list
                                .iter()
                                .any(|i| i.takeaway_id.as_deref() == Some(&num_str))
                            {
                                target_list.push(MenuItem {
                                    takeaway_id: Some(num_str.clone()),
                                    alternatives: vec![MenuComponent {
                                        name: format!("Al-Götür Menü {}", num),
                                        amount: None,
                                        calories: None,
                                        category: None,
                                    }],
                                });
                            }
                        }
                        // Numarasız al-götür: Menü adını ezmeden orijinal ismi koru
                        if !has_nums
                            && !target_list
                                .iter()
                                .any(|i| i.takeaway_id.as_deref() == Some("1"))
                        {
                            let display_name = if item_name.is_empty() {
                                "Al-Götür Menü 1".to_string()
                            } else {
                                item_name.clone()
                            };

                            target_list.push(MenuItem {
                                takeaway_id: Some("1".to_string()),
                                alternatives: vec![MenuComponent {
                                    name: display_name,
                                    amount: None,
                                    calories: None,
                                    category: None,
                                }],
                            });
                        }
                    } else {
                        let mut amount = None;
                        let mut calories = None;

                        let amount_col = c + 1;
                        let cal_col = if has_gramaj { c + 2 } else { c + 1 };

                        if has_gramaj && amount_col < item_row_len {
                            let a = &sheet.rows[item_row][amount_col];
                            amount = validation::validate_numeric_value(a);
                        }

                        if has_enerji && cal_col < item_row_len {
                            let cal = &sheet.rows[item_row][cal_col];
                            calories = validation::validate_numeric_value(cal);
                        }

                        let mut alternatives = Vec::new();
                        let names: Vec<String> =
                            crate::parser::normalizer::split_smart_alternatives(&item_name);

                        let amounts: Vec<String> = if let Some(ref a) = amount {
                            split_outside_parens(a, '/')
                        } else {
                            Vec::new()
                        };

                        let cals: Vec<String> = if let Some(ref cal) = calories {
                            split_outside_parens(cal, '/')
                        } else {
                            Vec::new()
                        };

                        for (i, name) in names.iter().enumerate() {
                            let amt = if amounts.len() > i {
                                Some(amounts[i].to_string())
                            } else if amounts.len() == 1 {
                                Some(amounts[0].to_string())
                            } else {
                                None
                            };

                            let cal_val = if cals.len() > i {
                                Some(cals[i].to_string())
                            } else if cals.len() == 1 {
                                Some(cals[0].to_string())
                            } else {
                                None
                            };

                            alternatives.push(MenuComponent {
                                name: name.to_string(),
                                amount: amt,
                                calories: cal_val,
                                category: None,
                            });
                        }

                        if alternatives.is_empty() {
                            alternatives.push(MenuComponent {
                                name: crate::parser::normalizer::normalize_food_name(&item_name),
                                amount,
                                calories,
                                category: None,
                            });
                        }

                        target_list.push(MenuItem {
                            takeaway_id: None,
                            alternatives,
                        });
                    }

                    item_row += 1;
                }
            } else if date_like_regex().is_match(cell) {
                let raw = cell.trim().to_string();
                if !diag.unparseable_dates.contains(&raw) {
                    diag.unparseable_dates.push(raw);
                }
            }
        }
    }
}
