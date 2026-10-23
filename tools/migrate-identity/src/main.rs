//! Kepçe Açık E-posta -> HMAC-SHA256 Kör İndeks Göç Aracı
//! Bu araç veritabanındaki açık e-posta adreslerini normalize edip
//! `EMAIL_BLIND_KEY` ve `AUTH_PEPPER` anahtarları ile HMAC-SHA256 özetlerine dönüştürür.

use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
use sha2::Sha256;
use std::env;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// E-posta normalizasyonu:
/// 1. Baştaki ve sondaki boşluklar kırpılır, küçük harfe çevrilir.
/// 2. Gmail / Googlemail alan adlarında noktalar kaldırılır ve '+' uzantıları ayıklanır.
pub fn normalize_email(email: &str) -> String {
    let trimmed = email.trim().to_lowercase();
    if let Some((user, domain)) = trimmed.split_once('@') {
        if domain == "gmail.com" || domain == "googlemail.com" {
            let clean_user = user.replace('.', "");
            let base_user = clean_user.split('+').next().unwrap_or(&clean_user);
            format!("{}@{}", base_user, domain)
        } else {
            trimmed
        }
    } else {
        trimmed
    }
}

/// HMAC-SHA256 ile kör indeks özeti hesaplama
pub fn compute_hmac(input: &str, key_hex: &str) -> Result<String> {
    let key_bytes = hex::decode(key_hex.trim()).context("Anahtar hex formatında değil")?;
    let mut mac = HmacSha256::new_from_slice(&key_bytes)
        .map_err(|e| anyhow::anyhow!("HMAC anahtar hatası: {:?}", e))?;
    mac.update(input.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    println!("=== KEPÇE SIFIR BİLGİ KİMLİK GÖÇ ARACI ===");

    let database_url = env::var("DATABASE_URL")
        .context("DATABASE_URL ortam değişkeni ayarlanmalıdır")?;
    let email_blind_key = env::var("EMAIL_BLIND_KEY")
        .context("EMAIL_BLIND_KEY ortam değişkeni ayarlanmalıdır (64 hex)")?;
    let auth_pepper = env::var("AUTH_PEPPER")
        .context("AUTH_PEPPER ortam değişkeni ayarlanmalıdır (64 hex)")?;

    if email_blind_key.len() != 64 || auth_pepper.len() != 64 {
        anyhow::bail!("EMAIL_BLIND_KEY ve AUTH_PEPPER tam olarak 64 karakter (32 bayt) hex olmalıdır!");
    }

    println!("Veritabanına bağlanılıyor...");
    let db = Database::connect(&database_url).await
        .context("Veritabanı bağlantısı kurulamadı")?;

    // 1. Göç edilmemiş kayıtları tespit et
    let select_stmt = Statement::from_string(
        DatabaseBackend::Postgres,
        "SELECT id, email FROM users WHERE email IS NOT NULL AND (email_hash IS NULL OR email_hash = '');"
    );

    let rows = db.query_all(select_stmt).await
        .context("Kullanıcılar sorgulanamadı (0035 migration çalıştırıldı mı?)")?;

    println!("Göç edilecek kullanıcı sayısı: {}", rows.len());

    let mut migrated_count = 0;
    for row in rows {
        let user_id: Uuid = row.try_get("", "id")?;
        let plain_email: String = row.try_get("", "email")?;

        let norm_email = normalize_email(&plain_email);
        let email_hash = compute_hmac(&norm_email, &email_blind_key)?;
        let auth_hash = compute_hmac(&norm_email, &auth_pepper)?;

        let update_stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE users SET email_hash = $1, auth_hash = $2 WHERE id = $3;",
            vec![email_hash.into(), auth_hash.into(), user_id.into()],
        );

        db.execute(update_stmt).await
            .context(format!("Kullanıcı güncellenemedi: {}", user_id))?;

        migrated_count += 1;
    }

    println!("Başarıyla dönüştürülen satır sayısı: {}", migrated_count);

    // 2. Doğrulama: Kalan açık e-posta kontrolü
    let verify_stmt = Statement::from_string(
        DatabaseBackend::Postgres,
        "SELECT count(*) as unmigrated_count FROM users WHERE email IS NOT NULL AND (email_hash IS NULL OR email_hash = '');"
    );

    let verify_row = db.query_one(verify_stmt).await?
        .context("Doğrulama sorgusu yanıt vermedi")?;

    let unmigrated_count: i64 = verify_row.try_get("", "unmigrated_count")?;

    if unmigrated_count != 0 {
        anyhow::bail!("KRİTİK HATA: Göç tamamlanamadı! {} kullanıcı hala kör indekslenmedi.", unmigrated_count);
    }

    println!("Teyit: Göç edilmemiş kullanıcı sayısı KESİNLİKLE 0.");
    println!("0036 manuel DROP betiğini çalıştırmak artık güvenlidir.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_email() {
        assert_eq!(
            normalize_email("  Test.User+junk@gmail.com "),
            "testuser@gmail.com"
        );
        assert_eq!(
            normalize_email("Student.Kyk@googlemail.com"),
            "studentkyk@googlemail.com"
        );
        assert_eq!(
            normalize_email("ogrenci.123@itu.edu.tr"),
            "ogrenci.123@itu.edu.tr"
        );
    }

    #[test]
    fn test_compute_hmac() {
        let key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let hash1 = compute_hmac("test@example.com", key).unwrap();
        let hash2 = compute_hmac("test@example.com", key).unwrap();
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }
}
