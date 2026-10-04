use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;

use worker::parser::llm::parse_document_with_llm_polymorphic;
use worker::parser::models::{MenuDatabase, MenuItem, ParsedDocumentPayload};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedTarget {
    pub city_slug: String,
    pub source_type: String,
    pub file_path: String,
    pub display_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedComponent {
    pub name: String,
    pub amount: Option<String>,
    pub calories: Option<String>,
    pub category: Option<String>,
    pub parsed_kcal: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedCourseItem {
    pub is_alternative_group: bool,
    pub slot_title: Option<String>,
    pub alternatives: Vec<StagedComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedMeal {
    pub meal_type: String,
    pub title: String,
    pub declared_kcal: Option<String>,
    pub computed_kcal: Option<u32>,
    pub courses: Vec<StagedCourseItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedDay {
    pub date: String,
    pub day_name: String,
    pub meals: Vec<StagedMeal>,
    pub total_computed_kcal: u32,
    pub has_calories: bool,
    pub has_portions: bool,
    pub is_empty_anomaly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStagingSummary {
    pub city_slug: String,
    pub source_type: String,
    pub file_name: String,
    pub display_label: String,
    pub rel_file_path: String,
    pub total_days: usize,
    pub min_date: Option<String>,
    pub max_date: Option<String>,
    pub breakfast_count: usize,
    pub lunch_count: usize,
    pub dinner_count: usize,
    pub empty_meals_count: usize,
    pub total_dishes_count: usize,
    pub total_alternatives_count: usize,
    pub monthly_avg_kcal: Option<u32>,
    pub min_day_kcal: Option<u32>,
    pub max_day_kcal: Option<u32>,
    pub days_with_kcal_count: usize,
    pub warnings: Vec<String>,
    pub has_stage1_csv: bool,
    pub days: Vec<StagedDay>,
}

fn parse_kcal_num(s: &str) -> Option<u32> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let primary_part = s.split('/').next().unwrap_or(s).trim();

    if let Some((low, high)) = primary_part.split_once('-') {
        let low_digits: String = low.chars().filter(|c| c.is_ascii_digit()).collect();
        let high_digits: String = high.chars().filter(|c| c.is_ascii_digit()).collect();
        let low_num: Option<u32> = low_digits.parse().ok();
        let high_num: Option<u32> = high_digits.parse().ok();
        match (low_num, high_num) {
            (Some(l), Some(h)) if l > 0 && h > 0 && l <= h && h < 5000 => {
                return Some((l + h) / 2);
            }
            (Some(l), _) if l > 0 && l < 5000 => return Some(l),
            (_, Some(h)) if h > 0 && h < 5000 => return Some(h),
            _ => {}
        }
    }

    let mut num_str = String::new();
    for c in primary_part.chars() {
        if c.is_ascii_digit() {
            num_str.push(c);
        } else if !num_str.is_empty() {
            break;
        }
    }

    if let Ok(val) = num_str.parse::<u32>()
        && val > 0
        && val <= 3500
    {
        return Some(val);
    }

    None
}

fn turkish_weekday_name(date: NaiveDate) -> &'static str {
    match date.weekday() {
        chrono::Weekday::Mon => "Pazartesi",
        chrono::Weekday::Tue => "Salı",
        chrono::Weekday::Wed => "Çarşamba",
        chrono::Weekday::Thu => "Perşembe",
        chrono::Weekday::Fri => "Cuma",
        chrono::Weekday::Sat => "Cumartesi",
        chrono::Weekday::Sun => "Pazar",
    }
}

fn build_staged_meal(
    items: &[MenuItem],
    meal_type: &str,
    title: &str,
    declared_kcal: Option<String>,
) -> Option<StagedMeal> {
    if items.is_empty() {
        return None;
    }

    let mut courses = Vec::new();
    let mut computed_sum: u32 = 0;
    let mut has_computed_val = false;

    for item in items {
        if item.alternatives.is_empty() {
            continue;
        }

        let raw_alts = &item.alternatives;
        let is_first_composite =
            raw_alts.len() > 2 && raw_alts[0].name.contains('/') && !raw_alts[1].name.contains('/');

        let target_slice = if is_first_composite {
            &raw_alts[1..]
        } else {
            &raw_alts[..]
        };

        let parent_calories_str = raw_alts[0].calories.as_deref().unwrap_or("");
        let split_calories: Vec<&str> = parent_calories_str.split('/').map(|s| s.trim()).collect();

        let parent_amount_str = raw_alts[0].amount.as_deref().unwrap_or("");
        let split_amounts: Vec<&str> = parent_amount_str.split('/').map(|s| s.trim()).collect();

        let mut alts = Vec::new();
        for (idx, comp) in target_slice.iter().enumerate() {
            let cal_str = if comp.calories.is_some()
                && !comp.calories.as_deref().unwrap_or("").contains('/')
            {
                comp.calories.clone()
            } else if idx < split_calories.len() && !split_calories[idx].is_empty() {
                Some(split_calories[idx].to_string())
            } else {
                comp.calories.clone()
            };

            let amt_str =
                if comp.amount.is_some() && !comp.amount.as_deref().unwrap_or("").contains('/') {
                    comp.amount.clone()
                } else if idx < split_amounts.len() && !split_amounts[idx].is_empty() {
                    Some(split_amounts[idx].to_string())
                } else {
                    comp.amount.clone()
                };

            let parsed_kcal = cal_str.as_deref().and_then(parse_kcal_num);

            if idx == 0
                && let Some(kc) = parsed_kcal
            {
                computed_sum += kc;
                has_computed_val = true;
            }

            alts.push(StagedComponent {
                name: comp.name.trim().to_string(),
                amount: amt_str.map(|s| s.trim().to_string()),
                calories: cal_str.map(|s| s.trim().to_string()),
                category: comp.category.clone(),
                parsed_kcal,
            });
        }

        if !alts.is_empty() {
            let is_alt = alts.len() > 1;
            courses.push(StagedCourseItem {
                is_alternative_group: is_alt,
                slot_title: if is_first_composite {
                    Some(raw_alts[0].name.trim().to_string())
                } else {
                    None
                },
                alternatives: alts,
            });
        }
    }

    if courses.is_empty() {
        return None;
    }

    let computed_kcal = if has_computed_val {
        Some(computed_sum)
    } else {
        None
    };

    Some(StagedMeal {
        meal_type: meal_type.to_string(),
        title: title.to_string(),
        declared_kcal,
        computed_kcal,
        courses,
    })
}

fn analyze_parsed_db(
    city_slug: &str,
    source_type: &str,
    display_label: &str,
    file_path: &Path,
    db: &MenuDatabase,
    has_csv: bool,
) -> FileStagingSummary {
    let mut warnings = Vec::new();
    let mut dates: BTreeSet<String> = BTreeSet::new();
    let mut breakfast_count = 0;
    let mut lunch_count = 0;
    let mut dinner_count = 0;
    let mut empty_meals_count = 0;
    let mut total_dishes_count = 0;
    let mut total_alternatives_count = 0;
    let mut days_summary = Vec::new();
    let mut daily_kcals = Vec::new();

    let mut sorted_dates: Vec<_> = db.keys().cloned().collect();
    sorted_dates.sort();

    for date_str in &sorted_dates {
        let day = &db[date_str];
        dates.insert(date_str.clone());

        let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok();
        let day_name = naive_date
            .map(turkish_weekday_name)
            .unwrap_or("")
            .to_string();

        let mut meals = Vec::new();
        let mut day_has_calories = false;
        let mut day_has_portions = false;

        // 1. Kahvaltı
        if let Some(m) = build_staged_meal(
            &day.normal.breakfast,
            "breakfast",
            "Kahvaltı",
            day.normal.breakfast_kcal.clone(),
        ) {
            breakfast_count += 1;
            for c in &m.courses {
                total_dishes_count += c.alternatives.len();
                if c.is_alternative_group {
                    total_alternatives_count += 1;
                }
                for comp in &c.alternatives {
                    if comp.calories.is_some() {
                        day_has_calories = true;
                    }
                    if comp.amount.is_some() {
                        day_has_portions = true;
                    }
                }
            }
            meals.push(m);
        }

        // 2. Öğle Yemeği
        if let Some(m) = build_staged_meal(
            &day.normal.lunch,
            "lunch",
            "Öğle Yemeği",
            day.normal.lunch_kcal.clone(),
        ) {
            lunch_count += 1;
            for c in &m.courses {
                total_dishes_count += c.alternatives.len();
                if c.is_alternative_group {
                    total_alternatives_count += 1;
                }
                for comp in &c.alternatives {
                    if comp.calories.is_some() {
                        day_has_calories = true;
                    }
                    if comp.amount.is_some() {
                        day_has_portions = true;
                    }
                }
            }
            meals.push(m);
        }

        // 3. Akşam Yemeği
        if let Some(m) = build_staged_meal(
            &day.normal.dinner,
            "dinner",
            "Akşam Yemeği",
            day.normal.dinner_kcal.clone(),
        ) {
            dinner_count += 1;
            for c in &m.courses {
                total_dishes_count += c.alternatives.len();
                if c.is_alternative_group {
                    total_alternatives_count += 1;
                }
                for comp in &c.alternatives {
                    if comp.calories.is_some() {
                        day_has_calories = true;
                    }
                    if comp.amount.is_some() {
                        day_has_portions = true;
                    }
                }
            }
            meals.push(m);
        }

        let is_empty_anomaly = (!day.normal.breakfast.is_empty()
            && day
                .normal
                .breakfast
                .iter()
                .all(|i| i.alternatives.is_empty()))
            || (!day.normal.dinner.is_empty()
                && day.normal.dinner.iter().all(|i| i.alternatives.is_empty()));

        if is_empty_anomaly {
            empty_meals_count += 1;
            warnings.push(format!("{}: Boş yemek listesi tespit edildi", date_str));
        }

        let total_computed_kcal: u32 = meals
            .iter()
            .map(|m| {
                m.computed_kcal
                    .or_else(|| m.declared_kcal.as_deref().and_then(parse_kcal_num))
                    .unwrap_or(0)
            })
            .sum();

        if total_computed_kcal > 0 {
            daily_kcals.push(total_computed_kcal);
        }

        days_summary.push(StagedDay {
            date: date_str.clone(),
            day_name,
            meals,
            total_computed_kcal,
            has_calories: day_has_calories,
            has_portions: day_has_portions,
            is_empty_anomaly,
        });
    }

    let days_with_kcal_count = daily_kcals.len();
    let monthly_avg_kcal = if !daily_kcals.is_empty() {
        let sum: u32 = daily_kcals.iter().sum();
        Some(sum / daily_kcals.len() as u32)
    } else {
        None
    };

    let min_day_kcal = daily_kcals.iter().copied().min();
    let max_day_kcal = daily_kcals.iter().copied().max();

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    FileStagingSummary {
        city_slug: city_slug.to_string(),
        source_type: source_type.to_string(),
        file_name,
        display_label: display_label.to_string(),
        rel_file_path: file_path.to_string_lossy().to_string(),
        total_days: dates.len(),
        min_date: dates.iter().next().cloned(),
        max_date: dates.iter().next_back().cloned(),
        breakfast_count,
        lunch_count,
        dinner_count,
        empty_meals_count,
        total_dishes_count,
        total_alternatives_count,
        monthly_avg_kcal,
        min_day_kcal,
        max_day_kcal,
        days_with_kcal_count,
        warnings,
        has_stage1_csv: has_csv,
        days: days_summary,
    }
}

pub fn generate_staging_hub_html(summaries: &[FileStagingSummary]) -> String {
    let summaries_json =
        serde_json::to_string_pretty(summaries).unwrap_or_else(|_| "[]".to_string());

    r##"<!DOCTYPE html>
<html lang="tr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Kepçe | Doğrulama Merkezi</title>
    <style>
        :root {
            --bg-canvas: #0b1120;
            --bg-card: #151f32;
            --bg-card-hover: #1e293b;
            --text-main: #f1f5f9;
            --text-muted: #94a3b8;
            --text-dim: #64748b;
            --primary: #38bdf8;
            --border: #243248;
            --border-subtle: rgba(255, 255, 255, 0.06);
            --radius-md: 8px;
            --radius-sm: 5px;
            --font-sans: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            --dur-fast: 0.15s;
            --ease-standard: cubic-bezier(0.4, 0, 0.2, 1);
        }

        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            background-color: var(--bg-canvas);
            color: var(--text-main);
            font-family: var(--font-sans);
            display: flex;
            flex-direction: column;
            height: 100vh;
            overflow: hidden;
        }

        /* Top Header */
        header {
            background-color: #0f172a;
            border-bottom: 1px solid var(--border);
            padding: 8px 16px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 14px;
            flex-shrink: 0;
        }
        .header-left {
            display: flex;
            align-items: center;
            gap: 12px;
            overflow: hidden;
        }
        .brand {
            font-weight: 700;
            font-size: 0.95rem;
            letter-spacing: -0.02em;
            flex-shrink: 0;
        }
        .city-selector {
            display: flex;
            gap: 6px;
            overflow-x: auto;
            scrollbar-width: none;
        }
        .city-selector::-webkit-scrollbar { display: none; }

        .city-btn {
            background: transparent;
            color: var(--text-muted);
            border: 1px solid var(--border);
            padding: 5px 11px;
            border-radius: var(--radius-sm);
            cursor: pointer;
            font-weight: 500;
            font-size: 0.8rem;
            transition: all var(--dur-fast) var(--ease-standard);
            white-space: nowrap;
        }
        .city-btn:hover { background: var(--bg-card-hover); color: var(--text-main); }
        .city-btn:active { transform: scale(0.98); }
        .city-btn.active {
            background: var(--primary);
            color: #0b1120;
            border-color: var(--primary);
            font-weight: 600;
        }

        .header-stats {
            font-size: 0.8rem;
            color: var(--text-muted);
            white-space: nowrap;
            display: flex;
            align-items: center;
            gap: 10px;
            flex-shrink: 0;
        }
        .stat-strong {
            color: var(--text-main);
            font-weight: 600;
        }

        /* 3 Panes */
        .workspace {
            display: flex;
            flex: 1;
            overflow: hidden;
        }
        .pane-doc {
            flex: 1.15;
            border-right: 1px solid var(--border);
            display: flex;
            flex-direction: column;
            background: #060911;
            overflow: hidden;
        }
        .pane-trace {
            flex: 0.95;
            border-right: 1px solid var(--border);
            display: flex;
            flex-direction: column;
            background: #090d16;
            overflow: hidden;
        }
        .pane-ui {
            flex: 1.4;
            display: flex;
            flex-direction: column;
            background: var(--bg-canvas);
            overflow: hidden;
        }

        .pane-header {
            padding: 8px 14px;
            background: #0f172a;
            border-bottom: 1px solid var(--border);
            font-size: 0.8rem;
            color: var(--text-muted);
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-weight: 600;
            flex-shrink: 0;
        }

        .doc-frame-wrapper {
            flex: 1;
            overflow: auto;
            display: flex;
            justify-content: center;
            align-items: flex-start;
            padding: 8px;
        }
        .doc-frame-wrapper embed,
        .doc-frame-wrapper iframe,
        .doc-frame-wrapper img {
            width: 100%;
            height: 100%;
            border: none;
            object-fit: contain;
        }

        .trace-content {
            flex: 1;
            overflow: auto;
            padding: 12px;
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
            font-size: 0.75rem;
            line-height: 1.45;
            white-space: pre;
            color: #cbd5e1;
        }

        .ui-content {
            flex: 1;
            overflow-y: auto;
            padding: 12px 14px;
            display: flex;
            flex-direction: column;
            gap: 10px;
        }

        /* View Mode Switch */
        .view-switch {
            display: flex;
            gap: 2px;
            background: rgba(255, 255, 255, 0.05);
            padding: 2px;
            border-radius: var(--radius-sm);
        }
        .view-btn {
            background: transparent;
            border: none;
            color: var(--text-muted);
            font-size: 0.74rem;
            padding: 3px 8px;
            border-radius: 4px;
            cursor: pointer;
            font-weight: 500;
        }
        .view-btn:hover { color: var(--text-main); }
        .view-btn.active {
            background: var(--bg-card-hover);
            color: var(--text-main);
            font-weight: 600;
        }

        /* Clean Menu Card */
        .menu-card {
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: var(--radius-md);
            padding: 12px 14px;
            display: flex;
            flex-direction: column;
            gap: 10px;
            transition: border-color var(--dur-fast) ease;
        }
        .menu-card.selected {
            border-color: var(--primary);
        }
        .card-header {
            display: flex;
            justify-content: space-between;
            align-items: baseline;
            border-bottom: 1px solid var(--border-subtle);
            padding-bottom: 6px;
        }
        .date-title {
            font-weight: 700;
            font-size: 0.92rem;
            color: var(--text-main);
        }
        .day-sub {
            font-weight: 500;
            font-size: 0.8rem;
            color: var(--primary);
            margin-left: 6px;
        }
        .card-total-kcal {
            font-size: 0.76rem;
            color: var(--text-muted);
        }
        .card-total-kcal strong {
            color: var(--text-main);
        }

        .meal-block {
            display: flex;
            flex-direction: column;
            gap: 4px;
        }
        .meal-label {
            font-size: 0.7rem;
            font-weight: 700;
            letter-spacing: 0.04em;
            color: var(--text-dim);
            text-transform: uppercase;
            margin-bottom: 2px;
        }

        .dish-list {
            display: flex;
            flex-direction: column;
            gap: 4px;
        }
        .dish-row {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 5px 8px;
            background: rgba(11, 17, 32, 0.45);
            border-radius: var(--radius-sm);
            font-size: 0.82rem;
            gap: 8px;
        }
        .dish-row.alt-row {
            background: rgba(11, 17, 32, 0.25);
            border-left: 2px solid var(--text-dim);
            margin-left: 10px;
        }
        .dish-name {
            color: var(--text-main);
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }
        .alt-prefix {
            color: var(--text-dim);
            font-size: 0.74rem;
            margin-right: 4px;
        }
        .dish-meta {
            display: flex;
            align-items: center;
            gap: 8px;
            flex-shrink: 0;
            font-size: 0.74rem;
            color: var(--text-muted);
        }

        /* Clean Table View */
        .table-view-container {
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: var(--radius-md);
            overflow: hidden;
        }
        .table-view-wrap {
            width: 100%;
            border-collapse: collapse;
            font-size: 0.76rem;
        }
        .table-view-wrap th {
            text-align: left;
            padding: 7px 10px;
            color: var(--text-dim);
            border-bottom: 1px solid var(--border);
            font-weight: 600;
            background: rgba(15, 23, 42, 0.5);
        }
        .table-view-wrap td {
            padding: 6px 10px;
            border-bottom: 1px solid var(--border-subtle);
            color: var(--text-main);
        }
        .table-view-wrap tr:last-child td {
            border-bottom: none;
        }
        .text-center { text-align: center; }
        .text-right { text-align: right; }

        /* Timeline Navigation */
        .timeline-bar {
            background: #0f172a;
            border-top: 1px solid var(--border);
            padding: 6px 14px;
            display: flex;
            gap: 5px;
            overflow-x: auto;
            flex-shrink: 0;
            scrollbar-width: thin;
        }
        .day-chip {
            padding: 4px 9px;
            background: transparent;
            border: 1px solid var(--border);
            border-radius: var(--radius-sm);
            font-size: 0.74rem;
            color: var(--text-muted);
            cursor: pointer;
            white-space: nowrap;
            transition: all var(--dur-fast) var(--ease-standard);
        }
        .day-chip:hover { color: var(--text-main); background: var(--bg-card-hover); }
        .day-chip:active { transform: scale(0.98); }
        .day-chip.active {
            background: var(--primary);
            color: #0b1120;
            font-weight: 600;
            border-color: var(--primary);
        }

        @media (max-width: 900px) {
            .workspace { flex-direction: column; overflow-y: auto; }
            .pane-doc, .pane-trace, .pane-ui { flex: none; height: 50vh; border-right: none; border-bottom: 1px solid var(--border); }
        }
        @media (max-width: 600px) {
            header { flex-direction: column; align-items: flex-start; gap: 6px; }
        }
    </style>
</head>
<body>
    <header>
        <div class="header-left">
            <div class="brand">Kepçe Staging</div>
            <div class="city-selector" id="citySelector"></div>
        </div>
        <div class="header-stats" id="headerStats"></div>
    </header>

    <div class="workspace">
        <!-- Sol: Orijinal Belge -->
        <section class="pane-doc">
            <div class="pane-header">
                <span>Zemin Gerçekliği</span>
                <span id="docMeta"></span>
            </div>
            <div class="doc-frame-wrapper" id="docViewer"></div>
        </section>

        <!-- Orta: Aşama 1 CSV Tablo Izgarası -->
        <section class="pane-trace">
            <div class="pane-header">
                <span>Aşama 1: Ham CSV Tablosu</span>
                <span id="csvMeta"></span>
            </div>
            <div class="trace-content" id="traceViewer">Yükleniyor...</div>
        </section>

        <!-- Sağ: Kepçe Tabldot Görünümü -->
        <section class="pane-ui">
            <div class="pane-header">
                <span>Tabldot Menüsü</span>
                <div style="display:flex; align-items:center; gap:10px;">
                    <div class="view-switch">
                        <button class="view-btn active" id="btnCards" onclick="setViewMode('cards')">Kartlar</button>
                        <button class="view-btn" id="btnTable" onclick="setViewMode('table')">Tablo</button>
                    </div>
                    <span id="uiMeta" style="font-size:0.75rem; color:var(--text-dim);"></span>
                </div>
            </div>
            <div class="ui-content" id="uiViewer"></div>
        </section>
    </div>

    <!-- Alt Çubuk: Gün Gezgini -->
    <div class="timeline-bar" id="timelineBar"></div>

    <script>
        const STAGING_DATA = "##.to_string()
        + &summaries_json
        + r##";
        let currentFileIdx = 0;
        let currentViewMode = 'cards';

        function init() {
            const selector = document.getElementById('citySelector');
            selector.innerHTML = '';

            STAGING_DATA.forEach((file, idx) => {
                const btn = document.createElement('button');
                btn.className = 'city-btn' + (idx === currentFileIdx ? ' active' : '');
                btn.textContent = file.display_label || file.file_name;
                btn.onclick = () => selectFile(idx);
                selector.appendChild(btn);
            });

            if (STAGING_DATA.length > 0) {
                renderCurrentFile();
            }
        }

        function selectFile(idx) {
            currentFileIdx = idx;
            document.querySelectorAll('.city-btn').forEach((b, i) => {
                b.classList.toggle('active', i === idx);
            });
            renderCurrentFile();
        }

        function setViewMode(mode) {
            currentViewMode = mode;
            document.getElementById('btnCards').classList.toggle('active', mode === 'cards');
            document.getElementById('btnTable').classList.toggle('active', mode === 'table');
            renderRightPane();
        }

        function renderCurrentFile() {
            const file = STAGING_DATA[currentFileIdx];
            if (!file) return;

            // 1. Header Stats: Quiet and clean
            const stats = document.getElementById('headerStats');
            let statsHtml = `<span><strong class="stat-strong">${file.total_days}</strong> gün</span>`;
            statsHtml += `<span>•</span><span><strong class="stat-strong">${file.total_dishes_count}</strong> kalem</span>`;
            if (file.total_alternatives_count > 0) {
                statsHtml += `<span>•</span><span><strong class="stat-strong">${file.total_alternatives_count}</strong> alternatifli</span>`;
            }
            if (file.monthly_avg_kcal) {
                statsHtml += `<span>•</span><span>Aylık Ort: <strong class="stat-strong">~${file.monthly_avg_kcal} kcal</strong></span>`;
            }
            stats.innerHTML = statsHtml;

            // 2. Doc Viewer
            document.getElementById('docMeta').textContent = file.file_name;
            const docViewer = document.getElementById('docViewer');
            const cleanPath = '/' + file.rel_file_path.replace(/^\.\//, '');
            if (file.file_name.endsWith('.pdf')) {
                docViewer.innerHTML = `<embed src="${cleanPath}" type="application/pdf" width="100%" height="100%">`;
            } else {
                docViewer.innerHTML = `<img src="${cleanPath}" alt="${file.file_name}">`;
            }

            // 3. Trace / CSV
            const csvViewer = document.getElementById('traceViewer');
            document.getElementById('csvMeta').textContent = file.has_stage1_csv ? 'CSV mevcut' : 'CSV yok';
            const fileStem = file.file_name.replace(/\.[^/.]+$/, "");
            fetch(`${file.city_slug}/${fileStem}/stage1_grid.csv`)
                .then(r => r.ok ? r.text() : "CSV iz dosyası bulunamadı.")
                .then(txt => { csvViewer.textContent = txt; })
                .catch(() => { csvViewer.textContent = "CSV yüklenemedi."; });

            // 4. Right Pane
            document.getElementById('uiMeta').textContent = `${file.days.length} Gün`;
            renderRightPane();

            // 5. Timeline Bar
            const timeline = document.getElementById('timelineBar');
            timeline.innerHTML = '';
            file.days.forEach(d => {
                const chip = document.createElement('div');
                chip.className = 'day-chip';
                chip.id = 'chip-' + d.date;
                const dayNum = d.date.split('-').slice(1).join('.');
                chip.textContent = dayNum;
                chip.onclick = () => scrollToDate(d.date);
                timeline.appendChild(chip);
            });
        }

        function renderRightPane() {
            const file = STAGING_DATA[currentFileIdx];
            if (!file) return;

            const uiViewer = document.getElementById('uiViewer');
            uiViewer.innerHTML = '';

            if (currentViewMode === 'cards') {
                file.days.forEach(d => {
                    const card = document.createElement('div');
                    card.className = 'menu-card';
                    card.id = 'card-' + d.date;

                    let mealsHtml = '';
                    d.meals.forEach(m => {
                        let coursesHtml = '';
                        m.courses.forEach(course => {
                            course.alternatives.forEach((comp, aIdx) => {
                                coursesHtml += `
                                    <div class="dish-row ${aIdx > 0 ? 'alt-row' : ''}">
                                        <div class="dish-name">
                                            ${aIdx > 0 ? '<span class="alt-prefix">Alternatif:</span>' : ''}
                                            ${comp.name}
                                        </div>
                                        <div class="dish-meta">
                                            ${comp.amount ? `<span>${comp.amount}</span>` : ''}
                                            ${comp.calories ? `<span>${comp.calories}</span>` : ''}
                                        </div>
                                    </div>
                                `;
                            });
                        });

                        mealsHtml += `
                            <div class="meal-block">
                                <div class="meal-label">${m.title}</div>
                                <div class="dish-list">${coursesHtml}</div>
                            </div>
                        `;
                    });

                    card.innerHTML = `
                        <div class="card-header">
                            <div>
                                <span class="date-title">${d.date}</span>
                                <span class="day-sub">${d.day_name}</span>
                            </div>
                            ${d.total_computed_kcal > 0 ? `<span class="card-total-kcal">Toplam: <strong>~${d.total_computed_kcal} kcal</strong></span>` : ''}
                        </div>
                        ${mealsHtml}
                    `;
                    uiViewer.appendChild(card);
                });
            } else {
                // Compact Table View
                const tableContainer = document.createElement('div');
                tableContainer.className = 'table-view-container';

                let rowsHtml = '';
                file.days.forEach(d => {
                    d.meals.forEach(m => {
                        m.courses.forEach(course => {
                            course.alternatives.forEach((comp, aIdx) => {
                                rowsHtml += `
                                    <tr id="card-${d.date}">
                                        <td>${d.date}</td>
                                        <td style="color:var(--text-muted);">${m.title}</td>
                                        <td>
                                            ${aIdx > 0 ? '<span class="alt-prefix">↳</span>' : ''}
                                            ${comp.name}
                                        </td>
                                        <td class="text-center" style="color:var(--text-muted);">${comp.amount || '-'}</td>
                                        <td class="text-right" style="color:var(--text-muted);">${comp.calories || '-'}</td>
                                    </tr>
                                `;
                            });
                        });
                    });
                });

                tableContainer.innerHTML = `
                    <table class="table-view-wrap">
                        <thead>
                            <tr>
                                <th>Tarih</th>
                                <th>Öğün</th>
                                <th>Yemek / Alternatif</th>
                                <th class="text-center">Gramaj</th>
                                <th class="text-right">Kalori</th>
                            </tr>
                        </thead>
                        <tbody>${rowsHtml}</tbody>
                    </table>
                `;
                uiViewer.appendChild(tableContainer);
            }
        }

        function scrollToDate(dateStr) {
            document.querySelectorAll('.day-chip').forEach(c => c.classList.remove('active'));
            const activeChip = document.getElementById('chip-' + dateStr);
            if (activeChip) activeChip.classList.add('active');

            const targetCard = document.getElementById('card-' + dateStr);
            if (targetCard) {
                targetCard.scrollIntoView({ behavior: 'smooth', block: 'center' });
                document.querySelectorAll('.menu-card').forEach(c => c.classList.remove('selected'));
                if (targetCard.classList.contains('menu-card')) {
                    targetCard.classList.add('selected');
                }
            }
        }

        window.onload = init;
    </script>
</body>
</html>
"##
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let gemini_key = env::var("GEMINI_API_KEY").ok();
    let openrouter_key = env::var("OPENROUTER_API_KEY").ok();

    println!("==================================================");
    println!("KEPÇE İZOLE DOĞRULAMA MERKEZİ (STAGE HUB)");
    println!("Gemini anahtarı mevcut: {}", gemini_key.is_some());
    println!("OpenRouter anahtarı mevcut: {}", openrouter_key.is_some());
    println!("==================================================");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()?;

    let base_staging_str = env::var("STAGING_DIR").unwrap_or_else(|_| "target/staging".to_string());
    let base_staging = Path::new(&base_staging_str);
    fs::create_dir_all(base_staging)?;

    let sources_root = env::var("SOURCES_DIR").unwrap_or_else(|_| "data/submissions".to_string());
    let p = |rel: &str| -> String { format!("{}/{}", sources_root.trim_end_matches('/'), rel) };

    let targets = vec![
        StagedTarget {
            city_slug: "erzincan".to_string(),
            source_type: "anonim".to_string(),
            file_path: p("anonim/erzincan/20261003_141633_CCF_000290.pdf"),
            display_label: "Erzincan Kahvaltı".to_string(),
        },
        StagedTarget {
            city_slug: "erzincan".to_string(),
            source_type: "anonim".to_string(),
            file_path: p("anonim/erzincan/20261003_153616_CCF_000292.pdf"),
            display_label: "Erzincan Akşam".to_string(),
        },
        StagedTarget {
            city_slug: "bursa".to_string(),
            source_type: "anonim".to_string(),
            file_path: p("anonim/bursa/menu_bursa.pdf"),
            display_label: "Bursa".to_string(),
        },
        StagedTarget {
            city_slug: "corum".to_string(),
            source_type: "anonim".to_string(),
            file_path: p("anonim/corum/menu_corum1.jpeg"),
            display_label: "Çorum Kahvaltı".to_string(),
        },
        StagedTarget {
            city_slug: "corum".to_string(),
            source_type: "anonim".to_string(),
            file_path: p("anonim/corum/menu_corum2.jpeg"),
            display_label: "Çorum Akşam".to_string(),
        },
        StagedTarget {
            city_slug: "afyonkarahisar".to_string(),
            source_type: "anonim".to_string(),
            file_path: p("anonim/afyonkarahisar/1.jpg"),
            display_label: "Afyon Kahvaltı".to_string(),
        },
        StagedTarget {
            city_slug: "afyonkarahisar".to_string(),
            source_type: "anonim".to_string(),
            file_path: p("anonim/afyonkarahisar/2.jpg"),
            display_label: "Afyon Akşam".to_string(),
        },
        StagedTarget {
            city_slug: "istanbul".to_string(),
            source_type: "admin".to_string(),
            file_path: p("admin/istanbul/Ekim_2026_Menuler.pdf"),
            display_label: "İstanbul Ekim".to_string(),
        },
        StagedTarget {
            city_slug: "istanbul".to_string(),
            source_type: "admin".to_string(),
            file_path: p("admin/istanbul/Aksam Yemegi Menu.png"),
            display_label: "İstanbul Akşam".to_string(),
        },
        StagedTarget {
            city_slug: "istanbul".to_string(),
            source_type: "admin".to_string(),
            file_path: p("admin/istanbul/Kahvalti Menu.png"),
            display_label: "İstanbul Kahvaltı".to_string(),
        },
    ];

    let mut all_summaries = Vec::new();
    let force = env::args().any(|a| a == "--force");

    for target in targets {
        let path = Path::new(&target.file_path);
        if !path.exists() {
            println!("UYARI: Dosya bulunamadı, atlanıyor: {}", target.file_path);
            continue;
        }

        let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let target_dir = base_staging.join(&target.city_slug).join(file_stem);
        fs::create_dir_all(&target_dir)?;

        let result_json_path = target_dir.join("stage2_menu.json");
        let stage1_csv_path = target_dir.join("stage1_grid.csv");
        let summary_json_path = target_dir.join("summary.json");

        println!(
            "\n>>> [{}] İşleniyor: {}",
            target.city_slug, target.file_path
        );

        let (menu_db, has_csv) = if result_json_path.exists() && !force {
            println!("--- Önbellekten okundu: {:?}", result_json_path);
            let raw = fs::read_to_string(&result_json_path)?;
            let db: MenuDatabase = serde_json::from_str(&raw)?;
            let has_csv = stage1_csv_path.exists();
            (db, has_csv)
        } else {
            println!("--- Worker çok biçimli çıkarım hattı çalıştırılıyor...");
            match parse_document_with_llm_polymorphic(&client, gemini_key.as_deref(), path).await {
                Ok((payload, diag)) => {
                    let mut db = match payload {
                        ParsedDocumentPayload::DailyMenu(d) => d,
                        ParsedDocumentPayload::Compound { menu: Some(d), .. } => d,
                        other => {
                            println!("UYARI: Belge tabldot menü formatında değil: {:?}", other);
                            continue;
                        }
                    };

                    if target.city_slug == "erzincan" {
                        let mut remapped = MenuDatabase::new();
                        for (date_str, day) in db {
                            let new_date = if date_str.starts_with("2024-10-")
                                || date_str.starts_with("2023-10-")
                            {
                                format!("2026-10-{}", &date_str[8..])
                            } else {
                                date_str
                            };
                            remapped.insert(new_date, day);
                        }
                        db = remapped;
                    }

                    let has_csv = if let Some(csv) = diag.table_grid_csv {
                        fs::write(&stage1_csv_path, csv)?;
                        println!("--- Aşama 1 CSV kaydedildi: {:?}", stage1_csv_path);
                        true
                    } else {
                        false
                    };

                    let serialized = serde_json::to_string_pretty(&db)?;
                    fs::write(&result_json_path, serialized)?;
                    println!("--- Aşama 2 Menü JSON kaydedildi: {:?}", result_json_path);

                    (db, has_csv)
                }
                Err(e) => {
                    println!("HATA: Ayrıştırma başarısız oldu: {:?}", e);
                    continue;
                }
            }
        };

        let summary = analyze_parsed_db(
            &target.city_slug,
            &target.source_type,
            &target.display_label,
            path,
            &menu_db,
            has_csv,
        );

        let summary_serialized = serde_json::to_string_pretty(&summary)?;
        fs::write(&summary_json_path, summary_serialized)?;

        println!(
            "ÖZET: {} gün | {} kahvaltı | {} akşam | {} alternatifli yuva | ortalama: {:?} kcal | boş öğün: {}",
            summary.total_days,
            summary.breakfast_count,
            summary.dinner_count,
            summary.total_alternatives_count,
            summary.monthly_avg_kcal,
            summary.empty_meals_count
        );

        all_summaries.push(summary);
    }

    let html_content = generate_staging_hub_html(&all_summaries);
    let html_path = base_staging.join("index.html");
    fs::write(&html_path, html_content)?;
    println!("\n==================================================");
    println!("STAGING HUB HTML BAŞARIYLA GÜNCELLENDİ:");
    println!("Dosya: {:?}", html_path);
    println!("Toplam İşlenen Dosya: {}", all_summaries.len());
    println!("==================================================");

    Ok(())
}
