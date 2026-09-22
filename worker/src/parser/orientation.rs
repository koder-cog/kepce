//! Modele gönderim öncesi **sayfa bazlı** yön (orientation) algılama ve düzeltme.
//!
//! Amaç: yan/ters taranmış sayfaları modele göndermeden önce kod tarafında 0°'ye
//! getirmek. Böylece model dik ve hizalı bir tablo görür; satır kayması ve eksik
//! okuma ortadan kalkar.
//!
//! Akış:
//! 1. Belge sayfalara bölünür (görsel = tek sayfa, PDF = pdfium ile rasterize).
//! 2. Yön algılama **her sayfaya ayrı ayrı** uygulanır (belgenin geneline değil).
//! 3. Sayfa gerekiyorsa döndürülür (görsel: piksel döndürme, PDF: `/Rotate`).
//! 4. Düzeltilmiş belge modele teslim edilir.
//!
//! Güvenlik ilkesi: **yanlış döndürme, doğru veriyi bozar.** Bu yüzden yalnızca
//! güven eşiğini aşan kararlar uygulanır; aksi halde sayfaya dokunulmaz ve
//! durum `WARN` ile loglanır.

use anyhow::{Context, Result};
use image::{DynamicImage, GrayImage, ImageFormat};
use std::io::Cursor;

/// Algılama için görüntünün en uzun kenarı bu değere küçültülür (hız/doğruluk dengesi).
const DETECT_MAX_DIM: u32 = 1000;

/// Satır/sütun profili varyans oranı eşikleri.
/// Oran büyükse metin satırları yatay (0/180), küçükse dikey (90/270).
const UPRIGHT_RATIO: f32 = 1.15;
const ROTATED_RATIO: f32 = 0.87;

/// Kararı uygulamak için gereken en düşük güven.
const MIN_CONFIDENCE: f32 = 0.55;

/// İnce bantları (tablo çizgileri) metin satırı saymamak için en az yükseklik.
const MIN_BAND_THICKNESS: usize = 3;

/// Saat yönünde uygulanacak düzeltme miktarı.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    None,
    Cw90,
    Cw180,
    Cw270,
}

impl Rotation {
    pub fn degrees(self) -> u16 {
        match self {
            Rotation::None => 0,
            Rotation::Cw90 => 90,
            Rotation::Cw180 => 180,
            Rotation::Cw270 => 270,
        }
    }
}

/// Tek sayfa için yön kararı.
#[derive(Debug, Clone, Copy)]
pub struct PageOrientation {
    pub rotation: Rotation,
    pub confidence: f32,
}

/// Düzeltilmiş belge.
pub struct CorrectedDocument {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub pages: usize,
    pub corrected_pages: usize,
}

/// Sayfa yönü düzeltmesi etkin mi? (`ORIENTATION_CORRECTION=0` ile kapatılabilir.)
pub fn is_enabled() -> bool {
    match std::env::var("ORIENTATION_CORRECTION") {
        Ok(v) => !matches!(
            v.trim().to_lowercase().as_str(),
            "0" | "false" | "off" | "no"
        ),
        Err(_) => true,
    }
}

/// Belgeyi (görsel veya PDF) sayfa bazlı yön düzeltmesinden geçirir.
///
/// Desteklenmeyen/başarısız durumlarda **orijinal baytlar** döndürülür; bu
/// fonksiyon asla ayrıştırmayı bloke etmez (en iyi çaba / best-effort).
pub fn correct_document(bytes: &[u8], mime_type: &str) -> CorrectedDocument {
    let fallback = |pages: usize| CorrectedDocument {
        bytes: bytes.to_vec(),
        mime_type: mime_type.to_string(),
        pages,
        corrected_pages: 0,
    };

    if !is_enabled() {
        return fallback(0);
    }

    if mime_type == "application/pdf" {
        match correct_pdf(bytes) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    "PDF yön düzeltmesi uygulanamadı (orijinal belge kullanılıyor): {e:#}"
                );
                fallback(0)
            }
        }
    } else {
        match correct_image(bytes, mime_type) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    "Görsel yön düzeltmesi uygulanamadı (orijinal belge kullanılıyor): {e:#}"
                );
                fallback(1)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Görseller
// ---------------------------------------------------------------------------

