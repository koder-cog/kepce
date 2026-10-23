use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFamily {
    Field,
    Kykyemek,
    Kykmenum,
    Yurtmenu,
    Community,
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustTier {
    Quarantined = 1,
    Untrusted = 2,
    Aggregator = 3,
    Community = 4,
    GroundTruth = 5,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SourceMetadata {
    pub label: String,
    pub family: SourceFamily,
    pub tier: TrustTier,
    pub priority: i32,
    pub is_auto_approvable: bool,
}

#[derive(Debug, Clone)]
pub struct StampedInput {
    pub file_path: std::path::PathBuf,
    pub file_hash: String,
    pub file_size_bytes: u64,
    pub source: SourceMetadata,
    pub attempt_count: u32,
    pub is_blacklisted: bool,
}

pub fn calculate_sha256(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut file = File::open(path)
        .with_context(|| format!("Hash hesaplanırken dosya açılamadı: {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let hash = hasher.finalize();
    Ok(hash.iter().map(|b| format!("{:02x}", b)).collect())
}

pub fn resolve_source(label: &str) -> SourceMetadata {
    let s = label.to_lowercase();
    if s.starts_with("kepce-admin") || s == "admin" {
        SourceMetadata {
            label: label.to_string(),
            family: SourceFamily::Field,
            tier: TrustTier::GroundTruth,
            priority: 100,
            is_auto_approvable: true,
        }
    } else if s.starts_with("community") || s.starts_with("kepce-user") {
        SourceMetadata {
            label: label.to_string(),
            family: SourceFamily::Community,
            tier: TrustTier::Community,
            priority: 75,
            is_auto_approvable: false,
        }
    } else if s.contains("kykyemek") {
        SourceMetadata {
            label: label.to_string(),
            family: SourceFamily::Kykyemek,
            tier: TrustTier::Aggregator,
            priority: 60,
            is_auto_approvable: true,
        }
    } else if s.contains("kykmenum") {
        SourceMetadata {
            label: label.to_string(),
            family: SourceFamily::Kykmenum,
            tier: TrustTier::Aggregator,
            priority: 60,
            is_auto_approvable: true,
        }
    } else if s.contains("yurtmenu") {
        SourceMetadata {
            label: label.to_string(),
            family: SourceFamily::Yurtmenu,
            tier: TrustTier::Aggregator,
            priority: 55,
            is_auto_approvable: false,
        }
    } else {
        SourceMetadata {
            label: if label.is_empty() { "anonymous".to_string() } else { label.to_string() },
            family: SourceFamily::Unknown,
            tier: TrustTier::Untrusted,
            priority: 10,
            is_auto_approvable: false,
        }
    }
}

const DEFAULT_ATTEMPT_HISTORY_FILE: &str = ".scratch/ocr_lab/outputs/.attempt_history.json";

fn resolve_history_path(output_dir: Option<&Path>) -> std::path::PathBuf {
    output_dir
        .map(|d| d.join(".attempt_history.json"))
        .unwrap_or_else(|| std::path::PathBuf::from(DEFAULT_ATTEMPT_HISTORY_FILE))
}

fn load_attempt_history(history_file: &Path) -> std::collections::HashMap<String, u32> {
    if let Ok(content) = std::fs::read_to_string(history_file) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        std::collections::HashMap::new()
    }
}

pub fn record_attempt(hash: &str, count: u32, history_file: &Path) {
    let mut history = load_attempt_history(history_file);
    history.insert(hash.to_string(), count);
    if let Ok(json) = serde_json::to_string(&history) {
        if let Some(parent) = history_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let temp_file = history_file.with_extension(format!("tmp.{}", std::process::id()));
        if std::fs::write(&temp_file, json).is_ok() {
            let _ = std::fs::rename(&temp_file, history_file);
        }
    }
}

pub fn stamp_file(
    file_path: &Path,
    source_label: Option<&str>,
    previous_attempts: u32,
    output_dir: Option<&Path>,
) -> Result<StampedInput> {
    let hash = calculate_sha256(file_path)?;
    let metadata = std::fs::metadata(file_path)
        .with_context(|| format!("Dosya meta verisi okunamadı: {}", file_path.display()))?;
    let source = resolve_source(source_label.unwrap_or("anonymous"));

    let history_file = resolve_history_path(output_dir);
    let history = load_attempt_history(&history_file);
    let recorded = history.get(&hash).copied().unwrap_or(0);
    let effective_previous = if previous_attempts > 0 {
        previous_attempts
    } else {
        recorded
    };

    let attempt_count = effective_previous + 1;
    let is_blacklisted = attempt_count >= 3;
    record_attempt(&hash, attempt_count, &history_file);

    Ok(StampedInput {
        file_path: file_path.to_path_buf(),
        file_hash: hash,
        file_size_bytes: metadata.len(),
        source,
        attempt_count,
        is_blacklisted,
    })
}
