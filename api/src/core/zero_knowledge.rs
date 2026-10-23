//! Zero-Knowledge Kriptografik Yardımcı Modülü
//!
//! 1. E-posta kör indeksleme (HMAC-SHA256)
//! 2. Peppered kimlik doğrulama özeti (HMAC-SHA256)
//! 3. Alt ağ (subnet) kör indeksleme
//! 4. 5651 sayılı kanun gereği RAM'de mühürlenen X25519 SealedBox trafik kayıtları
//! 5. E-posta maskeleme

use anyhow::{Context, Result};
use crypto_box::{PublicKey, aead::OsRng};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// E-posta normalizasyonu:
/// Gmail ve Googlemail adreslerinde noktaları atlar, '+' sonrasını kırpar.
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

/// HMAC-SHA256 ile kör indeksleme
pub fn compute_hmac(input: &str, key_hex: &str) -> Result<String> {
    let key_bytes = hex::decode(key_hex.trim()).context("Anahtar hex formatında değil")?;
    let mut mac = HmacSha256::new_from_slice(&key_bytes)
        .map_err(|e| anyhow::anyhow!("HMAC anahtar hatası: {:?}", e))?;
    mac.update(input.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

/// E-posta kör indeksi (veritabanında kullanıcı aramak için)
pub fn compute_email_hash(email: &str, blind_key_hex: &str) -> Result<String> {
    let normalized = normalize_email(email);
    compute_hmac(&normalized, blind_key_hex)
}

/// Kimlik doğrulama özeti (auth_hash: parola eşleme ve oturum açma için)
pub fn compute_auth_hash(email: &str, pepper_hex: &str) -> Result<String> {
    let normalized = normalize_email(email);
    compute_hmac(&normalized, pepper_hex)
}

/// Alt ağ kör indeksi (KYK WiFi / CGNAT mahremiyeti için)
pub fn compute_subnet_hash(subnet_prefix: &str, blind_key_hex: &str) -> Result<String> {
    compute_hmac(subnet_prefix.trim(), blind_key_hex)
}

/// E-posta maskeleme (istemciye ipucu olarak döndürmek için: o***r@g***l.com)
pub fn mask_email(email: &str) -> String {
    let trimmed = email.trim();
    if let Some((user, domain)) = trimmed.split_once('@') {
        let masked_user = if user.len() <= 2 {
            format!("{}***", &user[..1.min(user.len())])
        } else {
            format!("{}***{}", &user[..1], &user[user.len() - 1..])
        };

        if let Some((d_name, d_ext)) = domain.split_once('.') {
            let masked_domain = if d_name.len() <= 2 {
                format!("{}***", &d_name[..1.min(d_name.len())])
            } else {
                format!("{}***{}", &d_name[..1], &d_name[d_name.len() - 1..])
            };
            format!("{}@{}.{}", masked_user, masked_domain, d_ext)
        } else {
            format!("{}@{}", masked_user, domain)
        }
    } else {
        "***@***.***".to_string()
    }
}

/// 5651 Sayılı Kanun gereği RAM'de tutulan trafik bilgisini
/// Fransa sunucusunda özel anahtarı bulunmayan X25519 açık anahtarıyla mühürler.
pub fn seal_traffic_log(public_key_hex: &str, payload_json: &str) -> Result<String> {
    let pk_bytes_vec = hex::decode(public_key_hex.trim()).context("Açık anahtar hex formatında değil")?;
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
    fn test_normalize_email() {
        assert_eq!(
            normalize_email("Student.Kyk+spam@Gmail.com"),
            "studentkyk@gmail.com"
        );
        assert_eq!(
            normalize_email("normal.email@domain.com"),
            "normal.email@domain.com"
        );
    }

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("omer@gmail.com"), "o***r@g***l.com");
        assert_eq!(mask_email("ab@cd.com"), "a***@c***.com");
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
