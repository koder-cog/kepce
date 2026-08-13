use sea_orm::DbErr;

/// Merkezi veritabanı hata yakalama ve dönüştürme katmanı.
/// Veritabanından gelen sürücü tabanlı (driver-level) hata kodlarını
/// inceleyerek uygulamamıza özel anlamlı boolean veya enum değerlerine dönüştürür.
/// Böylece uygulamanın geri kalanı PostgreSQL veya SQLite kullanıldığından habersiz kalır.
pub fn is_unique_constraint_violation(err: &DbErr) -> bool {
    // SeaORM'un soyutladığı RuntimeErr üzerinden alt katmandaki Sqlx Error'a iniyoruz.
    match err {
        DbErr::Query(sea_orm::RuntimeErr::SqlxError(e))
        | DbErr::Exec(sea_orm::RuntimeErr::SqlxError(e)) => {
            if let Some(db_err) = e.as_database_error() {
                if let Some(code) = db_err.code() {
                    return match code.as_ref() {
                        "23505" => true, // PostgreSQL unique_violation
                        "2067"  => true, // SQLite SQLITE_CONSTRAINT_UNIQUE
                        "1555"  => true, // SQLite SQLITE_CONSTRAINT_PRIMARYKEY
                        "1062"  => true, // MySQL ER_DUP_ENTRY
                        _ => false,
                    };
                }
            }
            false
        }
        _ => false,
    }
}
