//! 5651 Sayılı Kanun Kapsamında Geçici RAM Tamponu (/dev/shm) ve Senkronizasyon Modülü.
//!
//! Bu modül, Fransa sunucusunda disk yazımlarını (SSD/HDD) 0 byte'ta tutmak için
//! log paketlerini yalnızca Linux tmpfs (`/dev/shm`) üzerinde tutar.
//! Türkiye sunucusundaki cron/pull servisi `X-Sync-Token` ile paketleri çeker ve onaylar (`ack`).

use chrono::Utc;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// 128 MB RAM tampon limitinin %85'i (Devre kesici eşiği: ~108 MB)
pub const CIRCUIT_BREAKER_MAX_BYTES: u64 = 108 * 1024 * 1024;

/// RAM tamponunda mevcut toplam dosya boyutunu hesaplar.
pub async fn get_ram_buffer_usage_bytes(ram_dir: &Path) -> u64 {
    let mut total: u64 = 0;
    if let Ok(mut entries) = tokio::fs::read_dir(ram_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(meta) = entry.metadata().await {
                total += if meta.is_file() { meta.len() } else { 0 };
            }
        }
    }
    total
}

/// Devre kesici kontrolü: RAM tamponu %85 kapasiteyi aşmışsa `true` döner.
/// Bu durumda API mutasyon isteklerini HTTP 503 ile reddeder.
pub async fn is_circuit_breaker_triggered(ram_dir: &Path) -> bool {
    let usage = get_ram_buffer_usage_bytes(ram_dir).await;
    usage >= CIRCUIT_BREAKER_MAX_BYTES
}

/// Trafik kaydını RAM tamponuna (`/dev/shm/kepce_traffic/`) şifreli olarak yazar.
pub async fn record_traffic_event(
    ram_dir: &Path,
    recipient_pubkey_hex: Option<&str>,
    ip_or_subnet: &str,
    action: &str,
    target: &str,
) -> anyhow::Result<()> {
    if is_circuit_breaker_triggered(ram_dir).await {
        anyhow::bail!("Circuit breaker triggered: 5651 RAM buffer exceeds 85% capacity (108MB)");
    }

    tokio::fs::create_dir_all(ram_dir).await?;

    let now = Utc::now();
    let log_line = format!(
        "{}\t{}\t{}\t{}\n",
        now.to_rfc3339(),
        ip_or_subnet,
        action,
        target
    );

    let sealed_bytes = if let Some(pubkey) = recipient_pubkey_hex {
        crate::core::crypto::seal_traffic_log(pubkey, &log_line)?.into_bytes()
    } else {
        // Test veya geliştirme ortamında public key tanımlı değilse doğrudan baytları yazar
        log_line.into_bytes()
    };

    let filename = format!(
        "{:020}_{}.zst.age",
        now.timestamp_nanos_opt().unwrap_or(0),
        Uuid::new_v4().simple()
    );
    let target_file = ram_dir.join(filename);
    tokio::fs::write(target_file, sealed_bytes).await?;

    Ok(())
}

/// Çekilmemiş en eski paketleri sıralı olarak okur ve birleştirilmiş toplu yük (batch) döndürür.
/// Dönüş: Option<(SonDosyaAdı/SeqId, BLAKE3BatchHash, TopluBaytlar)>
pub async fn pull_log_batch(
    ram_dir: &Path,
    max_files: usize,
) -> anyhow::Result<Option<(String, String, Vec<u8>)>> {
    if !ram_dir.exists() {
        return Ok(None);
    }

    let mut files: Vec<PathBuf> = Vec::new();
    let mut entries = tokio::fs::read_dir(ram_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let is_target = path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext == "age" || ext == "log");
        if is_target {
            files.push(path);
        }
    }

    if files.is_empty() {
        return Ok(None);
    }

    // Dosya adlarına göre zamansal artan sırada sırala
    files.sort();

    let batch_slice = &files[..files.len().min(max_files)];
    let last_seq_id = batch_slice
        .last()
        .and_then(|p| p.file_name())
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_string();

    let mut combined_bytes = Vec::new();
    for path in batch_slice {
        let content = tokio::fs::read(path).await?;
        // Paket uzunluk öneki (4-byte Big Endian) + içerik
        let len = (content.len() as u32).to_be_bytes();
        combined_bytes.extend_from_slice(&len);
        combined_bytes.extend_from_slice(&content);
    }

    let mut hasher = blake3::Hasher::new();
    hasher.update(&combined_bytes);
    let batch_hash = hasher.finalize().to_hex().to_string();

    Ok(Some((last_seq_id, batch_hash, combined_bytes)))
}

/// Türkiye sunucusu tarafından çekildiği onaylanan (`ack`) paketleri RAM tamponundan siler.
pub async fn ack_log_batch(ram_dir: &Path, up_to_seq_id: &str) -> anyhow::Result<usize> {
    if !ram_dir.exists() {
        return Ok(0);
    }

    let mut deleted_count = 0;
    let mut entries = tokio::fs::read_dir(ram_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let should_delete = path.is_file()
            && path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name <= up_to_seq_id);
        if should_delete {
            let _ = tokio::fs::remove_file(&path).await;
            deleted_count += 1;
        }
    }

    Ok(deleted_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_traffic_logger_lifecycle() {
        let temp_dir =
            std::env::temp_dir().join(format!("traffic_test_{}", Uuid::new_v4().simple()));
        tokio::fs::create_dir_all(&temp_dir).await.unwrap();

        // 1. İki log kaydı yaz
        record_traffic_event(&temp_dir, None, "192.168.1.0/24", "REGISTER", "user1")
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        record_traffic_event(&temp_dir, None, "192.168.1.0/24", "LOGIN", "user1")
            .await
            .unwrap();

        // 2. Batch pull yap
        let batch = pull_log_batch(&temp_dir, 10).await.unwrap();
        assert!(batch.is_some());
        let (seq_id, hash, bytes) = batch.unwrap();
        assert!(!seq_id.is_empty());
        assert_eq!(hash.len(), 64);
        assert!(!bytes.is_empty());

        // 3. Ack ile sil
        let deleted = ack_log_batch(&temp_dir, &seq_id).await.unwrap();
        assert_eq!(deleted, 2);

        // 4. Tekrar pull yapıldığında boş olmalı
        let empty_batch = pull_log_batch(&temp_dir, 10).await.unwrap();
        assert!(empty_batch.is_none());

        // Temizlik
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    }
}
