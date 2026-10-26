//! Kriptografik Yardımcılar
//!
//! 1. 125-bit Crockford Base32 Kurtarma Anahtarı üretimi (5 blok x 5 karakter)
//! 2. Domain separation (BLAKE3) ile kurtarma anahtarı özetleme
//! 3. 5651 Sayılı Kanun gereği RAM'de tutulan X25519 SealedBox trafik log mühürleme
//! 4. Alt ağ (subnet) kör indeksleme (HMAC-SHA256)

use anyhow::{Context, Result};
use blake3::Hasher;
use crypto_box::{PublicKey, aead::OsRng};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Crockford Base32 alfabesi (O, I, L, U çıkarılmıştır - karışıklık ve küfür önleme)
const CROCKFORD_CHARS: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// 125-bit güvenli, 5 bloklu (5-5-5-5-5) tek seferlik kurtarma anahtarı üretir.
/// Örnek çıktı: kpc-7x9mw-4q2k8-nb3vz-pr6ty-9ck2d
pub fn generate_secure_recovery_key() -> String {
    // 25 karakter x 5 bit = 125 bit entropi (her blok tam 5 karakter)
    let mut random_bytes = [0u8; 25];
    rand::thread_rng().fill_bytes(&mut random_bytes);

    let mut encoded = String::with_capacity(34);
    encoded.push_str("kpc-");

    for (i, byte) in random_bytes.iter().enumerate() {
        if i > 0 && i % 5 == 0 {
            encoded.push('-');
        }
        let idx = (byte % 32) as usize;
        encoded.push(CROCKFORD_CHARS[idx] as char);
    }
    encoded.to_lowercase()
}

/// Domain separation ile BLAKE3 kurtarma anahtarı özeti
pub fn hash_recovery_key(key: &str) -> String {
    let mut hasher = Hasher::new_derive_key("kepce.org 2026 recovery key v1");
    hasher.update(key.trim().to_lowercase().as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Son bloğun (5 karakter) aktif teyidi için BLAKE3 özeti
pub fn hash_recall_block(block: &str) -> String {
    let mut hasher = Hasher::new_derive_key("kepce.org 2026 recall block v1");
    hasher.update(block.trim().to_lowercase().as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Kurtarma anahtarından son 5 karakterlik bloğu çeker
pub fn extract_last_block(key: &str) -> Option<&str> {
    key.trim().split('-').last()
}

/// Alt ağ kör indeksi (KYK WiFi / CGNAT mahremiyeti için)
pub fn compute_subnet_hash(subnet_prefix: &str, blind_key_hex: &str) -> Result<String> {
    let key_bytes = hex::decode(blind_key_hex.trim()).context("Anahtar hex formatında değil")?;
    let mut mac = HmacSha256::new_from_slice(&key_bytes)
        .map_err(|e| anyhow::anyhow!("HMAC anahtar hatası: {:?}", e))?;
    mac.update(subnet_prefix.trim().as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

/// 5651 Sayılı Kanun gereği RAM'de tutulan trafik bilgisini
/// Fransa sunucusunda özel anahtarı bulunmayan X25519 açık anahtarıyla mühürler.
pub fn seal_traffic_log(public_key_hex: &str, payload_json: &str) -> Result<String> {
    let pk_bytes_vec =
        hex::decode(public_key_hex.trim()).context("Açık anahtar hex formatında değil")?;
    let pk_bytes: [u8; 32] = pk_bytes_vec
        .try_into()
        .map_err(|_| anyhow::anyhow!("Açık anahtar tam 32 bayt (64 hex) olmalıdır"))?;

    let public_key = PublicKey::from(pk_bytes);
    let ciphertext = public_key
        .seal(&mut OsRng, payload_json.as_bytes())
        .map_err(|e| anyhow::anyhow!("SealedBox mühürleme hatası: {:?}", e))?;

    Ok(hex::encode(ciphertext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_key_format() {
        let key = generate_secure_recovery_key();
        assert!(key.starts_with("kpc-"));
        let parts: Vec<&str> = key.split('-').collect();
        assert_eq!(parts.len(), 6);
        assert_eq!(parts[0], "kpc");
        assert_eq!(parts[1].len(), 5);
        assert_eq!(parts[2].len(), 5);
        assert_eq!(parts[3].len(), 5);
        assert_eq!(parts[4].len(), 5);
        assert_eq!(parts[5].len(), 5);
    }

    #[test]
    fn test_recovery_key_hashing() {
        let key1 = "kpc-7x9mw-4q2k8-nb3vz-pr6ty-9ck2d";
        let key2 = "  KPC-7X9MW-4Q2K8-NB3VZ-PR6TY-9CK2D  ";
        let hash1 = hash_recovery_key(key1);
        let hash2 = hash_recovery_key(key2);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_seal_roundtrip_with_secret_key() {
        let secret_key = crypto_box::SecretKey::generate(&mut OsRng);
        let public_key = secret_key.public_key();
        let pk_hex = hex::encode(public_key.as_bytes());

        let payload = r#"{"ip":"192.168.1.5","action":"comment"}"#;
        let sealed_hex = seal_traffic_log(&pk_hex, payload).unwrap();

        let ciphertext = hex::decode(&sealed_hex).unwrap();
        let unsealed_bytes = secret_key.unseal(&ciphertext).unwrap();
        let unsealed_str = String::from_utf8(unsealed_bytes).unwrap();

        assert_eq!(payload, unsealed_str);
    }
}
