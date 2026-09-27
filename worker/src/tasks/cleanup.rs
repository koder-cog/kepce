//! Veri Saklama ve Dosya Temizlik Görevi (Retention & Storage Cleanup).
//!
//! Google Cloud OLM (Object Lifecycle Management) ve KVKK veri minimizasyonu
//! ilkeleri gereğince, karantina havuzunda bekleyen yetim dosyaları ve
//! reddedilen menü arşivinde saklama süresi dolan içerikleri periyodik olarak temizler.

use anyhow::Result;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Temizlik çalıştırma raporu.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RetentionReport {
    pub quarantine_files_deleted: usize,
    pub quarantine_bytes_freed: u64,
    pub rejected_files_deleted: usize,
    pub rejected_bytes_freed: u64,
    pub empty_dirs_removed: usize,
}

/// Ortam değişkenlerinden yapılandırmayı okuyarak süresi dolmuş dosyaları temizler.
///
/// Varsayılan TTL süreleri:
/// - Karantina: 30 gün (`RETENTION_QUARANTINE_DAYS`)
/// - Reddedilen Arşivi: 90 gün (`RETENTION_REJECTED_DAYS`)
pub async fn clean_expired_files(dry_run: bool) -> Result<RetentionReport> {
    let quarantine_base = std::env::var("MENU_QUARANTINE_DIR").unwrap_or_else(|_| {
        if Path::new("/app/uploads/quarantine/menus").exists() {
            "/app/uploads/quarantine/menus".to_string()
        } else if Path::new("uploads/quarantine/menus").exists() {
            "uploads/quarantine/menus".to_string()
        } else {
            "../uploads/quarantine/menus".to_string()
        }
    });

    let ingest_base = std::env::var("MENU_INGEST_DIR")
        .or_else(|_| std::env::var("WORKER_MENU_DIR"))
        .unwrap_or_else(|_| {
            if Path::new("/app/data/menuler").exists() {
                "/app/data/menuler".to_string()
            } else if Path::new("data/menuler").exists() {
                "data/menuler".to_string()
            } else {
                "../data/menuler".to_string()
            }
        });

    let rejected_base = format!("{}/reddedilen", ingest_base.trim_end_matches('/'));

    let quarantine_days: u64 = std::env::var("RETENTION_QUARANTINE_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);

    let rejected_days: u64 = std::env::var("RETENTION_REJECTED_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(90);

    let quarantine_ttl = Duration::from_secs(quarantine_days * 86400);
    let rejected_ttl = Duration::from_secs(rejected_days * 86400);
    // 5 dakikadan yeni açılmış boş dizinleri aktif yüklemelerle yarış koşulunu önlemek için koru
    let min_dir_age = Duration::from_secs(300);

    clean_expired_files_inner(
        Path::new(&quarantine_base),
        Path::new(&rejected_base),
        quarantine_ttl,
        rejected_ttl,
        min_dir_age,
        dry_run,
    )
    .await
}

/// Belirtilen dizinler ve TTL süreleri için temizlik mantığını yürütür.
pub async fn clean_expired_files_inner(
    quarantine_base: &Path,
    rejected_base: &Path,
    quarantine_ttl: Duration,
    rejected_ttl: Duration,
    min_dir_age: Duration,
    dry_run: bool,
) -> Result<RetentionReport> {
    let mut report = RetentionReport::default();
    let now = SystemTime::now();

    // 1. Karantina Temizliği (30 Günlük Yetim / İşlemsiz Yüklemeler)
    if quarantine_base.exists() {
        let (files_del, bytes_freed) =
            purge_directory_files(quarantine_base, quarantine_ttl, now, dry_run).await?;
        report.quarantine_files_deleted = files_del;
        report.quarantine_bytes_freed = bytes_freed;

        let empty_dirs = remove_empty_subdirectories(quarantine_base, min_dir_age, dry_run).await?;
        report.empty_dirs_removed += empty_dirs;
    }

    // 2. Reddedilen Dosyalar Temizliği (90 Günlük Soft Archive Süresi Dolanlar)
    if rejected_base.exists() {
        let (files_del, bytes_freed) =
            purge_directory_files(rejected_base, rejected_ttl, now, dry_run).await?;
        report.rejected_files_deleted = files_del;
        report.rejected_bytes_freed = bytes_freed;

        let empty_dirs = remove_empty_subdirectories(rejected_base, min_dir_age, dry_run).await?;
        report.empty_dirs_removed += empty_dirs;
    }

    tracing::info!(
        "[RETENTION-CLEANUP] Tamamlandı (dry_run: {}). Karantina: {} dosya ({} bayt), Reddedilen: {} dosya ({} bayt), Boş Dizin: {}",
        dry_run,
        report.quarantine_files_deleted,
        report.quarantine_bytes_freed,
        report.rejected_files_deleted,
        report.rejected_bytes_freed,
        report.empty_dirs_removed
    );

    Ok(report)
}

/// Bir kök dizin altındaki tüm dosyaları rekürsif tarar ve TTL süresini aşanları temizler.
/// Sembolik linkler dizin dışına sızmayı engellemek için kesinlikle takip edilmez.
async fn purge_directory_files(
    root: &Path,
    ttl: Duration,
    now: SystemTime,
    dry_run: bool,
) -> Result<(usize, u64)> {
    let mut files_deleted = 0;
    let mut bytes_freed = 0;

    let mut stack = vec![root.to_path_buf()];

    while let Some(current_dir) = stack.pop() {
        let mut reader = match tokio::fs::read_dir(&current_dir).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Dizin okunamadı ({:?}): {}", current_dir, e);
                continue;
            }
        };

        while let Ok(Some(entry)) = reader.next_entry().await {
            let path = entry.path();
            let file_type = match entry.file_type().await {
                Ok(ft) => ft,
                Err(e) => {
                    tracing::warn!("Dosya türü okunamadı ({:?}): {}", path, e);
                    continue;
                }
            };

            // Güvenlik: Sembolik linkler dizin dışına sızmayı engellemek için taranmaz
            if file_type.is_symlink() {
                tracing::debug!("Sembolik link tespit edildi, taranmıyor ({:?})", path);
                continue;
            }

            if file_type.is_dir() {
                stack.push(path);
            } else if file_type.is_file() {
                let metadata = match entry.metadata().await {
                    Ok(m) => m,
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::NotFound {
                            tracing::warn!("Dosya metaverisi okunamadı ({:?}): {}", path, e);
                        }
                        continue;
                    }
                };

                let modified = metadata.modified().unwrap_or(now);
                let age = now.duration_since(modified).unwrap_or(Duration::ZERO);

                if age >= ttl {
                    let file_size = metadata.len();
                    if !dry_run {
                        if let Err(e) = tokio::fs::remove_file(&path).await {
                            tracing::warn!(
                                "Zaman aşımına uğrayan dosya silinemedi ({:?}): {}",
                                path,
                                e
                            );
                            continue;
                        }
                    }
                    files_deleted += 1;
                    bytes_freed += file_size;
                }
            }
        }
    }

    Ok((files_deleted, bytes_freed))
}

