//! Moderatör ve yönetici veritabanı yönetim konsolu & SQL çalıştırıcı.
//!
//! Web üzerinden psql ihtiyacını ortadan kaldırarak güvenli tablo inceleme,
//! filtreleme, satır düzenleme/silme ve kontrollü serbest SQL sorgusu çalıştırma imkanı sunar.

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use sea_orm::{ConnectionTrait, Statement};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::{
    config::AppState, dto::user::UserRole, error::AppError, extractors::auth::AuthenticatedUser,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tables", get(get_tables))
        .route("/tables/:table", get(get_table_data))
        .route("/tables/:table/row", delete(delete_row).put(update_row))
        .route("/query", post(execute_query))
}

fn require_admin(user: &AuthenticatedUser) -> Result<(), AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "Yalnızca yöneticiler veritabanı konsoluna erişebilir.".into(),
        ));
    }
    Ok(())
}

fn sanitize_ident(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 63 {
        return Err(AppError::BadRequest(
            "Geçersiz tablo veya sütun adı.".into(),
        ));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AppError::BadRequest(
            "Tanımlayıcı yalnızca harf, rakam ve alt çizgi içerebilir.".into(),
        ));
    }
    Ok(trimmed.to_string())
}

#[derive(Serialize)]
pub struct TableSummaryDto {
    pub name: String,
    pub estimated_rows: i64,
    pub column_count: i32,
}

/// Tüm public tabloları ve tahmini satır sayılarını listeler
async fn get_tables(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<TableSummaryDto>>, AppError> {
    require_admin(&user)?;

    let sql = r#"
        SELECT 
            t.table_name,
            COALESCE(c.reltuples::bigint, 0) AS estimated_rows,
            (
                SELECT count(*) 
                FROM information_schema.columns col 
                WHERE col.table_schema = 'public' AND col.table_name = t.table_name
            )::int AS column_count
        FROM information_schema.tables t
        LEFT JOIN pg_class c ON c.relname = t.table_name
        WHERE t.table_schema = 'public' 
          AND t.table_type = 'BASE TABLE'
        ORDER BY t.table_name ASC;
    "#;

    let stmt = Statement::from_string(state.db.get_database_backend(), sql.to_string());
    let rows = state
        .db
        .query_all(stmt)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let name: String = row.try_get("", "table_name").unwrap_or_default();
        let estimated_rows: i64 = row.try_get("", "estimated_rows").unwrap_or(0);
        let column_count: i32 = row.try_get("", "column_count").unwrap_or(0);

        result.push(TableSummaryDto {
            name,
            estimated_rows: estimated_rows.max(0),
            column_count,
        });
    }

    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct TableDataQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
}

#[derive(Serialize)]
pub struct ColumnInfoDto {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
    pub is_primary_key: bool,
}

#[derive(Serialize)]
pub struct TableDataResponseDto {
    pub table_name: String,
    pub columns: Vec<ColumnInfoDto>,
    pub primary_keys: Vec<String>,
    pub total_rows: i64,
    pub page: u64,
    pub limit: u64,
    pub rows: Vec<serde_json::Value>,
}

