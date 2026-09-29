use anyhow::Result;
use calamine::{Data, DataType, Reader, Xlsx, open_workbook};
use std::path::Path;

use crate::parser::core::{
    DateOrderResolution, ParseDiagnostics, SheetGrid, parse_grid_with_order,
    resolve_file_date_order,
};
use crate::parser::models::MenuDatabase;
use crate::parser::validation;

/// Geriye dönük uyumlu sarmalayıcı: tanılama sinyallerini yok sayar.
///
/// Yeni akışta çağıranlar `parse_excel_with_diagnostics` kullanır; karar motoru
/// şüphe sinyallerine (tarih sırası çelişkisi, çözülemeyen tarihler, gün adı
/// uyuşmazlığı) ihtiyaç duyar.
pub fn parse_excel(path_str: &str, db: &mut MenuDatabase) -> Result<()> {
    parse_excel_with_diagnostics(path_str, db).map(|_| ())
}

/// Excel çalışma kitabını DOSYA bazlı tarih sırası çözümüyle ayrıştırır.
///
/// İki geçişli çalışır:
/// 1. Tüm sayfaların ızgaraları belleğe alınır ve dosya bazlı deterministik
///    tarih sırası çözümü (`resolve_file_date_order`, T1..T5) uygulanır.
/// 2. Çözülen sıra ile her sayfa ayrıştırılır; şüphe sinyalleri `ParseDiagnostics`
///    olarak döner. Sıra `Conflict` ise ayrıştırma DMY varsayılanıyla devam eder
///    ama çağıran taraf dosyayı karantinaya almalıdır (veri yazılmaz).
pub fn parse_excel_with_diagnostics(
    path_str: &str,
    db: &mut MenuDatabase,
) -> Result<ParseDiagnostics> {
    let path = Path::new(path_str);
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let sheet_names = workbook.sheet_names().to_vec();

    let mut diag = ParseDiagnostics::default();

    if sheet_names.len() > validation::MAX_SHEET_COUNT {
        tracing::warn!(
            "SKIP: {:?} has {} sheets (max {})",
            path.file_name().unwrap_or_default(),
            sheet_names.len(),
            validation::MAX_SHEET_COUNT
        );
        return Ok(diag);
    }

    let file_name_hint = path.file_name().unwrap_or_default().to_string_lossy();

    // 1. geçiş: tüm sayfaları ızgaraya çevir
    let mut grids: Vec<SheetGrid> = Vec::new();
    for sheet_name in sheet_names {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let height = range.height();
            let width = range.width();

            let mut rows = Vec::new();
            for r in 0..height {
                let mut row = Vec::new();
                for c in 0..width {
                    let cell = range.get((r, c)).unwrap_or(&Data::Empty);

                    // Gerçek tarih hücreleri DMY metnine normalize edilir;
                    // böylece seri numarası belirsizliği ortadan kalkar.
                    if let Some(date) = cell.as_date() {
                        row.push(date.format("%d.%m.%Y").to_string());
                    } else {
                        row.push(cell.to_string());
                    }
                }
                rows.push(row);
            }

            grids.push(SheetGrid {
                name: sheet_name.clone(),
                rows,
            });
        }
    }

    // Dosya bazlı deterministik sıra çözümü (T1..T5)
    let resolution = resolve_file_date_order(&grids, &file_name_hint);
    match &resolution {
        DateOrderResolution::Resolved(order) => {
            tracing::debug!("{}: tarih sırası çözüldü: {:?}", file_name_hint, order);
        }
        DateOrderResolution::Weak(order) => {
            tracing::warn!(
                "{}: tarih sırası kanıtı ZAYIF, varsayılan {:?} kullanılıyor (karar motoru şüphe olarak işaretleyecek)",
                file_name_hint,
                order
            );
        }
        DateOrderResolution::Conflict(detail) => {
            tracing::warn!(
                "{}: tarih sırası ÇELİŞKİSİ: {} (dosya karantinaya alınmalı)",
                file_name_hint,
                detail
            );
        }
    }
    diag.date_order = Some(resolution.clone());
    let date_order = resolution.order_or_default();

    // 2. geçiş: çözülen sırayla ayrıştır
    for grid in &grids {
        parse_grid_with_order(grid, db, &file_name_hint, date_order, &mut diag);
    }

    Ok(diag)
}
