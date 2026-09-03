// Kepçe API - Dinamik OG Image Motoru (resvg + tiny-skia)
// =======================================================

use std::sync::{Arc, OnceLock};
use resvg::usvg::{self, fontdb, Tree};
use resvg::tiny_skia::{Pixmap, Transform};

static FONT_DB: OnceLock<Arc<fontdb::Database>> = OnceLock::new();

fn get_font_db() -> Arc<fontdb::Database> {
    FONT_DB.get_or_init(|| {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        // Gömülü yazı tiplerini yükle
        db.load_font_data(include_bytes!("../../assets/fonts/PaytoneOne-Regular.ttf").to_vec());
        db.load_font_data(include_bytes!("../../assets/fonts/PaytoneOne-LatinExt.ttf").to_vec());
        db.load_font_data(include_bytes!("../../assets/fonts/PublicSans-Regular.ttf").to_vec());
        db.load_font_data(include_bytes!("../../assets/fonts/PublicSans-SemiBold.ttf").to_vec());
        db.set_sans_serif_family("Public Sans");
        Arc::new(db)
    }).clone()
}

// XML Kaçış Yardımcısı
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

/// Standart Kart veya Dinamik Sayfa Kartı için SVG Oluşturucu
pub fn render_og_card(
    title: Option<&str>,
    sub1: &str,
    sub2: Option<&str>,
    badge: Option<&str>,
) -> Result<Vec<u8>, anyhow::Error> {
    let font_db = get_font_db();

    let safe_title = title.map(escape_xml);
    let safe_sub1 = escape_xml(sub1);
    let safe_sub2 = sub2.map(escape_xml);
    let safe_badge = badge.map(escape_xml);

    // Rozet genişliği dinamik hesaplama (yaklaşık 1 karakter = 17px + 64px padding)
    let badge_svg = if let Some(badge_text) = safe_badge {
        if badge_text != "null" && !badge_text.is_empty() {
            let char_count = badge_text.chars().count();
            let badge_width = (char_count as f32 * 17.0 + 64.0).max(120.0);
            let badge_x = 1050.0 - badge_width;
            format!(
                r##"<g transform="translate({badge_x}, 150)">
                    <rect width="{badge_width}" height="77" rx="38.5" fill="#ECBF7F"/>
                    <text x="{badge_center}" y="48" font-family="Public Sans" font-size="28" font-weight="800" fill="#242828" text-anchor="middle" dominant-baseline="central">{badge_text}</text>
                </g>"##,
                badge_x = badge_x,
                badge_width = badge_width,
                badge_center = badge_width / 2.0,
                badge_text = badge_text
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Başlık ve Alt Metin Konumlandırmaları
    let content_svg = if let Some(sub2_text) = safe_sub2 {
        // ÇİFT SATIRLI DİNAMİK KART (/:city_slug, /menu/:id, /menu/:id/:threadId)
        let t_text = safe_title.unwrap_or_default();
        format!(
            r##"<text x="150" y="320" font-family="Paytone One" font-size="56" font-weight="400" fill="#F9F5E5">{t_text}</text>
               <text x="150" y="385" font-family="Public Sans" font-size="40" font-weight="600" fill="#F9F5E5">{safe_sub1}</text>
               <text x="150" y="445" font-family="Public Sans" font-size="40" font-weight="600" fill="#F9F5E5">{sub2_text}</text>"##,
            t_text = t_text,
            safe_sub1 = safe_sub1,
            sub2_text = sub2_text
        )
    } else {
        // TEK METİNLİ KART (Hakkında, SSS, Menü Gönder, vb. - 2 satıra sarma destekli)
        if let Some(t_text) = safe_title {
            let words: Vec<&str> = sub1.split_whitespace().collect();
            let is_long = sub1.chars().count() > 40;
            
            if is_long && words.len() > 3 {
                let mid = words.len() / 2;
                let line1 = escape_xml(&words[..mid].join(" "));
                let line2 = escape_xml(&words[mid..].join(" "));
                format!(
                    r##"<text x="150" y="325" font-family="Paytone One" font-size="62" font-weight="400" fill="#F9F5E5">{t_text}</text>
                       <text x="150" y="395" font-family="Public Sans" font-size="38" font-weight="600" fill="#F9F5E5">{line1}</text>
                       <text x="150" y="450" font-family="Public Sans" font-size="38" font-weight="600" fill="#F9F5E5">{line2}</text>"##,
                    t_text = t_text,
                    line1 = line1,
                    line2 = line2
                )
            } else {
                format!(
                    r##"<text x="150" y="335" font-family="Paytone One" font-size="64" font-weight="400" fill="#F9F5E5">{t_text}</text>
                       <text x="150" y="415" font-family="Public Sans" font-size="42" font-weight="600" fill="#F9F5E5">{safe_sub1}</text>"##,
                    t_text = t_text,
                    safe_sub1 = safe_sub1
                )
            }
        } else {
            // Sadece alt metin (Örn Master Kart)
            format!(
                r##"<text x="150" y="360" font-family="Public Sans" font-size="44" font-weight="600" fill="#F9F5E5">{safe_sub1}</text>"##,
                safe_sub1 = safe_sub1
            )
        }
    };

    let svg = format!(
        r##"<svg width="1200" height="630" viewBox="0 0 1200 630" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
  <defs>
    <radialGradient id="glow" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#ECBF7F" stop-opacity="0.25"/>
      <stop offset="100%" stop-color="#ECBF7F" stop-opacity="0"/>
    </radialGradient>
  </defs>

  <!-- Arka Plan -->
  <rect width="1200" height="630" fill="#242828"/>
  <rect width="1200" height="630" fill="url(#glow)"/>

  <!-- Kepçe Logosu (Vektör) -->
  <g transform="translate(150, 150) scale(1.15)" fill="#F9F5E5">
    <path d="M0 54V21h11.1v13.1h3.4L21 21h12L24.2 37.4 34.2 54H21.5L14.4 41.3H11.1V54H0z"/>
    <path d="M49.3 47.7c.9 0 1.9-.1 2.9-.3 1.1-.2 2.1-.5 3.2-1l1.2 6.1c-1.2.6-2.5 1.1-3.9 1.4-1.4.3-3 .5-4.8.5-3.2 0-6-.5-8.2-1.6-2.2-1-3.8-2.5-5-4.4-1.1-1.9-1.7-4-1.7-6.4 0-2.5.6-4.7 1.8-6.5 1.2-1.9 2.8-3.3 4.8-4.3 2-1 4.3-1.6 6.7-1.6 3.6 0 6.4 1.1 8.3 3.4 1.9 2.2 2.9 5.4 2.9 9.6l-14.3 1.8c1 2.2 3 3.3 6 3.3zm-6.7-7.6l7.5-1.6c-.1-1-.5-1.8-1-2.3-.6-.5-1.3-.8-2.2-.8-1.3 0-2.3.4-3.1 1.3-.7.9-1.1 2-1.2 3.4z"/>
    <path d="M71.1 61.6l-10.1 1.5V30h10v5.2c.8-2 2-3.4 3.4-4.3 1.4-.9 3-1.3 4.5-1.3 2.2 0 4.2.5 6 1.6 1.8 1 3.2 2.4 4.2 4.3 1.1 1.8 1.6 4 1.6 6.4 0 2.4-.5 4.6-1.6 6.5-1 1.9-2.4 3.4-4.2 4.5-1.8 1.1-3.8 1.6-6 1.6-1.9 0-3.5-.4-4.8-1.2-1.3-.8-2.3-2-3-3.6v12zm4.7-14.8c1.4 0 2.5-.5 3.5-1.4.9-.9 1.4-2 1.4-3.4s-.5-2.5-1.4-3.4c-.9-.9-2-1.4-3.5-1.4-1.3 0-2.4.5-3.3 1.3-.9.9-1.4 2-1.4 3.4s.5 2.5 1.4 3.5c1 .9 2.1 1.4 3.3 1.4z"/>
    <path d="M106.2 54.4c-2.7 0-5-.5-7-1.6-1.9-1-3.4-2.5-4.5-4.3-1-1.9-1.6-4-1.6-6.4 0-2.4.6-4.5 1.7-6.4 1.1-1.9 2.6-3.4 4.5-4.5 2-1.1 4.3-1.6 6.9-1.6 3.1 0 5.5.6 7.3 1.9l-1.1 6.5c-.8-.3-1.6-.6-2.3-.7-.7-.2-1.4-.2-2-.2-1.6 0-2.8.4-3.7 1.3-.9.9-1.3 2-1.3 3.5 0 1.5.4 2.7 1.3 3.6.9.9 2.1 1.3 3.6 1.3 1.3 0 2.8-.3 4.5-.8l.9 6.7c-1 .5-2.1.9-3.3 1.2-1.1.1-2.4.3-3.9.3zm-2.6 11.5c-.7 0-1.5-.1-2.3-.2-.8-.1-1.5-.3-2.1-.6v-2.7c.4.1.9.2 1.2.3.4.1.8.1 1.1.1.8 0 1.5-.2 2-.6.5-.4.8-1 .8-1.7 0-.6-.3-1.1-.8-1.5-.5-.4-1.2-.6-2-.7l1.9-5.6h3.6l-1.2 3.6c1.6.4 2.7 1 3.3 1.7.7.8 1 1.7 1 2.8 0 1.6-.6 2.8-1.8 3.7-1.2 1-2.8 1.4-4.8 1.4z"/>
    <path d="M143 39.7c.7 7.7-4.6 14-12.2 14.8-7.6.8-14.7-3.8-15.4-11.5l20-4.7L133 8.4c-.4-3.8 3.4-8.7 9-8.1 4.2.4 7 4.5 7.3 8.3l-4.6.5c-.1-1.3-1.2-2.2-2.5-2.1s-2.2 1.3-2.1 2.5l2.9 30.2z"/>
  </g>

  <!-- Rozet -->
  {badge_svg}

  <!-- İçerik Metinleri -->
  {content_svg}
</svg>"##,
        badge_svg = badge_svg,
        content_svg = content_svg
    );

    rasterize_svg(&svg, font_db)
}

/// Kullanıcı Profili için SVG Oluşturucu (Avatar Resimli veya Avatarsız)
pub fn render_og_profile(
    username: &str,
    karma_text: &str,
    avatar_base64: Option<&str>,
    badge: &str,
) -> Result<Vec<u8>, anyhow::Error> {
    let font_db = get_font_db();

    let safe_user = escape_xml(username);
    let safe_karma = escape_xml(karma_text);
    let safe_badge = escape_xml(badge);

    let badge_width = (safe_badge.chars().count() as f32 * 17.0 + 64.0).max(120.0);
    let badge_x = 1050.0 - badge_width;

    let body_svg = if let Some(b64) = avatar_base64 {
        format!(
            r##"<g transform="translate(150, 275)">
                <defs>
                  <clipPath id="avatar-clip">
                    <circle cx="80" cy="80" r="80"/>
                  </clipPath>
                  <filter id="avatar-shadow" x="-20%" y="-20%" width="140%" height="140%">
                    <feDropShadow dx="0" dy="12" stdDeviation="18" flood-color="#000000" flood-opacity="0.5"/>
                  </filter>
                </defs>
                <circle cx="80" cy="80" r="80" fill="#303535" filter="url(#avatar-shadow)"/>
                <image href="data:image/jpeg;base64,{b64}" width="160" height="160" clip-path="url(#avatar-clip)" preserveAspectRatio="xMidYMid slice"/>
                
                <text x="205" y="65" font-family="Paytone One" font-size="56" font-weight="400" fill="#F9F5E5">{safe_user}</text>
                <text x="205" y="125" font-family="Public Sans" font-size="38" font-weight="600" fill="#F9F5E5">{safe_karma}</text>
              </g>"##,
            b64 = b64,
            safe_user = safe_user,
            safe_karma = safe_karma
        )
    } else {
        format!(
            r##"<text x="150" y="335" font-family="Paytone One" font-size="64" font-weight="400" fill="#F9F5E5">{safe_user}</text>
               <text x="150" y="415" font-family="Public Sans" font-size="42" font-weight="600" fill="#F9F5E5">{safe_karma}</text>"##,
            safe_user = safe_user,
            safe_karma = safe_karma
        )
    };

    let svg = format!(
        r##"<svg width="1200" height="630" viewBox="0 0 1200 630" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
  <defs>
    <radialGradient id="glow" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#ECBF7F" stop-opacity="0.25"/>
      <stop offset="100%" stop-color="#ECBF7F" stop-opacity="0"/>
    </radialGradient>
  </defs>

  <rect width="1200" height="630" fill="#242828"/>
  <rect width="1200" height="630" fill="url(#glow)"/>

  <!-- Kepçe Logosu (Vektör) -->
  <g transform="translate(150, 150) scale(1.15)" fill="#F9F5E5">
    <path d="M0 54V21h11.1v13.1h3.4L21 21h12L24.2 37.4 34.2 54H21.5L14.4 41.3H11.1V54H0z"/>
    <path d="M49.3 47.7c.9 0 1.9-.1 2.9-.3 1.1-.2 2.1-.5 3.2-1l1.2 6.1c-1.2.6-2.5 1.1-3.9 1.4-1.4.3-3 .5-4.8.5-3.2 0-6-.5-8.2-1.6-2.2-1-3.8-2.5-5-4.4-1.1-1.9-1.7-4-1.7-6.4 0-2.5.6-4.7 1.8-6.5 1.2-1.9 2.8-3.3 4.8-4.3 2-1 4.3-1.6 6.7-1.6 3.6 0 6.4 1.1 8.3 3.4 1.9 2.2 2.9 5.4 2.9 9.6l-14.3 1.8c1 2.2 3 3.3 6 3.3zm-6.7-7.6l7.5-1.6c-.1-1-.5-1.8-1-2.3-.6-.5-1.3-.8-2.2-.8-1.3 0-2.3.4-3.1 1.3-.7.9-1.1 2-1.2 3.4z"/>
    <path d="M71.1 61.6l-10.1 1.5V30h10v5.2c.8-2 2-3.4 3.4-4.3 1.4-.9 3-1.3 4.5-1.3 2.2 0 4.2.5 6 1.6 1.8 1 3.2 2.4 4.2 4.3 1.1 1.8 1.6 4 1.6 6.4 0 2.4-.5 4.6-1.6 6.5-1 1.9-2.4 3.4-4.2 4.5-1.8 1.1-3.8 1.6-6 1.6-1.9 0-3.5-.4-4.8-1.2-1.3-.8-2.3-2-3-3.6v12zm4.7-14.8c1.4 0 2.5-.5 3.5-1.4.9-.9 1.4-2 1.4-3.4s-.5-2.5-1.4-3.4c-.9-.9-2-1.4-3.5-1.4-1.3 0-2.4.5-3.3 1.3-.9.9-1.4 2-1.4 3.4s.5 2.5 1.4 3.5c1 .9 2.1 1.4 3.3 1.4z"/>
    <path d="M106.2 54.4c-2.7 0-5-.5-7-1.6-1.9-1-3.4-2.5-4.5-4.3-1-1.9-1.6-4-1.6-6.4 0-2.4.6-4.5 1.7-6.4 1.1-1.9 2.6-3.4 4.5-4.5 2-1.1 4.3-1.6 6.9-1.6 3.1 0 5.5.6 7.3 1.9l-1.1 6.5c-.8-.3-1.6-.6-2.3-.7-.7-.2-1.4-.2-2-.2-1.6 0-2.8.4-3.7 1.3-.9.9-1.3 2-1.3 3.5 0 1.5.4 2.7 1.3 3.6.9.9 2.1 1.3 3.6 1.3 1.3 0 2.8-.3 4.5-.8l.9 6.7c-1 .5-2.1.9-3.3 1.2-1.1.1-2.4.3-3.9.3zm-2.6 11.5c-.7 0-1.5-.1-2.3-.2-.8-.1-1.5-.3-2.1-.6v-2.7c.4.1.9.2 1.2.3.4.1.8.1 1.1.1.8 0 1.5-.2 2-.6.5-.4.8-1 .8-1.7 0-.6-.3-1.1-.8-1.5-.5-.4-1.2-.6-2-.7l1.9-5.6h3.6l-1.2 3.6c1.6.4 2.7 1 3.3 1.7.7.8 1 1.7 1 2.8 0 1.6-.6 2.8-1.8 3.7-1.2 1-2.8 1.4-4.8 1.4z"/>
    <path d="M143 39.7c.7 7.7-4.6 14-12.2 14.8-7.6.8-14.7-3.8-15.4-11.5l20-4.7L133 8.4c-.4-3.8 3.4-8.7 9-8.1 4.2.4 7 4.5 7.3 8.3l-4.6.5c-.1-1.3-1.2-2.2-2.5-2.1s-2.2 1.3-2.1 2.5l2.9 30.2z"/>
  </g>

  <g transform="translate({badge_x}, 150)">
    <rect width="{badge_width}" height="77" rx="38.5" fill="#ECBF7F"/>
    <text x="{badge_center}" y="48" font-family="Public Sans" font-size="28" font-weight="800" fill="#242828" text-anchor="middle" dominant-baseline="central">{safe_badge}</text>
  </g>

  {body_svg}
</svg>"##,
        badge_x = badge_x,
        badge_width = badge_width,
        badge_center = badge_width / 2.0,
        safe_badge = safe_badge,
        body_svg = body_svg
    );

    rasterize_svg(&svg, font_db)
}

fn rasterize_svg(svg: &str, font_db: Arc<fontdb::Database>) -> Result<Vec<u8>, anyhow::Error> {
    let opt = usvg::Options {
        fontdb: font_db,
        ..Default::default()
    };

    let tree = Tree::from_str(svg, &opt)
        .map_err(|e| anyhow::anyhow!("SVG parse hatası: {:?}", e))?;

    let width = 1200;
    let height = 630;
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| anyhow::anyhow!("Pixmap oluşturulamadı"))?;

    resvg::render(&tree, Transform::default(), &mut pixmap.as_mut());

    let png_data = pixmap.encode_png()
        .map_err(|e| anyhow::anyhow!("PNG encode hatası: {:?}", e))?;

    Ok(png_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_og_card_png_magic_bytes() {
        let png = render_og_card(
            Some("İstanbul"),
            "3 Eylül 2026 · Akşam Yemeği",
            Some("4 Çeşit Yemek · 1100-1500 kkal"),
            Some("Öğün"),
        ).expect("Render failed");

        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"), "Output must be valid PNG");
        assert!(png.len() > 10000, "PNG should have substantial size");
    }

    #[test]
    fn test_render_og_profile_png() {
        let png = render_og_profile(
            "@modparator",
            "1.420 Karma · Ağustos 2026",
            None,
            "Kullanıcı Profili",
        ).expect("Profile render failed");

        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"), "Output must be valid PNG");
    }
}
