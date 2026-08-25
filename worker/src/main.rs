use std::env;
use std::time::Duration;
use sea_orm::{Database, DbConn};

pub mod tasks;
pub mod parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Kepçe Distributed Ingestion Worker başlatılıyor...");

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db: DbConn = Database::connect(&db_url).await?;
    tracing::info!("Veritabanı bağlantısı başarılı.");

    // One-shot lokal dosya ingest (admin/kullanıcı Excel-PDF drop-zone).
    // Triggered only when WORKER_LOCAL_INGEST is set; safe to re-run.
    // Başarılı dosyalar vault'a taşınır, hatalılar hatali/ klasörüne düşer.
    if std::env::var("WORKER_LOCAL_INGEST").is_ok() {
        tracing::info!("[LOKAL] Tek seferlik lokal dosya ingest başlatılıyor...");
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(600))
            .build()?;
        let gemini_key = env::var("GEMINI_API_KEY").ok();
        if let Err(e) = tasks::file_ingest::process_local_files(&db, &client, gemini_key.as_deref())
            .await
        {
            tracing::error!("[LOKAL] Lokal dosya ingest hatası: {:?}", e);
        }
        if std::env::var("WORKER_ONESHOT").is_ok() {
            tracing::info!("[LOKAL] Tek seferlik lokal dosya aktarımı tamamlandı. Çıkış yapılıyor.");
            return Ok(());
        }
    }

    // One-shot backup ingestion (kykyemek-şmnmh-yedek/). Idempotent upsert.
    // Triggered only when WORKER_BACKUP_INGEST is set; safe to re-run.
    if std::env::var("WORKER_BACKUP_INGEST").is_ok() {
        let backup_dir = std::env::var("WORKER_BACKUP_DIR")
            .unwrap_or_else(|_| "kykyemek-şmnmh-yedek".to_string());
        tracing::info!("[BACKUP] Backup menü ingest başlatılıyor: {}", backup_dir);
        if let Err(e) = tasks::backup_ingest::ingest_backup_menus(&db, &backup_dir).await {
            tracing::error!("[BACKUP] Backup menü ingest hatası: {:?}", e);
        } else {
            tracing::info!("[BACKUP] Backup menü ingest tamamlandı.");
        }
        if std::env::var("WORKER_ONESHOT").is_ok() {
            tracing::info!("[BACKUP] Tek seferlik yedek aktarımı tamamlandı. Çıkış yapılıyor.");
            return Ok(());
        }
    }

    if std::env::var("WORKER_RECATEGORIZE").is_ok() {
        tracing::info!("[RECATEGORIZE] Yemek kategorileri yeniden sınıflandırılıyor...");
        if let Err(e) = tasks::historical_ingest::recategorize_all_dishes(&db).await {
            tracing::error!("[RECATEGORIZE] Kategori güncelleme hatası: {:?}", e);
        }
        if std::env::var("WORKER_ONESHOT").is_ok() && std::env::var("WORKER_HISTORICAL_INGEST").is_err() {
            tracing::info!("[RECATEGORIZE] Tek seferlik kategori güncellemesi tamamlandı. Çıkış yapılıyor.");
            return Ok(());
        }
    }

    if std::env::var("WORKER_HISTORICAL_INGEST").is_ok() {
        let historical_file = std::env::var("WORKER_HISTORICAL_FILE")
            .unwrap_or_else(|_| ".scratch/archive/historical_menus/unified/master_historical_menus.json".to_string());
        tracing::info!("[HISTORICAL] Tarihsel menü Worker ingest başlatılıyor: {}", historical_file);
        if let Err(e) = tasks::historical_ingest::ingest_historical_menus(&db, &historical_file).await {
            tracing::error!("[HISTORICAL] Tarihsel menü ingest hatası: {:?}", e);
        } else {
            tracing::info!("[HISTORICAL] Tarihsel menü ingest tamamlandı.");
        }
        if std::env::var("WORKER_ONESHOT").is_ok() {
            tracing::info!("[HISTORICAL] Tek seferlik tarihsel menü aktarımı tamamlandı. Çıkış yapılıyor.");
            return Ok(());
        }
    }

    if std::env::var("WORKER_BACKUP_EXPORT").is_ok() {
        let export_dir = std::env::var("WORKER_BACKUP_DIR")
            .unwrap_or_else(|_| "kykyemek-şmnmh-yedek-export".to_string());
        tracing::info!("[BACKUP] Backup menü export başlatılıyor: {}", export_dir);
        if let Err(e) = tasks::backup_export::export_backup_menus(&db, &export_dir).await {
            tracing::error!("[BACKUP] Backup menü export hatası: {:?}", e);
        } else {
            tracing::info!("[BACKUP] Backup menü export tamamlandı.");
        }
        if std::env::var("WORKER_ONESHOT").is_ok() {
            tracing::info!("[BACKUP] Tek seferlik yedek dışa aktarımı tamamlandı. Çıkış yapılıyor.");
            return Ok(());
        }
    }
    
    let gemini_api_key = env::var("GEMINI_API_KEY").ok();
    if gemini_api_key.is_none() {
        tracing::warn!("GEMINI_API_KEY bulunamadı. PDF ayrıştırma devre dışı bırakılacak.");
    }
    
    let reqwest_client = reqwest::Client::builder()
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(600))
        .build()?;

    // Gemini model erişilebilirlik kontrolü (startup)
    if let Some(ref api_key) = gemini_api_key {
        let model_name = env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-flash-latest".to_string());
        let check_url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}",
            model_name
        );
        match reqwest_client.get(&check_url)
            .header("x-goog-api-key", api_key)
            .send()
            .await
        {
            Ok(res) if res.status().is_success() => {
                tracing::info!("Gemini model '{}' erişilebilir [OK]", model_name);
            }
            Ok(res) => {
                tracing::warn!(
                    "Gemini model '{}' erişilebilirlik kontrolü başarısız (HTTP {}). PDF parsing çalışmayabilir.",
                    model_name, res.status()
                );
            }
            Err(e) => {
                tracing::warn!(
                    "Gemini model '{}' erişilebilirlik kontrolü başarısız: {:?}. PDF parsing çalışmayabilir.",
                    model_name, e
                );
            }
        }
    }

    let local_interval_secs: u64 = env::var("WORKER_LOCAL_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3600); // 1 saat

    let scraper_interval_secs: u64 = env::var("WORKER_SCRAPER_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(12 * 60 * 60); // 12 saat

    // Graceful shutdown channel
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    let db_local = db.clone();
    let client_local = reqwest_client.clone();
    let gemini_key_local = gemini_api_key.clone();
    let mut rx_local = shutdown_rx.clone();

    // Lokal dosya işleme döngüsü (Excel/PDF)
    let local_task = tokio::spawn(async move {
        loop {
            if *rx_local.borrow() {
                tracing::info!("[LOKAL] Kapatma sinyali algılandı. Döngüden çıkılıyor.");
                break;
            }
            tracing::info!("--- [LOKAL] DOSYA İŞLEME DÖNGÜSÜ BAŞLIYOR ---");
            if let Err(e) = tasks::file_ingest::process_local_files(&db_local, &client_local, gemini_key_local.as_deref()).await {
                tracing::error!("[LOKAL] Dosya taramasında hata: {:?}", e);
            }
            tracing::info!("--- [LOKAL] DÖNGÜ TAMAMLANDI. Bekleme: {}s ---", local_interval_secs);
            
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(local_interval_secs)) => {},
                _ = rx_local.changed() => {
                    tracing::info!("[LOKAL] Uyku sırasında kapatma sinyali alındı. Döngüden çıkılıyor.");
                    break;
                }
            }
        }
    });

    let db_scraper = db.clone();
    let client_scraper = reqwest_client.clone();
    let mut rx_scraper = shutdown_rx.clone();

    // Kykyemek Scraper döngüsü
    let scraper_task = tokio::spawn(async move {
        loop {
            if *rx_scraper.borrow() {
                tracing::info!("[WEB] Kapatma sinyali algılandı. Döngüden çıkılıyor.");
                break;
            }
            tracing::info!("--- [WEB] KYKYEMEK SCRAPER DÖNGÜSÜ BAŞLIYOR ---");
            if let Err(e) = tasks::scraper::run_kykyemek_scraper(&db_scraper, &client_scraper, rx_scraper.clone()).await {
                tracing::error!("[WEB] Kykyemek taramasında hata: {:?}", e);
            }
            tracing::info!("--- [WEB] DÖNGÜ TAMAMLANDI. Bekleme: {}s ---", scraper_interval_secs);
            
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(scraper_interval_secs)) => {},
                _ = rx_scraper.changed() => {
                    tracing::info!("[WEB] Uyku sırasında kapatma sinyali alındı. Döngüden çıkılıyor.");
                    break;
                }
            }
        }
    });

    let db_notifier = db.clone();
    let mut rx_notifier = shutdown_rx.clone();

    // Öğün Bildirimi Döngüsü (Her 60 saniyede bir saat kontrolü)
    let notifier_task = tokio::spawn(async move {
        loop {
            if *rx_notifier.borrow() {
                tracing::info!("[NOTIFIER] Kapatma sinyali algılandı. Döngüden çıkılıyor.");
                break;
            }
            if let Err(e) = tasks::meal_notifier::check_and_dispatch_meal_notifications(&db_notifier).await {
                tracing::error!("[NOTIFIER] Öğün bildirimi tetiklemesinde hata: {:?}", e);
            }

            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(60)) => {},
                _ = rx_notifier.changed() => {
                    tracing::info!("[NOTIFIER] Uyku sırasında kapatma sinyali alındı. Döngüden çıkılıyor.");
                    break;
                }
            }
        }
    });

    // Graceful Shutdown dinleyicisi
    let shutdown_signal = async {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate()).unwrap();
            let mut sigint = signal(SignalKind::interrupt()).unwrap();
            tokio::select! {
                _ = sigterm.recv() => {
                    tracing::info!("SIGTERM (Kapatma sinyali) alındı.");
                }
                _ = sigint.recv() => {
                    tracing::info!("SIGINT (Ctrl+C sinyali) alındı.");
                }
            }
        }
        #[cfg(not(unix))]
        {
            let _ = tokio::signal::ctrl_c().await;
            tracing::info!("Kapatma sinyali alındı.");
        }
    };

    shutdown_signal.await;
    tracing::info!("Kapatma sinyali alındı. Çalışan döngüler tamamlandıktan sonra çıkış yapılacak...");
    
    // Kapatma sinyali gönder
    let _ = shutdown_tx.send(true);

    // Görevlerin bitmesini bekle
    let _ = tokio::join!(local_task, scraper_task, notifier_task);
    tracing::info!("Tüm görevler başarıyla durduruldu. Worker güvenli bir şekilde kapatıldı.");

    Ok(())
}
