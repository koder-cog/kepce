//! OpenRouter API bakiye, limit ve kota gözlemlenebilirlik servisi.
//!
//! `/api/v1/key` ve `/api/v1/credits` uç noktaları üzerinden harcama limitini
//! ve hesap bakiyesini sorgular; kritik seviyelerde `WARN` logları üretir.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterKeyResponse {
    pub data: OpenRouterKeyData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterKeyData {
    pub label: Option<String>,
    pub usage: Option<f64>,
    pub limit: Option<f64>,
    pub limit_remaining: Option<f64>,
    pub is_free_tier: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterCreditsResponse {
    pub data: OpenRouterCreditsData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterCreditsData {
    pub total_credits: f64,
    pub total_usage: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BalanceLevel {
    Normal,
    LowKeyLimit { remaining: f64, threshold: f64 },
    LowAccountCredits { balance: f64, threshold: f64 },
    Unknown,
}

#[derive(Debug, Clone)]
pub struct BalanceStatus {
    pub level: BalanceLevel,
    pub key_label: Option<String>,
    pub limit_remaining: Option<f64>,
    pub account_balance: Option<f64>,
}

/// Anahtar ve kredi verilerini verilen eşik değerlerine göre değerlendirir.
pub fn evaluate_balance(
    key_data: Option<&OpenRouterKeyData>,
    credits_data: Option<&OpenRouterCreditsData>,
    key_limit_warn_threshold: f64,
    account_credits_warn_threshold: f64,
) -> BalanceStatus {
    let limit_rem = key_data.and_then(|k| k.limit_remaining);
    let key_label = key_data.and_then(|k| k.label.clone());
    let account_balance = credits_data.map(|c| (c.total_credits - c.total_usage).max(0.0));

    let mut level = BalanceLevel::Normal;

    // 1. Anahtar üzerinde harcama limiti atanmışsa ve eşik değerinin altındaysa
    if let Some(rem) = limit_rem
        && rem < key_limit_warn_threshold
    {
        level = BalanceLevel::LowKeyLimit {
            remaining: rem,
            threshold: key_limit_warn_threshold,
        };
    }

    // 2. Anahtar limiti sınırsız olsa bile hesap bakiyesi eşik değerinin altındaysa
    if level == BalanceLevel::Normal
        && let Some(bal) = account_balance
        && bal < account_credits_warn_threshold
    {
        level = BalanceLevel::LowAccountCredits {
            balance: bal,
            threshold: account_credits_warn_threshold,
        };
    }

    BalanceStatus {
        level,
        key_label,
        limit_remaining: limit_rem,
        account_balance,
    }
}

/// OpenRouter uç noktalarından canlı bakiye ve kota durumunu çeker.
pub async fn check_openrouter_balance(
    client: &reqwest::Client,
    api_key: &str,
    base_url: Option<&str>,
) -> Option<BalanceStatus> {
    let root = base_url
        .unwrap_or("https://openrouter.ai/api/v1")
        .trim_end_matches('/');

    let key_url = format!("{}/key", root);
    let key_res = client
        .get(&key_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .ok()?;

    let key_data: Option<OpenRouterKeyData> = if key_res.status().is_success() {
        key_res
            .json::<OpenRouterKeyResponse>()
            .await
            .ok()
            .map(|r| r.data)
    } else {
        None
    };

    let credits_url = format!("{}/credits", root);
    let credits_res = client
        .get(&credits_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .ok()?;

    let credits_data: Option<OpenRouterCreditsData> = if credits_res.status().is_success() {
        credits_res
            .json::<OpenRouterCreditsResponse>()
            .await
            .ok()
            .map(|r| r.data)
    } else {
        None
    };

    let key_threshold: f64 = std::env::var("OPENROUTER_MIN_LIMIT_WARN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1.0);

    let credits_threshold: f64 = std::env::var("OPENROUTER_MIN_CREDITS_WARN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1.50);

    let status = evaluate_balance(
        key_data.as_ref(),
        credits_data.as_ref(),
        key_threshold,
        credits_threshold,
    );

    match &status.level {
        BalanceLevel::LowKeyLimit {
            remaining,
            threshold,
        } => {
            tracing::warn!(
                "[OPENROUTER] Anahtar harcama limiti kritik seviyede: ${:.2} (Uyarı eşiği: ${:.2}, Anahtar: {:?})",
                remaining,
                threshold,
                status.key_label
            );
        }
        BalanceLevel::LowAccountCredits { balance, threshold } => {
            tracing::warn!(
                "[OPENROUTER] Hesap toplam bakiyesi kritik seviyede: ${:.2} (Uyarı eşiği: ${:.2})",
                balance,
                threshold
            );
        }
        BalanceLevel::Normal => {
            tracing::info!(
                "[OPENROUTER] Bakiye durumu olağan. Kalan anahtar limiti: {:?}, Net hesap kredisi: {:?}",
                status.limit_remaining,
                status.account_balance
            );
        }
        BalanceLevel::Unknown => {}
    }

    Some(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_balance_low_key_limit() {
        let key = OpenRouterKeyData {
            label: Some("test-key".to_string()),
            usage: Some(5.0),
            limit: Some(10.0),
            limit_remaining: Some(0.45),
            is_free_tier: Some(false),
        };
        let credits = OpenRouterCreditsData {
            total_credits: 50.0,
            total_usage: 10.0,
        };

        let status = evaluate_balance(Some(&key), Some(&credits), 1.0, 1.5);
        assert_eq!(
            status.level,
            BalanceLevel::LowKeyLimit {
                remaining: 0.45,
                threshold: 1.0
            }
        );
        assert_eq!(status.limit_remaining, Some(0.45));
    }

    #[test]
    fn test_evaluate_balance_low_account_credits_when_limit_unrestricted() {
        let key = OpenRouterKeyData {
            label: Some("unlimited-key".to_string()),
            usage: Some(25.0),
            limit: None,
            limit_remaining: None,
            is_free_tier: Some(false),
        };
        let credits = OpenRouterCreditsData {
            total_credits: 45.0,
            total_usage: 44.1, // Net bakiye: 0.90 USD
        };

        let status = evaluate_balance(Some(&key), Some(&credits), 1.0, 1.5);
        assert!(matches!(
            status.level,
            BalanceLevel::LowAccountCredits { balance, threshold } if (balance - 0.9).abs() < 1e-4 && threshold == 1.5
        ));
    }

    #[test]
    fn test_evaluate_balance_normal() {
        let key = OpenRouterKeyData {
            label: Some("prod-key".to_string()),
            usage: Some(10.0),
            limit: Some(50.0),
            limit_remaining: Some(40.0),
            is_free_tier: Some(false),
        };
        let credits = OpenRouterCreditsData {
            total_credits: 100.0,
            total_usage: 15.0,
        };

        let status = evaluate_balance(Some(&key), Some(&credits), 1.0, 1.5);
        assert_eq!(status.level, BalanceLevel::Normal);
        assert_eq!(status.account_balance, Some(85.0));
    }
}