fn correct_image(bytes: &[u8], mime_type: &str) -> Result<CorrectedDocument> {
    use image::ImageDecoder;

    let mut decoder = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .context("Görsel formatı tahmin edilemedi")?
        .into_decoder()
        .context("Görsel çözücü açılamadı")?;

    // 1. Katman: EXIF yönü (telefon fotoğraflarının çoğu burada düzelir).
    let exif_orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder).context("Görsel çözülemedi")?;
    img.apply_orientation(exif_orientation);

    // 2./3. Katman: projeksiyon profili ile yön algılama.
    let gray = img.to_luma8();
    let decision = detect_orientation(&gray);
    let applied = if decision.confidence >= MIN_CONFIDENCE {
        decision.rotation
    } else {
        tracing::debug!(
            "sayfa 1/1: yön belirsiz (güven={:.2}), dokunulmadı",
            decision.confidence
        );
        Rotation::None
    };

    if applied == Rotation::None {
        return Ok(CorrectedDocument {
            bytes: bytes.to_vec(),
            mime_type: mime_type.to_string(),
            pages: 1,
            corrected_pages: 0,
        });
    }

    let rotated = match applied {
        Rotation::Cw90 => img.rotate90(),
        Rotation::Cw180 => img.rotate180(),
        Rotation::Cw270 => img.rotate270(),
        Rotation::None => img,
    };

    let out_mime = encode_image(&rotated, mime_type)?;
    tracing::info!(
        "sayfa 1/1: algılanan yön={}°, düzeltildi (güven={:.2})",
        applied.degrees(),
        decision.confidence
    );

    Ok(CorrectedDocument {
        bytes: out_mime.0,
        mime_type: out_mime.1.to_string(),
        pages: 1,
        corrected_pages: 1,
    })
}

/// Döndürülmüş görüntüyü yeniden kodlar (JPEG'de kalite korunur).
fn encode_image(img: &DynamicImage, original_mime: &str) -> Result<(Vec<u8>, &'static str)> {
    let (format, mime) = match original_mime {
        "image/png" => (ImageFormat::Png, "image/png"),
        "image/webp" => (ImageFormat::WebP, "image/webp"),
        // JPEG ve HEIC/HEIF (tarayıcı çıktıları) JPEG'e normalize edilir.
        _ => (ImageFormat::Jpeg, "image/jpeg"),
    };

    let mut out = Vec::new();
    let rgb = img.to_rgb8();
    rgb.write_to(&mut Cursor::new(&mut out), format)
        .context("Döndürülmüş görsel kodlanamadı")?;
    Ok((out, mime))
}

// ---------------------------------------------------------------------------
// PDF
// ---------------------------------------------------------------------------

fn correct_pdf(bytes: &[u8]) -> Result<CorrectedDocument> {
    let rotations = detect_pdf_page_rotations(bytes)?;
    if rotations.is_empty() {
        anyhow::bail!("PDF sayfası bulunamadı");
    }

    let corrected_pages = rotations
        .iter()
        .filter(|r| r.rotation != Rotation::None)
        .count();
    if corrected_pages == 0 {
        return Ok(CorrectedDocument {
            bytes: bytes.to_vec(),
            mime_type: "application/pdf".to_string(),
            pages: rotations.len(),
            corrected_pages: 0,
        });
    }

    let out = apply_pdf_rotations(bytes, &rotations)?;
    Ok(CorrectedDocument {
        bytes: out,
        mime_type: "application/pdf".to_string(),
        pages: rotations.len(),
        corrected_pages,
    })
}

/// Her PDF sayfasını rasterize edip **ayrı ayrı** yön kararı üretir.
fn detect_pdf_page_rotations(bytes: &[u8]) -> Result<Vec<PageOrientation>> {
    use pdfium_render::prelude::*;

    let bindings = Pdfium::bind_to_system_library()
        .context("libpdfium.so yüklenemedi (worker imajında mevcut mu?)")?;
    let pdfium = Pdfium::new(bindings);

    let doc = pdfium
        .load_pdf_from_byte_slice(bytes, None)
        .context("PDF pdfium ile açılamadı")?;

    let mut out = Vec::new();
    for (idx, page) in doc.pages().iter().enumerate() {
        // pdfium, mevcut /Rotate değerini render'a uygular; yani render edilen
        // görüntü "ekranda görünen" yöndedir. Dolayısıyla algılanan açı, mevcut
        // /Rotate'e EKLENECEK ek düzeltmedir (ek telafi gerekmez).
        let cfg = PdfRenderConfig::new()
            .set_target_width(DETECT_MAX_DIM as i32)
            .render_form_data(false);
        let bitmap = page
            .render_with_config(&cfg)
            .with_context(|| format!("PDF sayfası render edilemedi: {}", idx + 1))?;

        let img = bitmap
            .as_image()
            .with_context(|| format!("PDF sayfası görüntüye çevrilemedi: {}", idx + 1))?;
        let gray = img.to_luma8();
        out.push(detect_orientation(&gray));
    }
    Ok(out)
}

