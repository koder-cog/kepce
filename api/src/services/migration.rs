use sea_orm::{DatabaseConnection, ConnectionTrait, Statement};
use std::fs;
use std::path::{Path, PathBuf};

pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), anyhow::Error> {
    tracing::info!("Veritabanı migrasyon kontrolü başlatılıyor...");

    // 1. Ensure schema_migrations table exists
    let create_migrations_table_stmt = Statement::from_string(
        db.get_database_backend(),
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version VARCHAR(255) PRIMARY KEY,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );".to_string()
    );
    db.execute(create_migrations_table_stmt).await?;

    // 2. Locate migrations directory
    let migrations_dir = find_directory(&["db/migrations", "./db/migrations", "../db/migrations", "/app/db/migrations"])
        .ok_or_else(|| anyhow::anyhow!("Migrasyon dizini (db/migrations) bulunamadı!"))?;

    tracing::info!("Migrasyonlar dizinden yükleniyor: {:?}", migrations_dir);

    // Read and sort migration files
    let mut migration_files: Vec<PathBuf> = fs::read_dir(&migrations_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sql"))
        .collect();

    migration_files.sort();

    for file_path in migration_files {
        let file_name = match file_path.file_name().and_then(|s| s.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Check if migration has already been applied
        let check_stmt = Statement::from_string(
            db.get_database_backend(),
            format!("SELECT EXISTS (SELECT 1 FROM schema_migrations WHERE version = '{}');", file_name)
        );

        let is_applied = match db.query_one(check_stmt).await {
            Ok(Some(row)) => row.try_get_by_index::<bool>(0).unwrap_or(false),
            _ => false,
        };

        if !is_applied {
            tracing::info!("Migrasyon çalıştırılıyor: {}", file_name);
            execute_sql_file(db, &file_path).await?;

            let record_stmt = Statement::from_string(
                db.get_database_backend(),
                format!("INSERT INTO schema_migrations (version) VALUES ('{}');", file_name)
            );
            db.execute(record_stmt).await?;
            tracing::info!("Migrasyon başarıyla uygulandı ve kaydedildi: {}", file_name);
        } else {
            tracing::debug!("Migrasyon zaten uygulanmış, atlanıyor: {}", file_name);
        }
    }

    // 3. Check & Apply Prod Seeds if cities table is empty
    let check_cities_stmt = Statement::from_string(
        db.get_database_backend(),
        "SELECT COUNT(*) FROM cities;".to_string()
    );

    let cities_seeded = match db.query_one(check_cities_stmt).await {
        Ok(Some(row)) => row.try_get_by_index::<i64>(0).unwrap_or(0) > 0,
        _ => false,
    };

    if !cities_seeded {
        if let Some(seeds_dir) = find_directory(&["db/seeds/prod", "./db/seeds/prod", "../db/seeds/prod", "/app/db/seeds/prod"]) {
            tracing::info!("Veritabanı tohum verileri eksik. Prod seed'ler yükleniyor: {:?}", seeds_dir);
            let mut seed_files: Vec<PathBuf> = fs::read_dir(&seeds_dir)?
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sql"))
                .collect();

            seed_files.sort();

            for seed_file in seed_files {
                if let Some(name) = seed_file.file_name().and_then(|s| s.to_str()) {
                    tracing::info!("Seed çalıştırılıyor: {}", name);
                    execute_sql_file(db, &seed_file).await?;
                }
            }
            tracing::info!("Prod seed verileri başarıyla yüklendi.");
        } else {
            tracing::warn!("Prod seed dizini bulunamadı, tohumlama atlandı.");
        }
    }

    tracing::info!("Veritabanı migrasyon ve seed kontrolleri tamamlandı.");
    Ok(())
}

fn find_directory(candidates: &[&str]) -> Option<PathBuf> {
    for candidate in candidates {
        let path = Path::new(candidate);
        if path.exists() && path.is_dir() {
            return Some(path.to_path_buf());
        }
    }
    None
}

async fn execute_sql_file(db: &DatabaseConnection, path: &Path) -> Result<(), anyhow::Error> {
    let sql_content = fs::read_to_string(path)?;
    
    let mut current_stmt = String::new();
    let mut in_dollar_block = false;

    for line in sql_content.lines() {
        let trimmed = line.trim();
        // Skip comment lines if not in dollar block
        if !in_dollar_block && trimmed.starts_with("--") {
            continue;
        }
        if !in_dollar_block && trimmed.is_empty() {
            continue;
        }

        // $$ ile başlayan/biten blokları takip et
        let dollar_matches = line.matches("$$").count();
        if dollar_matches % 2 != 0 {
            in_dollar_block = !in_dollar_block;
        }
        
        current_stmt.push_str(line);
        current_stmt.push('\n');
        
        // Cümle ; ile bitiyorsa ve $$ bloğu içinde değilsek çalıştır
        if !in_dollar_block && trimmed.ends_with(';') {
            let stmt_str = current_stmt.trim();
            if !stmt_str.is_empty() {
                let stmt = Statement::from_string(db.get_database_backend(), stmt_str.to_string());
                db.execute(stmt).await?;
            }
            current_stmt.clear();
        }
    }
    
    // Execute any remaining statement
    let stmt_str = current_stmt.trim();
    if !stmt_str.is_empty() {
        let stmt = Statement::from_string(db.get_database_backend(), stmt_str.to_string());
        db.execute(stmt).await?;
    }
    
    Ok(())
}