/// Kök dizin altındaki boş alt dizinleri post-order (en derinden başlayarak) temizler.
/// Kök dizinin kendisi ve 5 dakikadan taze açılmış dizinler hiçbir zaman silinmez.
async fn remove_empty_subdirectories(
    root: &Path,
    min_dir_age: Duration,
    dry_run: bool,
) -> Result<usize> {
    let mut dirs_to_check = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    let now = SystemTime::now();

    // Önce tüm alt dizinleri topla (sembolik linkler hariç)
    while let Some(current_dir) = stack.pop() {
        let mut reader = match tokio::fs::read_dir(&current_dir).await {
            Ok(r) => r,
            Err(_) => continue,
        };

        while let Ok(Some(entry)) = reader.next_entry().await {
            if let Ok(file_type) = entry.file_type().await {
                if file_type.is_dir() && !file_type.is_symlink() {
                    let p = entry.path();
                    dirs_to_check.push(p.clone());
                    stack.push(p);
                }
            }
        }
    }

    // Derin dizinlerin önce işlenmesi için tersine sırala (post-order / bottom-up)
    dirs_to_check.sort_by_key(|a| std::cmp::Reverse(a.components().count()));

    let mut removed_count = 0;
    let mut simulated_removed: std::collections::HashSet<std::path::PathBuf> =
        std::collections::HashSet::new();

    for dir in dirs_to_check {
        if dir == root {
            continue;
        }

        // Yaş kontrolü: Aktif yüklemelerin yeni açtığı boş dizinleri yarış koşuluna karşı koru
        if min_dir_age > Duration::ZERO {
            if let Ok(metadata) = tokio::fs::metadata(&dir).await {
                let modified = metadata.modified().unwrap_or(now);
                let age = now.duration_since(modified).unwrap_or(Duration::ZERO);
                if age < min_dir_age {
                    continue;
                }
            }
        }

        // Dizin boş mu kontrol et (dry-run modunda simüle edilmiş silinmiş alt dizinleri yoksay)
        let is_empty = if let Ok(mut reader) = tokio::fs::read_dir(&dir).await {
            let mut empty = true;
            while let Ok(Some(child)) = reader.next_entry().await {
                let child_path = child.path();
                if dry_run && simulated_removed.contains(&child_path) {
                    continue;
                }
                empty = false;
                break;
            }
            empty
        } else {
            false
        };

        if is_empty {
            if !dry_run {
                if tokio::fs::remove_dir(&dir).await.is_ok() {
                    removed_count += 1;
                }
            } else {
                simulated_removed.insert(dir);
                removed_count += 1;
            }
        }
    }

    Ok(removed_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cleanup_quarantine_ttl_and_empty_dirs() {
        let base_temp =
            std::env::temp_dir().join(format!("kepce_test_cleanup_{}", uuid::Uuid::new_v4()));
        let quarantine = base_temp.join("quarantine");
        let rejected = base_temp.join("rejected");

        let stale_dir = quarantine.join("ankara").join("2026").join("stale_uuid");
        tokio::fs::create_dir_all(&stale_dir).await.unwrap();
        let stale_file = stale_dir.join("menu1.pdf");
        tokio::fs::write(&stale_file, b"eski dosya verisi 12345")
            .await
            .unwrap();

        // TTL süresini 0 saniye, min_dir_age 0 saniye vererek dosyanın hemen silinmesini sağla
        let report = clean_expired_files_inner(
            &quarantine,
            &rejected,
            Duration::from_secs(0),
            Duration::from_secs(3600),
            Duration::ZERO,
            false,
        )
        .await
        .unwrap();

        assert_eq!(report.quarantine_files_deleted, 1);
        assert!(report.quarantine_bytes_freed > 0);
        assert!(!stale_file.exists());
        assert!(!stale_dir.exists());

        let _ = tokio::fs::remove_dir_all(&base_temp).await;
    }

    #[tokio::test]
    async fn test_cleanup_dry_run_preserves_files_and_simulates_empty_dirs() {
        let base_temp =
            std::env::temp_dir().join(format!("kepce_test_cleanup_dry_{}", uuid::Uuid::new_v4()));
        let quarantine = base_temp.join("quarantine");
        let rejected = base_temp.join("rejected");

        let nested_dir = rejected.join("bursa").join("some_uuid");
        tokio::fs::create_dir_all(&nested_dir).await.unwrap();

        let report = clean_expired_files_inner(
            &quarantine,
            &rejected,
            Duration::from_secs(3600),
            Duration::from_secs(0),
            Duration::ZERO,
            true, // dry_run
        )
        .await
        .unwrap();

        // Dry-run aktifken iç içe dizinler (bursa ve some_uuid) simüle edilerek sayılmış olmalı
        assert_eq!(report.empty_dirs_removed, 2);
        // Ancak fiziksel olarak silinmemiş olmalı
        assert!(nested_dir.exists());
        assert!(rejected.join("bursa").exists());

        let _ = tokio::fs::remove_dir_all(&base_temp).await;
    }

    #[tokio::test]
    async fn test_cleanup_fresh_empty_dirs_are_preserved_against_race_condition() {
        let base_temp =
            std::env::temp_dir().join(format!("kepce_test_fresh_{}", uuid::Uuid::new_v4()));
        let quarantine = base_temp.join("quarantine");
        let rejected = base_temp.join("rejected");

        // Taze oluşturulmuş boş bir karantina yükleme klasörü
        let fresh_dir = quarantine.join("izmir").join("2026").join("fresh_uuid");
        tokio::fs::create_dir_all(&fresh_dir).await.unwrap();

        // min_dir_age = 300 saniye (5 dakika)
        let report = clean_expired_files_inner(
            &quarantine,
            &rejected,
            Duration::from_secs(0),
            Duration::from_secs(0),
            Duration::from_secs(300),
            false,
        )
        .await
        .unwrap();

        // 5 dakikadan yeni olduğu için silinmemeli
        assert_eq!(report.empty_dirs_removed, 0);
        assert!(fresh_dir.exists());

        let _ = tokio::fs::remove_dir_all(&base_temp).await;
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_cleanup_symlinks_are_not_traversed() {
        let base_temp =
            std::env::temp_dir().join(format!("kepce_test_symlink_{}", uuid::Uuid::new_v4()));
        let quarantine = base_temp.join("quarantine");
        let rejected = base_temp.join("rejected");
        let outside_dir = base_temp.join("outside_vault");

        tokio::fs::create_dir_all(&quarantine).await.unwrap();
        tokio::fs::create_dir_all(&rejected).await.unwrap();
        tokio::fs::create_dir_all(&outside_dir).await.unwrap();

        let target_file = outside_dir.join("critical_menu.pdf");
        tokio::fs::write(&target_file, b"disaridaki kritik veri")
            .await
            .unwrap();

        // Karantina içine dışarıdaki dizine işaret eden sembolik link oluştur
        let symlink_path = quarantine.join("symlink_to_outside");
        std::os::unix::fs::symlink(&outside_dir, &symlink_path).unwrap();

        let report = clean_expired_files_inner(
            &quarantine,
            &rejected,
            Duration::from_secs(0),
            Duration::from_secs(0),
            Duration::ZERO,
            false,
        )
        .await
        .unwrap();

        // Sembolik link takip edilip dışarıdaki dosya silinmemeli
        assert_eq!(report.quarantine_files_deleted, 0);
        assert!(target_file.exists());

        let _ = tokio::fs::remove_dir_all(&base_temp).await;
    }

    #[tokio::test]
    async fn test_cleanup_preserves_unexpired_files() {
        let base_temp = std::env::temp_dir().join(format!(
            "kepce_test_unexpired_{}",
            uuid::Uuid::new_v4()
        ));
        let quarantine = base_temp.join("quarantine");
        let rejected = base_temp.join("rejected");

        let active_dir = quarantine.join("antalya").join("2026").join("active_uuid");
        tokio::fs::create_dir_all(&active_dir).await.unwrap();
        let active_file = active_dir.join("today_menu.pdf");
        tokio::fs::write(&active_file, b"guncel ve silinmemesi gereken dosya")
            .await
            .unwrap();

        // 30 günlük TTL (dosya henüz yeni oluşturulduğu için süresi dolmamıştır)
        let report = clean_expired_files_inner(
            &quarantine,
            &rejected,
            Duration::from_secs(30 * 86400),
            Duration::from_secs(90 * 86400),
            Duration::from_secs(300),
            false,
        )
        .await
        .unwrap();

        assert_eq!(report.quarantine_files_deleted, 0);
        assert_eq!(report.quarantine_bytes_freed, 0);
        assert_eq!(report.empty_dirs_removed, 0);
        assert!(active_file.exists());
        assert!(active_dir.exists());

        let _ = tokio::fs::remove_dir_all(&base_temp).await;
    }

    #[tokio::test]
    async fn test_cleanup_handles_nonexistent_base_directories_gracefully() {
        let base_temp = std::env::temp_dir().join(format!(
            "kepce_test_nonexistent_{}",
            uuid::Uuid::new_v4()
        ));
        let nonexistent_quarantine = base_temp.join("no_quarantine");
        let nonexistent_rejected = base_temp.join("no_rejected");

        // Dizinler fiziksel olarak mevcut değilken hata patlamamalı, sıfır raporla dönmeli
        let report = clean_expired_files_inner(
            &nonexistent_quarantine,
            &nonexistent_rejected,
            Duration::from_secs(3600),
            Duration::from_secs(3600),
            Duration::ZERO,
            false,
        )
        .await
        .unwrap();

        assert_eq!(report.quarantine_files_deleted, 0);
        assert_eq!(report.rejected_files_deleted, 0);
        assert_eq!(report.empty_dirs_removed, 0);
    }
}