/// Seçilen tablonun sütun meta verilerini ve sayfalanmış satırlarını döner
async fn get_table_data(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(table): Path<String>,
    Query(query): Query<TableDataQuery>,
) -> Result<Json<TableDataResponseDto>, AppError> {
    require_admin(&user)?;
    let clean_table = sanitize_ident(&table)?;

    // 1. Tablonun public şemasında var olduğunu doğrula
    let exists_sql = "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1);";
    let exists_stmt = Statement::from_sql_and_values(
        state.db.get_database_backend(),
        exists_sql,
        vec![clean_table.clone().into()],
    );
    let is_present = match state.db.query_one(exists_stmt).await {
        Ok(Some(row)) => row.try_get_by_index::<bool>(0).unwrap_or(false),
        _ => false,
    };
    if !is_present {
        return Err(AppError::NotFound("Tablo bulunamadı.".into()));
    }

    // 2. Sütun meta verilerini ve birincil anahtarları al
    let col_sql = r#"
        SELECT 
            c.column_name, 
            c.data_type, 
            (c.is_nullable = 'YES') AS is_nullable, 
            c.column_default,
            EXISTS (
                SELECT 1 FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu 
                  ON tc.constraint_name = kcu.constraint_name 
                  AND tc.table_schema = kcu.table_schema
                WHERE tc.constraint_type = 'PRIMARY KEY' 
                  AND tc.table_schema = 'public' 
                  AND tc.table_name = c.table_name 
                  AND kcu.column_name = c.column_name
            ) AS is_primary_key
        FROM information_schema.columns c
        WHERE c.table_schema = 'public' AND c.table_name = $1
        ORDER BY c.ordinal_position ASC;
    "#;

    let col_stmt = Statement::from_sql_and_values(
        state.db.get_database_backend(),
        col_sql,
        vec![clean_table.clone().into()],
    );
    let col_rows = state
        .db
        .query_all(col_stmt)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut columns = Vec::with_capacity(col_rows.len());
    let mut primary_keys = Vec::new();

    for r in col_rows {
        let name: String = r.try_get("", "column_name").unwrap_or_default();
        let data_type: String = r.try_get("", "data_type").unwrap_or_default();
        let is_nullable: bool = r.try_get("", "is_nullable").unwrap_or(true);
        let column_default: Option<String> = r.try_get("", "column_default").ok();
        let is_primary_key: bool = r.try_get("", "is_primary_key").unwrap_or(false);

        if is_primary_key {
            primary_keys.push(name.clone());
        }

        columns.push(ColumnInfoDto {
            name,
            data_type,
            is_nullable,
            column_default,
            is_primary_key,
        });
    }

    // 3. Toplam satır sayısı
    let count_sql = format!("SELECT count(*)::bigint FROM \"{}\";", clean_table);
    let count_stmt = Statement::from_string(state.db.get_database_backend(), count_sql);
    let total_rows: i64 = match state.db.query_one(count_stmt).await {
        Ok(Some(r)) => r.try_get_by_index::<i64>(0).unwrap_or(0),
        _ => 0,
    };

    // 4. Sıralama ve sayfalama
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(25).clamp(5, 100);
    let offset = (page - 1) * limit;

    let order_col = if let Some(ref sc) = query.sort_by {
        if columns.iter().any(|c| &c.name == sc) {
            sanitize_ident(sc)?
        } else if let Some(pk) = primary_keys.first() {
            pk.clone()
        } else {
            "1".to_string()
        }
    } else if let Some(pk) = primary_keys.first() {
        pk.clone()
    } else {
        "1".to_string()
    };

    let dir = if query.sort_dir.as_deref() == Some("asc") {
        "ASC"
    } else {
        "DESC"
    };

    let select_sql = format!(
        "SELECT to_jsonb(t) AS row_data FROM (SELECT * FROM \"{}\" ORDER BY \"{}\" {} LIMIT {} OFFSET {}) t;",
        clean_table, order_col, dir, limit, offset
    );

    let select_stmt = Statement::from_string(state.db.get_database_backend(), select_sql);
    let rows_data = state
        .db
        .query_all(select_stmt)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut rows = Vec::with_capacity(rows_data.len());
    for r in rows_data {
        if let Ok(val) = r.try_get::<serde_json::Value>("", "row_data") {
            rows.push(val);
        }
    }

    Ok(Json(TableDataResponseDto {
        table_name: clean_table,
        columns,
        primary_keys,
        total_rows,
        page,
        limit,
        rows,
    }))
}

#[derive(Deserialize)]
pub struct DeleteRowDto {
    pub pk_column: String,
    pub pk_value: serde_json::Value,
}

