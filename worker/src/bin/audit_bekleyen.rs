//! Tanılama aracı: yerel `bekleyen` drop-zone dosyalarını DB ve LLM olmadan
//! ayrıştırıp gün/öğün özetini ve YENİ KARAR MOTORUNUN önizlemesini yazdırır.
//! `file_ingest` akışının dosya-başına ayrıştırma ve sınıflandırma davranışını
//! birebir taklit eder; hiçbir dosya taşınmaz, veritabanına yazılmaz.
//!
//! Kullanım: `cargo run -p worker --example audit_bekleyen -- <dizin>`

use std::path::{Path, PathBuf};
use worker::parser::core::ParseDiagnostics;
use worker::parser::models::MenuDatabase;
use worker::tasks::file_ingest::{
    GateConfig, IngestOutcome, build_parsed_file, classify_ingest, reason_message,
};

fn summarize(
    db: &MenuDatabase,
) -> (
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    Option<String>,
    Option<String>,
) {
    let mut days = 0usize;
    let mut nb = 0usize;
    let mut nl = 0usize;
    let mut nd = 0usize;
    let mut cb = 0usize;
    let mut cl = 0usize;
    let mut cd = 0usize;
    let mut first: Option<String> = None;
    let mut last: Option<String> = None;
    for (date, day) in db {
        days += 1;
        if !day.normal.breakfast.is_empty() {
            nb += 1;
        }
        if !day.normal.lunch.is_empty() {
            nl += 1;
        }
        if !day.normal.dinner.is_empty() {
            nd += 1;
        }
        if !day.colyak.breakfast.is_empty() {
            cb += 1;
        }
        if !day.colyak.lunch.is_empty() {
            cl += 1;
        }
        if !day.colyak.dinner.is_empty() {
            cd += 1;
        }
        match &first {
            Some(f) if f <= date => {}
            _ => first = Some(date.clone()),
        }
        match &last {
            Some(x) if x >= date => {}
            _ => last = Some(date.clone()),
        }
    }
    (days, nb, nl, nd, cb, cl, cd, first, last)
}

fn dump_excel_sheets(path: &Path) {
    use calamine::{Data, Reader, Xlsx, open_workbook};
    let Ok(mut wb) = open_workbook::<Xlsx<_>, _>(path) else {
        println!("    [HATA] workbook açılamadı");
        return;
    };
    let names = wb.sheet_names().to_vec();
    println!("       sayfalar: {:?}", names);
    for name in names {
        let Ok(range) = wb.worksheet_range(&name) else {
            continue;
        };
        let (h, w) = (range.height(), range.width());
        let mut preview: Vec<String> = Vec::new();
        'outer: for r in 0..h.min(12) {
            for c in 0..w {
                let cell = range.get((r, c)).unwrap_or(&Data::Empty);
                let s = cell.to_string();
                if !s.trim().is_empty() {
                    preview.push(format!("[{}:{}] {}", r, c, s.trim()));
                    if preview.len() >= 12 {
                        break 'outer;
                    }
                }
            }
        }
        println!(
            "       - '{name}': {h}x{w} | ilk hücreler: {}",
            preview.join(" | ")
        );
    }
}

fn parse_offline(path: &Path) -> Option<(MenuDatabase, ParseDiagnostics)> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let path_str = path.to_string_lossy().to_string();

    let mut db = MenuDatabase::new();
    let result: Option<(MenuDatabase, ParseDiagnostics)> = match ext.as_str() {
        "xlsx" | "xls" => {
            match worker::parser::excel::parse_excel_with_diagnostics(&path_str, &mut db) {
                Ok(diag) => Some((db, diag)),
                Err(e) => {
                    println!("    [HATA] excel: {e}");
                    None
                }
            }
        }
        "json" => {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.json");
            match std::fs::read_to_string(path).ok().and_then(|c| {
                worker::parser::json::parse_json_str_with_diagnostics(&c, file_name).ok()
            }) {
                Some((d, mismatches)) => Some((
                    d,
                    ParseDiagnostics {
                        date_raw_mismatches: mismatches,
                        ..Default::default()
                    },
                )),
                _ => {
                    println!("    [HATA] json: ayrıştırılamadı");
                    None
                }
            }
        }
        _ => None,
    };
    result.map(|(mut db, diag)| {
        for day in db.values_mut() {
            worker::parser::validation::finalize_day_metadata(day);
        }
        (db, diag)
    })
}