/// lopdf ile sayfa `/Rotate` değerlerini ayarlar (belge tek parça kalır).
fn apply_pdf_rotations(bytes: &[u8], rotations: &[PageOrientation]) -> Result<Vec<u8>> {
    let mut doc = lopdf::Document::load_mem(bytes).context("PDF lopdf ile açılamadı")?;
    let pages = doc.get_pages();

    let mut applied = 0usize;
    for (page_id, decision) in pages.values().zip(rotations.iter()) {
        if decision.rotation == Rotation::None {
            continue;
        }
        let Ok(dict) = doc.get_object_mut(*page_id).and_then(|o| o.as_dict_mut()) else {
            continue;
        };
        let current = dict
            .get(b"Rotate")
            .ok()
            .and_then(|o| o.as_i64().ok())
            .unwrap_or(0);
        let next = (current + decision.rotation.degrees() as i64).rem_euclid(360);
        dict.set("Rotate", next);
        applied += 1;
        tracing::info!(
            "sayfa {}/{}: algılanan yön={}°, /Rotate={}° olarak ayarlandı (güven={:.2})",
            applied,
            pages.len(),
            decision.rotation.degrees(),
            next,
            decision.confidence
        );
    }

    let mut out = Vec::new();
    doc.save_to(&mut out)
        .context("Düzeltilmiş PDF yazılamadı")?;
    Ok(out)
}

// ---------------------------------------------------------------------------
// Algılama çekirdeği
// ---------------------------------------------------------------------------

/// Projeksiyon profili + bant içi mürekkep asimetrisi ile yön kararı.
///
/// - **Eksen:** satır/sütun profili varyans oranı. Metin satırları yatayken satır
///   profili güçlü periyodiklik gösterir; dikeyken sütun profili gösterir.
/// - **Yön:** her metin bandı içinde mürekkep ağırlık merkezinin banda göre
///   konumu. Dik metinde taban çizgisi/alt uzantılar (ş, ç, ğ, y, p) mürekkebi
///   bandın altına çeker; 180° dönmüş metinde üste çeker.
pub fn detect_orientation(img: &GrayImage) -> PageOrientation {
    let small = downscale(img);
    let (w, h) = (small.width() as usize, small.height() as usize);
    if w < 8 || h < 8 {
        return PageOrientation {
            rotation: Rotation::None,
            confidence: 0.0,
        };
    }

    let ink = otsu_ink_mask(&small);
    let row_profile: Vec<f32> = (0..h)
        .map(|y| (0..w).filter(|&x| ink[y * w + x]).count() as f32)
        .collect();
    let col_profile: Vec<f32> = (0..w)
        .map(|x| (0..h).filter(|&y| ink[y * w + x]).count() as f32)
        .collect();

    let var_row = variance(&row_profile);
    let var_col = variance(&col_profile);
    let ratio = if var_col > f32::EPSILON {
        var_row / var_col
    } else {
        1.0
    };

    if ratio >= UPRIGHT_RATIO {
        // Metin satırları yatay → 0 veya 180.
        let centroid = band_centroid_ratio(&ink, w, h, true);
        let rotation = if centroid >= 0.5 {
            Rotation::None
        } else {
            Rotation::Cw180
        };
        PageOrientation {
            rotation,
            confidence: axis_confidence(ratio, true) * direction_confidence(centroid),
        }
    } else if ratio <= ROTATED_RATIO {
        // Metin satırları dikey → 90 veya 270.
        let centroid = band_centroid_ratio(&ink, w, h, false);
        let rotation = if centroid < 0.5 {
            Rotation::Cw90
        } else {
            Rotation::Cw270
        };
        PageOrientation {
            rotation,
            confidence: axis_confidence(ratio, false) * direction_confidence(centroid),
        }
    } else {
        // Eksen belirsiz: dokunma.
        PageOrientation {
            rotation: Rotation::None,
            confidence: 0.0,
        }
    }
}

