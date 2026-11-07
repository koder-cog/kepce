use anyhow::{Context, Result, bail};
use bcrypt::{DEFAULT_COST, hash};
use chrono::Utc;
use clap::{Parser, Subcommand};
use rand::Rng;
use sea_orm::{ActiveModelTrait, ColumnTrait, Database, EntityTrait, QueryFilter, Set};
use shared::entities::users;
use std::io::{self, Read};

/// Kepçe Kullanıcı Kimlik ve Şifre Yönetim Aracı
#[derive(Parser)]
#[command(
    name = "set_user_password",
    about = "Kepçe Kullanıcı Kimlik, Şifre ve Kurtarma Anahtarı Yönetim Aracı",
    version = "0.6.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Kurtarma kodu bulunmayan (eski SSO) veya filtrelenen kullanıcıları listeler
    List {
        /// İsteğe bağlı kullanıcı adı arama filtresi (büyük/küçük harf duyarsız)
        #[arg(short, long)]
        query: Option<String>,
    },
    /// Kullanıcıya yeni şifre ve kurtarma kodu atar
    Set {
        /// Hedef kullanıcı adı (nickname)
        nickname: String,

        /// Şifreyi terminalden etkileşimli ve maskeli olarak girmek için
        #[arg(long, conflicts_with = "stdin")]
        prompt: bool,

        /// Şifreyi standart girdiden (stdin / pipe) okumak için
        #[arg(long, conflicts_with = "prompt")]
        stdin: bool,
    },
}

/// Tahmin edilebilir önek içermeyen, CSPRNG tabanlı, 24 karakterlik yüksek entropili parola üretir (~144 bit)
fn generate_high_entropy_password() -> String {
    const CHARSET: &[u8] = b"abcdefghijkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789!@#$%^&*-_+=?";
    let mut rng = rand::thread_rng();
    (0..24)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://kepce_admin:SIFRE@127.0.0.1:5432/kepce".to_string());

    let cli = Cli::parse();
    let db = Database::connect(&db_url).await.context(
        "Veritabanı bağlantısı kurulamadı. Lütfen DATABASE_URL ortam değişkenini kontrol edin.",
    )?;

    match cli.command {
        Commands::List { query } => {
            let mut select = users::Entity::find();

            if let Some(term) = query {
                let pattern = format!("%{}%", term.to_lowercase());
                select = select.filter(
                    sea_orm::sea_query::Expr::expr(sea_orm::sea_query::Func::lower(
                        sea_orm::sea_query::Expr::col(users::Column::Nickname),
                    ))
                    .like(&pattern),
                );
            } else {
                // Filtre verilmemişse kurtarma kodu bulunmayan (eski Google SSO vb.) hesapları getir
                select = select.filter(users::Column::RecoveryKeyHash.is_null());
            }

            let user_list = select.all(&db).await?;

            if user_list.is_empty() {
                println!("Kriterlere uyan kullanıcı bulunamadı.");
                return Ok(());
            }

            println!("\nBulunan Kullanıcılar ({} adet):", user_list.len());
            println!(
                "{:<38} | {:<25} | {:<10} | {:<12} | Kurtarma Kodu",
                "ID", "Kullanıcı Adı", "Rol", "Onaylı"
            );
            println!("{}", "-".repeat(95));

            for u in user_list {
                let has_recovery = if u.recovery_key_hash.is_some() {
                    "Mevcut"
                } else {
                    "YOK (Eski SSO)"
                };
                println!(
                    "{:<38} | {:<25} | {:<10?} | {:<12} | {}",
                    u.id, u.nickname, u.role, u.is_verified, has_recovery
                );
            }
            println!();
        }
        Commands::Set {
            nickname,
            prompt,
            stdin,
        } => {
            let lower_target = nickname.to_lowercase();

            let user_opt = users::Entity::find()
                .filter(
                    sea_orm::sea_query::Expr::expr(sea_orm::sea_query::Func::lower(
                        sea_orm::sea_query::Expr::col(users::Column::Nickname),
                    ))
                    .eq(&lower_target),
                )
                .one(&db)
                .await?;

            let user = match user_opt {
                Some(u) => u,
                None => {
                    bail!(
                        "'{}' kullanıcı adına sahip hesap veritabanında bulunamadı.",
                        nickname
                    );
                }
            };

            // Şifre alımı: ps aux veya shell geçmişine sızmayı önleyen güvenli akış
            let (raw_password, is_generated) = if prompt {
                let pass = rpassword::prompt_password("Yeni şifreyi giriniz (ekranda gizlenir): ")
                    .context("Şifre okunurken hata oluştu")?;
                if pass.trim().len() < 8 {
                    bail!("Şifre en az 8 karakter olmalıdır.");
                }
                (pass, false)
            } else if stdin {
                let mut buffer = String::new();
                io::stdin()
                    .read_to_string(&mut buffer)
                    .context("Standart girdiden (stdin) şifre okunamadı")?;
                let trimmed = buffer.trim_end_matches(&['\r', '\n'][..]).to_string();
                if trimmed.len() < 8 {
                    bail!("Stdin üzerinden gelen şifre en az 8 karakter olmalıdır.");
                }
                (trimmed, false)
            } else {
                // Varsayılan: CSPRNG ile 24 karakterlik yüksek entropili güvenli geçici şifre
                (generate_high_entropy_password(), true)
            };

            let password_hash =
                hash(&raw_password, DEFAULT_COST).context("Bcrypt şifre özeti oluşturulamadı")?;

            let mut active: users::ActiveModel = user.clone().into();
            active.password_hash = Set(password_hash);
            active.updated_at = Set(Some(Utc::now().into()));

            let mut generated_recovery_key = None;

            // Eğer kullanıcının Crockford kurtarma anahtarı yoksa (Google SSO geçişi) otomatik üret
            if user.recovery_key_hash.is_none() {
                let recovery_key = api::core::crypto::generate_secure_recovery_key();
                let full_hash = api::core::crypto::hash_recovery_key(&recovery_key);
                active.recovery_key_hash = Set(Some(full_hash));
                active.is_verified = Set(true);
                generated_recovery_key = Some(recovery_key);
            }

            active.update(&db).await?;

            println!("\n✔ Kullanıcı Hesabı Başarıyla Güncellendi");
            println!("========================================================");
            println!("Kullanıcı ID    : {}", user.id);
            println!("Kullanıcı Adı   : {}", user.nickname);
            if is_generated {
                println!("Üretilen Şifre  : {}", raw_password);
            } else {
                println!("Şifre Durumu    : Güvenli şekilde atandı (maskelendi)");
            }
            if let Some(ref rk) = generated_recovery_key {
                println!("Kurtarma Anahtarı: {}", rk);
            }
            println!("========================================================\n");
        }
    }

    Ok(())
}