/// Karar önizlemesi: ingest akışının vereceği kararı (Complete/Partial/Suspect/
/// Permanent) ve hedef klasörü yazdırır.
fn print_decision_preview(file_name: &str, db: &MenuDatabase, diag: &ParseDiagnostics) {
    let cfg = GateConfig::from_env();
    let parsed = build_parsed_file(file_name, db, diag.clone());
    let (outcome, scope) = classify_ingest(&parsed, &cfg);
    let (label, dest) = match &outcome {
        IngestOutcome::Complete => ("COMPLETE", "vault/ (yazılır)"),
        IngestOutcome::Partial { .. } => ("PARTIAL", "_karantina/ (karar bekler, YAZILMAZ)"),
        IngestOutcome::Suspect { .. } => ("SUSPECT", "_karantina/ (karar bekler, YAZILMAZ)"),
        IngestOutcome::Permanent { .. } => ("PERMANENT", "hatali/ (bir daha denenmez)"),
        IngestOutcome::Transient { .. } => ("TRANSIENT", "bekleyen/ (kalır)"),
    };
    println!("       KARAR: {} -> {}", label, dest);
    if let Some(sm) = scope.scope_month {
        println!(
            "       kapsam: {}-{:02} ({} gün / beklenen {}), kapsam dışı: {}",
            sm.0,
            sm.1,
            scope.in_scope_days,
            scope.expected_days.unwrap_or(0),
            if scope.stray_dates.is_empty() {
                "yok".to_string()
            } else {
                scope
                    .stray_dates
                    .iter()
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        );
    }
    match &outcome {
        IngestOutcome::Complete => {}
        IngestOutcome::Partial { reason }
        | IngestOutcome::Suspect { reason }
        | IngestOutcome::Permanent { reason }
        | IngestOutcome::Transient { reason } => {
            println!(
                "       sebep: {} — {}",
                reason.as_str(),
                reason_message(*reason, &parsed, &scope)
            );
        }
    }
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk(&p, out);
        } else {
            out.push(p);
        }
    }
}

fn main() {
    let base = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "data/menuler/admin/bekleyen".to_string());
    let mut files = Vec::new();
    walk(Path::new(&base), &mut files);
    files.sort();

    println!("=== bekleyen ayrıştırma denetimi: {} ===", base);
    let mut ok = 0usize;
    let mut empty = 0usize;
    let mut skipped = 0usize;
    for f in &files {
        let ext = f
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let rel = f
            .strip_prefix(&base)
            .unwrap_or(f)
            .to_string_lossy()
            .to_string();

        if ext == "xlsx" || ext == "xls" {
            dump_excel_sheets(f);
        }
        match parse_offline(f) {
            Some((db, diag)) => {
                let (days, nb, nl, nd, cb, cl, cd, first, last) = summarize(&db);
                if cb > 0 || cl > 0 || cd > 0 {
                    println!(
                        "[OK ] {rel:<55} gün={days:<3} normal(k={nb},ö={nl},a={nd}) çölyak(k={cb},ö={cl},a={cd}) aralık={}..{}",
                        first.unwrap_or_else(|| "-".into()),
                        last.unwrap_or_else(|| "-".into())
                    );
                } else {
                    println!(
                        "[OK ] {rel:<55} gün={days:<3} kahvaltı={nb:<3} öğle={nl:<3} akşam={nd:<3} aralık={}..{}",
                        first.unwrap_or_else(|| "-".into()),
                        last.unwrap_or_else(|| "-".into())
                    );
                }
                let mut keys: Vec<String> = db.keys().cloned().collect();
                keys.sort();
                println!("       tarihler: {}", keys.join(", "));
                let file_name = f
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                print_decision_preview(&file_name, &db, &diag);
                if days == 0 {
                    empty += 1;
                } else {
                    ok += 1;
                }
            }
            None => {
                skipped += 1;
                println!("[SKIP] {rel:<55} (uzantı='{ext}', yalnızca pdf/görsel → LLM gerekir)");
            }
        }
    }
    println!(
        "\nÖzet: {} dosya; {} başarılı ayrıştırma, {} boş (0 gün), {} atlandı (LLM)",
        files.len(),
        ok,
        empty,
        skipped
    );
}