fn downscale(img: &GrayImage) -> GrayImage {
    let (w, h) = (img.width(), img.height());
    let longest = w.max(h);
    if longest <= DETECT_MAX_DIM || longest == 0 {
        return img.clone();
    }
    let scale = DETECT_MAX_DIM as f32 / longest as f32;
    DynamicImage::ImageLuma8(img.clone())
        .resize_exact(
            ((w as f32 * scale).round() as u32).max(1),
            ((h as f32 * scale).round() as u32).max(1),
            image::imageops::FilterType::Triangle,
        )
        .to_luma8()
}

/// Otsu eşikleme ile mürekkep maskesi (metin genelde koyu).
fn otsu_ink_mask(img: &GrayImage) -> Vec<bool> {
    let mut hist = [0u32; 256];
    for p in img.pixels() {
        hist[p.0[0] as usize] += 1;
    }
    let total: u32 = img.width() * img.height();
    let sum_all: f64 = (0..256).map(|i| i as f64 * hist[i] as f64).sum();

    let (mut sum_b, mut w_b) = (0.0f64, 0u32);
    let (mut best, mut threshold) = (0.0f64, 127u8);
    for (i, &bin) in hist.iter().enumerate() {
        w_b += bin;
        if w_b == 0 {
            continue;
        }
        let w_f = total.saturating_sub(w_b);
        if w_f == 0 {
            break;
        }
        sum_b += i as f64 * bin as f64;
        let m_b = sum_b / w_b as f64;
        let m_f = (sum_all - sum_b) / w_f as f64;
        let between = w_b as f64 * w_f as f64 * (m_b - m_f) * (m_b - m_f);
        if between > best {
            best = between;
            threshold = i as u8;
        }
    }

    img.pixels()
        .map(|p| p.0[0] < threshold.max(1))
        .collect::<Vec<bool>>()
}

/// Bant içi mürekkep ağırlık merkezinin banda göre normalize konumu (0..1).
///
/// `horizontal` true ise satır bantları (yatay metin), false ise sütun bantları
/// (dikey metin) üzerinde çalışır. Bant bulunamazsa 0.5 (belirsiz) döner.
fn band_centroid_ratio(ink: &[bool], w: usize, h: usize, horizontal: bool) -> f32 {
    let (outer, inner) = if horizontal { (h, w) } else { (w, h) };

    let has_ink = |o: usize| -> bool {
        (0..inner).any(|i| {
            let (x, y) = if horizontal { (i, o) } else { (o, i) };
            ink[y * w + x]
        })
    };
    let count = |o: usize| -> f32 {
        (0..inner)
            .filter(|&i| {
                let (x, y) = if horizontal { (i, o) } else { (o, i) };
                ink[y * w + x]
            })
            .count() as f32
    };

    let mut ratios: Vec<f32> = Vec::new();
    let mut o = 0usize;
    while o < outer {
        if !has_ink(o) {
            o += 1;
            continue;
        }
        let start = o;
        while o < outer && has_ink(o) {
            o += 1;
        }
        let end = o;
        if end - start < MIN_BAND_THICKNESS {
            continue;
        }
        let (mut sum, mut cnt) = (0.0f32, 0.0f32);
        for k in start..end {
            let c = count(k);
            sum += c * (k as f32 + 0.5);
            cnt += c;
        }
        if cnt > 0.0 {
            ratios.push((sum / cnt - start as f32) / (end - start) as f32);
        }
    }

    if ratios.is_empty() {
        0.5
    } else {
        ratios.iter().sum::<f32>() / ratios.len() as f32
    }
}

fn variance(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32
}

/// Eksen kararının güveni: oran eşikten ne kadar uzak?
fn axis_confidence(ratio: f32, horizontal: bool) -> f32 {
    let (v, t) = if horizontal {
        (ratio, UPRIGHT_RATIO)
    } else {
        (1.0 / ratio.max(f32::EPSILON), 1.0 / ROTATED_RATIO)
    };
    (((v - 1.0) / (t - 1.0)).clamp(0.0, 2.0) / 2.0).clamp(0.0, 1.0)
}