/// Tablodan belirli bir birincil anahtara sahip satırı siler
async fn delete_row(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(table): Path<String>,
    Json(payload): Json<DeleteRowDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let clean_table = sanitize_ident(&table)?;
    let clean_pk_col = sanitize_ident(&payload.pk_column)?;

    let val_str = match &payload.pk_value {
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        _ => {
            return Err(AppError::BadRequest(
                "Geçersiz birincil anahtar değeri.".into(),
            ))
        }
    };

    let del_sql = format!(
        "DELETE FROM \"{}\" WHERE \"{}\"::text = $1;",
        clean_table, clean_pk_col
    );

    let stmt = Statement::from_sql_and_values(
        state.db.get_database_backend(),
        &del_sql,
        vec![val_str.into()],
    );

    let res = state
        .db
        .execute(stmt)
        .await
        .map_err(|e| AppError::BadRequest(format!("Silme hatası: {}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "Satır silindi.",
        "affected_rows": res.rows_affected()
    })))
}

#[derive(Deserialize)]
pub struct UpdateRowDto {
    pub pk_column: String,
    pub pk_value: serde_json::Value,
    pub update_column: String,
    pub new_value: serde_json::Value,
}

/// Tabloda belirli bir satırın alanını günceller
async fn update_row(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(table): Path<String>,
    Json(payload): Json<UpdateRowDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let clean_table = sanitize_ident(&table)?;
    let clean_pk_col = sanitize_ident(&payload.pk_column)?;
    let clean_update_col = sanitize_ident(&payload.update_column)?;

    let pk_val_str = match &payload.pk_value {
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        _ => {
            return Err(AppError::BadRequest(
                "Geçersiz birincil anahtar değeri.".into(),
            ))
        }
    };

    let (update_sql, values) = if payload.new_value.is_null() {
        (
            format!(
                "UPDATE \"{}\" SET \"{}\" = NULL WHERE \"{}\"::text = $1;",
                clean_table, clean_update_col, clean_pk_col
            ),
            vec![pk_val_str.into()],
        )
    } else {
        let val_str = match &payload.new_value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            other => other.to_string(),
        };
        (
            format!(
                "UPDATE \"{}\" SET \"{}\" = $1 WHERE \"{}\"::text = $2;",
                clean_table, clean_update_col, clean_pk_col
            ),
            vec![val_str.into(), pk_val_str.into()],
        )
    };

    let stmt = Statement::from_sql_and_values(state.db.get_database_backend(), &update_sql, values);

    let res = state
        .db
        .execute(stmt)
        .await
        .map_err(|e| AppError::BadRequest(format!("Güncelleme hatası: {}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "Satır güncellendi.",
        "affected_rows": res.rows_affected()
    })))
}

#[derive(Deserialize)]
pub struct DatabaseQueryRequestDto {
    pub query: String,
    pub write_mode: Option<bool>,
}

#[derive(Serialize)]
pub struct QueryResultDto {
    pub columns: Vec<String>,
    pub rows: Vec<serde_json::Value>,
    pub affected_rows: u64,
    pub duration_ms: u128,
}

