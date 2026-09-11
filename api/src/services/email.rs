use reqwest::Client;
use serde::Serialize;

#[derive(Debug)]
pub enum EmailError {
    ReqwestError(reqwest::Error),
    ApiError(String),
}

#[derive(Serialize)]
struct ResendRequest {
    from: String,
    to: Vec<String>,
    subject: String,
    html: String,
}

#[derive(Clone, Debug)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Clone)]
pub struct EmailService {
    client: Client,
    api_key: String,
    base_url: String, // e.g. "https://kepce.org"
    smtp_config: Option<SmtpConfig>,
}

impl EmailService {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
            smtp_config: None,
        }
    }

    pub fn from_config(config: &crate::config::Config) -> Self {
        let smtp_config = if let (Some(host), Some(user), Some(pass)) = (
            &config.smtp_host,
            &config.smtp_username,
            &config.smtp_password,
        ) {
            Some(SmtpConfig {
                host: host.clone(),
                port: config.smtp_port,
                username: user.clone(),
                password: pass.clone(),
            })
        } else {
            None
        };

        Self {
            client: Client::new(),
            api_key: config.resend_api_key.clone(),
            base_url: config.base_url.clone(),
            smtp_config,
        }
    }

    pub fn with_smtp(mut self, smtp_config: Option<SmtpConfig>) -> Self {
        self.smtp_config = smtp_config;
        self
    }

    async fn send_via_resend(&self, to: &str, subject: &str, html: &str) -> Result<(), EmailError> {
        let req = ResendRequest {
            from: "Kepçe <noreply@kepce.org>".to_string(),
            to: vec![to.to_string()],
            subject: subject.to_string(),
            html: html.to_string(),
        };

        let res = self
            .client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await
            .map_err(EmailError::ReqwestError)?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            tracing::warn!("Resend API Hatası: {} - {}", status, body);
            return Err(EmailError::ApiError(format!("Resend API Hatası {}: {}", status, body)));
        }

        Ok(())
    }

    async fn send_via_smtp(&self, smtp: &SmtpConfig, to: &str, subject: &str, html: String) -> Result<(), EmailError> {
        use lettre::message::header::ContentType;
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

        let email = Message::builder()
            .from("Kepçe <noreply@kepce.org>".parse().map_err(|e| EmailError::ApiError(format!("Geçersiz From adresi: {}", e)))?)
            .to(to.parse().map_err(|e| EmailError::ApiError(format!("Geçersiz To adresi: {}", e)))?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html)
            .map_err(|e| EmailError::ApiError(format!("Mesaj oluşturma hatası: {}", e)))?;

        let creds = Credentials::new(smtp.username.clone(), smtp.password.clone());

        let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.host)
            .map_err(|e| EmailError::ApiError(format!("SMTP relay başlatma hatası: {}", e)))?
            .port(smtp.port)
            .credentials(creds)
            .build();

        transport.send(email).await.map_err(|e| {
            tracing::error!("Amazon SES SMTP Gönderim Hatası: {:?}", e);
            EmailError::ApiError(format!("SES SMTP hatası: {}", e))
        })?;

        tracing::info!("E-posta SES SMTP üzerinden başarıyla gönderildi: {}", to);
        Ok(())
    }

    async fn send_email(&self, to: &str, subject: &str, html: String) -> Result<(), EmailError> {
        // Eğer API anahtarı boşsa veya mock_key ise ve SMTP yapılandırılmamışsa atla (test/yerel ortam)
        if (self.api_key.is_empty() || self.api_key == "mock_key") && self.smtp_config.is_none() {
            tracing::info!("Mock Email sent to {}: Subject: {}", to, subject);
            return Ok(());
        }

        // 1. Öncelikli olarak Resend üzerinden göndermeyi dene
        if !self.api_key.is_empty() && self.api_key != "mock_key" {
            match self.send_via_resend(to, subject, &html).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    tracing::warn!(
                        "Resend gönderimi başarısız ({:?}). SES SMTP fallback deneniyor...",
                        e
                    );
                    if self.smtp_config.is_none() {
                        return Err(e);
                    }
                }
            }
        }

        // 2. Resend başarısız olduysa veya kota bittiyse: Amazon SES SMTP fallback
        if let Some(smtp) = &self.smtp_config {
            tracing::info!("Amazon SES SMTP fallback devreye girdi: {}", to);
            return self.send_via_smtp(smtp, to, subject, html).await;
        }

        Ok(())
    }

    pub async fn send_passwordless_login(&self, to_email: &str, token: &str) -> Result<(), EmailError> {
        let magic_link = format!("{}/auth/sifresiz?token={}", self.base_url, token);
        
        let html = format!(
            r#"
            <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
                <h2 style="color: #ff8717;">Kepçe'ye Hoş Geldiniz</h2>
                <p>Şifresiz giriş yapmak için aşağıdaki butona tıklayın:</p>
                <a href="{}" style="display: inline-block; padding: 12px 24px; background-color: #ff8717; color: white; text-decoration: none; border-radius: 6px; font-weight: bold; margin: 20px 0;">Sisteme Giriş Yap</a>
                <p style="color: #666; font-size: 14px;">Bu bağlantı 15 dakika boyunca geçerlidir. Eğer bu talebi siz yapmadıysanız bu e-postayı görmezden gelebilirsiniz.</p>
            </div>
            "#,
            magic_link
        );

        self.send_email(to_email, "Kepçe Şifresiz Giriş Bağlantınız", html).await
    }

    pub async fn send_verification_email(&self, to_email: &str, token: &str) -> Result<(), EmailError> {
        let verify_link = format!("{}/auth/dogrula?token={}", self.base_url, token);
        
        let html = format!(
            r#"
            <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
                <h2 style="color: #ff8717;">Kepçe'ye Hoş Geldiniz!</h2>
                <p>Hesabınızı doğrulamak için aşağıdaki butona tıklayın:</p>
                <a href="{}" style="display: inline-block; padding: 12px 24px; background-color: #ff8717; color: white; text-decoration: none; border-radius: 6px; font-weight: bold; margin: 20px 0;">E-postamı Doğrula</a>
                <p style="color: #666; font-size: 14px;">Bu bağlantı 24 saat boyunca geçerlidir.</p>
            </div>
            "#,
            verify_link
        );

        self.send_email(to_email, "Kepçe Hesabınızı Doğrulayın", html).await
    }

    pub async fn send_reset_password_email(&self, to_email: &str, token: &str) -> Result<(), EmailError> {
        let reset_link = format!("{}/auth/sifre-sifirla?token={}", self.base_url, token);
        
        let html = format!(
            r#"
            <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
                <h2 style="color: #ff8717;">Şifre Sıfırlama Talebi</h2>
                <p>Şifrenizi sıfırlamak için aşağıdaki butona tıklayın:</p>
                <a href="{}" style="display: inline-block; padding: 12px 24px; background-color: #ff8717; color: white; text-decoration: none; border-radius: 6px; font-weight: bold; margin: 20px 0;">Şifremi Sıfırla</a>
                <p style="color: #666; font-size: 14px;">Bu bağlantı 1 saat boyunca geçerlidir. Eğer bu talebi siz yapmadıysanız güvenliğiniz için şifrenizi değiştirmeyi düşünebilirsiniz.</p>
            </div>
            "#,
            reset_link
        );

        self.send_email(to_email, "Şifre Sıfırlama Talebi", html).await
    }

    pub async fn send_security_alert(
        &self,
        to_email: &str,
        username: &str,
        event_title: &str,
        details: &str,
    ) -> Result<(), EmailError> {
        let timestamp = chrono::Utc::now().format("%d.%m.%Y %H:%M (UTC)").to_string();
        let html = format!(
            r#"
            <div style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 540px; margin: 0 auto; padding: 24px; background-color: #ffffff; border: 1px solid #e2e8f0; border-radius: 12px;">
                <div style="margin-bottom: 20px;">
                    <h2 style="color: #ff8717; margin: 0 0 6px 0; font-size: 20px;">Kepçe Güvenlik Bildirimi</h2>
                    <p style="color: #475569; font-size: 15px; margin: 0;">Merhaba <strong>@{}</strong>,</p>
                </div>
                <div style="background-color: #fff7ed; border-left: 4px solid #ff8717; padding: 14px 16px; margin-bottom: 20px; border-radius: 6px;">
                    <h3 style="margin: 0 0 6px 0; color: #c2410c; font-size: 15px;">{}</h3>
                    <p style="margin: 0; color: #7c2d12; font-size: 14px; line-height: 1.4;">{}</p>
                    <p style="margin: 8px 0 0 0; color: #9a3412; font-size: 12px;">Zaman: {}</p>
                </div>
                <p style="color: #64748b; font-size: 13px; line-height: 1.5; margin: 0 0 16px 0;">
                    Bu işlemi siz gerçekleştirdiyseniz ek bir işlem yapmanıza gerek yoktur. Eğer bu işlem bilginiz dışında gerçekleştiyse lütfen derhal şifrenizi sıfırlayın.
                </p>
                <div style="border-top: 1px solid #f1f5f9; padding-top: 14px; text-align: center;">
                    <a href="{}/ayarlar" style="color: #ff8717; text-decoration: none; font-size: 13px; font-weight: 600;">Hesap Güvenlik Ayarlarına Git &rarr;</a>
                </div>
            </div>
            "#,
            username,
            event_title,
            details,
            timestamp,
            self.base_url
        );

        self.send_email(to_email, &format!("Kepçe Güvenlik Uyarısı: {}", event_title), html).await
    }
}