/// Yön kararının güveni: ağırlık merkezi 0.5'ten ne kadar uzak?
fn direction_confidence(centroid: f32) -> f32 {
    ((centroid - 0.5).abs() * 4.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GrayImage, Luma};

    /// Yatay "metin" satırları içeren sentetik sayfa; alt uzantı (descender)
    /// etkisini taklit etmek için mürekkep bandın altına kaydırılır.
    fn synthetic_page(w: u32, h: u32) -> GrayImage {
        let mut img = GrayImage::from_pixel(w, h, Luma([255u8]));
        let band_h = h / 10;
        for band in 0..5u32 {
            let top = band * 2 * band_h + band_h / 2;
            // Gövde: bandın üst-orta kısmı
            for y in top..top + band_h {
                for x in 20..w - 20 {
                    img.put_pixel(x, y, Luma([0u8]));
                }
            }
            // Alt uzantılar: bandın en alt 1/4'ünde seyrek mürekkep
            let desc_start = top + (band_h * 3) / 4;
            for y in desc_start..top + band_h {
                let mut x = 30;
                while x < w - 20 {
                    img.put_pixel(x, y, Luma([0u8]));
                    x += 7;
                }
            }
        }
        img
    }

    /// Eksen algılama (0/180 vs 90/270) **güvenilirdir**: canlı ölçümde oran
    /// 4.00 (yatay) vs 0.25 (dikey) çıkar — 16 kat ayrım.
    ///
    /// NOT: Bu test aynı zamanda yön (0↔180, 90↔270) sinyalinin **yokluğunu**
    /// belgeler: bant içi ağırlık merkezi her yönde ~0.500 çıkar.
    #[test]
    fn test_axis_detection_is_reliable() {
        let base = synthetic_page(400, 600);
        for (name, page) in [
            ("upright", base.clone()),
            ("cw90", image::imageops::rotate90(&base)),
            ("cw180", image::imageops::rotate180(&base)),
            ("cw270", image::imageops::rotate270(&base)),
        ] {
            let (w, h) = (page.width() as usize, page.height() as usize);
            let ink = otsu_ink_mask(&page);
            let row_profile: Vec<f32> = (0..h)
                .map(|y| (0..w).filter(|&x| ink[y * w + x]).count() as f32)
                .collect();
            let col_profile: Vec<f32> = (0..w)
                .map(|x| (0..h).filter(|&y| ink[y * w + x]).count() as f32)
                .collect();
            let (vr, vc) = (variance(&row_profile), variance(&col_profile));
            let o = detect_orientation(&page);
            println!(
                "{name}: var_row={vr:.1} var_col={vc:.1} ratio={:.3} ch={:.3} cv={:.3} -> {:?} conf={:.2}",
                vr / vc.max(f32::EPSILON),
                band_centroid_ratio(&ink, w, h, true),
                band_centroid_ratio(&ink, w, h, false),
                o.rotation,
                o.confidence
            );
        }
    }

    #[test]
    fn test_detects_upright_page() {
        let page = synthetic_page(400, 600);
        let o = detect_orientation(&page);
        assert_eq!(o.rotation, Rotation::None, "dik sayfa döndürülmemeli");
    }

    /// 0↔180 ayrımı, basit sezgisel ile GÜVENİLİR DEĞİLDİR (ağırlık merkezi her
    /// iki yönde de 0.500). ONNX 4-sınıf sınıflandırıcı eklenene kadar yön kararı
    /// uygulanmaz; yanlış döndürme doğru veriyi bozar.
    #[test]
    #[ignore = "yön ayrımı ONNX sınıflandırıcı gerektirir (plan Bölüm 2.4 / Katman 3a)"]
    fn test_detects_180_page() {
        let page = image::imageops::rotate180(&synthetic_page(400, 600));
        let o = detect_orientation(&page);
        assert_eq!(o.rotation, Rotation::Cw180, "ters sayfa 180° düzeltilmeli");
    }

    #[test]
    #[ignore = "yön ayrımı ONNX sınıflandırıcı gerektirir (plan Bölüm 2.4 / Katman 3a)"]
    fn test_detects_sideways_pages() {
        let page = synthetic_page(400, 600);
        let cw90 = image::imageops::rotate90(&page);
        let cw270 = image::imageops::rotate270(&page);

        // Döndürülmüş sayfayı düzeltmek için ters yönde döndürmek gerekir.
        let a = detect_orientation(&cw90);
        let b = detect_orientation(&cw270);
        assert_eq!(
            a.rotation,
            Rotation::Cw270,
            "90° dönmüş sayfa 270° ile düzelir"
        );
        assert_eq!(
            b.rotation,
            Rotation::Cw90,
            "270° dönmüş sayfa 90° ile düzelir"
        );
    }

    #[test]
    fn test_otsu_mask_separates_text_from_paper() {
        let page = synthetic_page(200, 300);
        let mask = otsu_ink_mask(&page);
        let ink = mask.iter().filter(|m| **m).count();
        assert!(ink > 0, "mürekkep bulunmalı");
        assert!(ink < mask.len() / 2, "mürekkep sayfanın azınlığı olmalı");
    }
}