/// SQL Konsolu: Kontrollü serbest SQL sorgusu çalıştırma
async fn execute_query(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<DatabaseQueryRequestDto>,
) -> Result<Json<QueryResultDto>, AppError> {
    require_admin(&user)?;

    let trimmed = payload.query.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("Sorgu boş olamaz.".into()));
    }

    let write_mode = payload.write_mode.unwrap_or(false);

    // Salt okunur mod denetimi
    if !write_mode {
        let upper = trimmed.to_uppercase();
        let forbidden = [
            "INSERT", "UPDATE", "DELETE", "DROP", "ALTER", "TRUNCATE", "CREATE", "GRANT", "REVOKE",
        ];
        for kw in forbidden {
            let pattern = format!(r"\b{}\b", kw);
            if regex::Regex::new(&pattern)
                .map(|r| r.is_match(&upper))
                .unwrap_or(false)
            {
                return Err(AppError::BadRequest(format!(
                    "Salt okunur mod devrede. '{}' içeren veri değiştiren sorguları çalıştırmak için 'Yazma İzni' anahtarını açınız.",
                    kw
                )));
            }
        }
    }

    let start = Instant::now();
    let upper = trimmed.to_uppercase();
    let is_select =
        upper.starts_with("SELECT") || upper.starts_with("WITH") || upper.starts_with("EXPLAIN");

    // 5 saniye zaman aşımı kuralı
    let timeout_stmt = Statement::from_string(
        state.db.get_database_backend(),
        "SET statement_timeout = 5000;".to_string(),
    );
    let _ = state.db.execute(timeout_stmt).await;

    if is_select {
        // Satırları to_jsonb ile JSON olarak topla (maksimum 500 satır önlemi)
        let wrapped_sql = format!(
            "SELECT to_jsonb(t) AS row_data FROM ({}) t LIMIT 500;",
            trimmed.trim_end_matches(';')
        );

        let stmt = Statement::from_string(state.db.get_database_backend(), wrapped_sql);
        let rows_data = state
            .db
            .query_all(stmt)
            .await
            .map_err(|e| AppError::BadRequest(format!("SQL Yürütme Hatası: {}", e)))?;

        let duration_ms = start.elapsed().as_millis();

        let mut rows = Vec::with_capacity(rows_data.len());
        let mut columns = Vec::new();

        for (i, r) in rows_data.into_iter().enumerate() {
            if let Ok(val) = r.try_get::<serde_json::Value>("", "row_data") {
                if i == 0 {
                    if let Some(obj) = val.as_object() {
                        columns = obj.keys().cloned().collect();
                    }
                }
                rows.push(val);
            }
        }

        let total_count = rows.len() as u64;

        Ok(Json(QueryResultDto {
            columns,
            rows,
            affected_rows: total_count,
            duration_ms,
        }))
    } else {
        // Veri değiştiren DDL veya DML
        let stmt = Statement::from_string(state.db.get_database_backend(), trimmed.to_string());
        let res = state
            .db
            .execute(stmt)
            .await
            .map_err(|e| AppError::BadRequest(format!("SQL Yürütme Hatası: {}", e)))?;

        let duration_ms = start.elapsed().as_millis();

        Ok(Json(QueryResultDto {
            columns: vec!["affected_rows".to_string()],
            rows: vec![serde_json::json!({ "affected_rows": res.rows_affected() })],
            affected_rows: res.rows_affected(),
            duration_ms,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_ident_valid() {
        assert_eq!(sanitize_ident("menus").unwrap(), "menus");
        assert_eq!(
            sanitize_ident("contact_messages").unwrap(),
            "contact_messages"
        );
        assert_eq!(sanitize_ident("users_2026").unwrap(), "users_2026");
    }

    #[test]
    fn test_sanitize_ident_rejects_sql_injection() {
        assert!(sanitize_ident("users; DROP TABLE menus;").is_err());
        assert!(sanitize_ident("menus--").is_err());
        assert!(sanitize_ident("menus' OR '1'='1").is_err());
        assert!(sanitize_ident("").is_err());
        assert!(sanitize_ident(&"a".repeat(70)).is_err());
    }

    #[test]
    fn test_read_only_keyword_blocking() {
        let forbidden = [
            "INSERT", "UPDATE", "DELETE", "DROP", "ALTER", "TRUNCATE", "CREATE", "GRANT", "REVOKE",
        ];
        let test_query = "DELETE FROM users WHERE id = 1";
        let upper = test_query.to_uppercase();

        let mut matched = false;
        for kw in forbidden {
            let pattern = format!(r"\b{}\b", kw);
            if regex::Regex::new(&pattern).unwrap().is_match(&upper) {
                matched = true;
                break;
            }
        }
        assert!(matched);

        let safe_query = "SELECT * FROM users WHERE status = 'active'";
        let upper_safe = safe_query.to_uppercase();
        let mut safe_matched = false;
        for kw in forbidden {
            let pattern = format!(r"\b{}\b", kw);
            if regex::Regex::new(&pattern).unwrap().is_match(&upper_safe) {
                safe_matched = true;
                break;
            }
        }
        assert!(!safe_matched);
    }
}
