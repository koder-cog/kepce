use anyhow::Result;
use calamine::{open_workbook, Data, DataType, Reader, Xlsx};
use std::path::Path;

use crate::parser::models::MenuDatabase;
use crate::parser::validation;
use crate::parser::core::{SheetGrid, parse_grid};

pub fn parse_excel(path_str: &str, db: &mut MenuDatabase) -> Result<()> {
    let path = Path::new(path_str);
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let sheet_names = workbook.sheet_names().to_vec();

    if sheet_names.len() > validation::MAX_SHEET_COUNT {
        tracing::warn!("SKIP: {:?} has {} sheets (max {})", path.file_name().unwrap_or_default(), sheet_names.len(), validation::MAX_SHEET_COUNT);
        return Ok(());
    }

    let file_name_hint = path.file_name().unwrap_or_default().to_string_lossy();

    for sheet_name in sheet_names {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let height = range.height();
            let width = range.width();

            let mut rows = Vec::new();
            for r in 0..height {
                let mut row = Vec::new();
                for c in 0..width {
                    let cell = range.get((r, c)).unwrap_or(&Data::Empty);
                    
                    // Maintain Date formatting to match parse_date_string's expectation
                    if let Some(date) = cell.as_date() {
                        row.push(date.format("%d.%m.%Y").to_string());
                    } else {
                        row.push(cell.to_string());
                    }
                }
                rows.push(row);
            }

            let grid = SheetGrid {
                name: sheet_name.clone(),
                rows,
            };

            parse_grid(&grid, db, &file_name_hint);
        }
    }

    Ok(())
}
