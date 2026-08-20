// Kepçe API - Dinamik OG Image Motoru (resvg + tiny-skia)
// =======================================================

use std::sync::{Arc, OnceLock};
use resvg::usvg::{self, fontdb, Tree};
use resvg::tiny_skia::{Pixmap, Transform};

static FONT_DB: OnceLock<Arc<fontdb::Database>> = OnceLock::new();

fn get_font_db() -> Arc<fontdb::Database> {
    FONT_DB.get_or_init(|| {
        let mut db = fontdb::Database::new();
        // Gömülü yazı tiplerini yükle
        db.load_font_data(include_bytes!("../../assets/fonts/PaytoneOne-Regular.ttf").to_vec());
        db.load_font_data(include_bytes!("../../assets/fonts/PaytoneOne-LatinExt.ttf").to_vec());
        db.load_font_data(include_bytes!("../../assets/fonts/PublicSans-Regular.ttf").to_vec());
        db.load_font_data(include_bytes!("../../assets/fonts/PublicSans-LatinExt.ttf").to_vec());
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

  <!-- Kepçe Logosu (174x77) -->
  <g transform="translate(150, 150)">
    <!-- Kepçe Yazısı -->
    <text x="0" y="55" font-family="Paytone One" font-size="54" font-weight="400" fill="#F9F5E5">Kepç</text>
    <!-- Kepçe İkonu Çizimi -->
    <path d="M161.761 0.771973C168.52 0.771973 174 6.25191 174 13.0109V22.25C174 38.3845 160.849 51.528 144.717 51.6215C144.597 51.6222 144.477 51.6226 144.356 51.6226C137.95 51.6226 132.33 48.9702 128.381 44.7088L128.324 45.6983C128.324 62.9856 114.309 77 97.022 77C79.7346 77 65.72 62.9856 65.72 45.6983C65.72 28.4109 79.7346 14.3965 97.022 14.3965C108.681 14.3965 118.895 20.7836 124.27 30.2741C126.969 22.9559 133.649 17.5855 141.674 16.9248L141.761 16.9179V13.0109C141.761 6.25191 147.241 0.771973 154 0.771973H161.761Z" fill="#F9F5E5" transform="translate(2, 0) scale(0.9)"/>
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

  <g transform="translate(150, 150)">
    <text x="0" y="55" font-family="Paytone One" font-size="54" font-weight="400" fill="#F9F5E5">Kepç</text>
    <path d="M161.761 0.771973C168.52 0.771973 174 6.25191 174 13.0109V22.25C174 38.3845 160.849 51.528 144.717 51.6215C144.597 51.6222 144.477 51.6226 144.356 51.6226C137.95 51.6226 132.33 48.9702 128.381 44.7088L128.324 45.6983C128.324 62.9856 114.309 77 97.022 77C79.7346 77 65.72 62.9856 65.72 45.6983C65.72 28.4109 79.7346 14.3965 97.022 14.3965C108.681 14.3965 118.895 20.7836 124.27 30.2741C126.969 22.9559 133.649 17.5855 141.674 16.9248L141.761 16.9179V13.0109C141.761 6.25191 147.241 0.771973 154 0.771973H161.761Z" fill="#F9F5E5" transform="translate(2, 0) scale(0.9)"/>
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
            "12 Şubat 2026",
            Some("8 Çeşit Yemek · 1400-1950 kkal"),
            Some("Günlük Menü"),
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
