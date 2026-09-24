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
use ort::session::Session;
use std::io::Cursor;
use std::sync::{Mutex, OnceLock};

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
// ONNX sayfa yönü sınıflandırıcı (4-sınıf: 0/90/180/270)
// ---------------------------------------------------------------------------

/// Model yolu ortam değişkeni; verilmezse imajdaki varsayılan yol kullanılır.
const MODEL_PATH_ENV: &str = "ORIENTATION_MODEL_PATH";
const DEFAULT_MODEL_PATH: &str = "/app/models/doc_orientation.onnx";

/// `PP-LCNet_x1_0_doc_ori` ön işleme sabitleri (resmî `inference.yml`):
/// kısa kenarı 256'ya ölçekle → 224 ortadan kırp → ImageNet normalize → CHW.
const MODEL_RESIZE_SHORT: u32 = 256;
const MODEL_INPUT_SIZE: u32 = 224;
const MODEL_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const MODEL_STD: [f32; 3] = [0.229, 0.224, 0.225];

/// Model sınıf sırası → uygulanacak saat yönü düzeltmesi.
///
/// `PP-LCNet_x1_0_doc_ori` sınıfları `['0','90','180','270']`'dir (sayfanın
/// **mevcut** dönüşü); düzeltme ters yöndedir. `calibrate_model_classes` testiyle
/// gerçek sayfalarda doğrulanır.
const CLASS_ROTATIONS: [Rotation; 4] = [
    Rotation::None,
    Rotation::Cw270,
    Rotation::Cw180,
    Rotation::Cw90,
];

/// Sınıflandırıcı kararının uygulanması için en düşük top-1 olasılığı ve
/// top-1 ile top-2 arasındaki en düşük fark (marj).
///
/// `PP-LCNet_x1_0_doc_ori` olasılıkları yayvandır (doğru sınıf ~0.40, diğerleri
/// ~0.20); bu yüzden mutlak eşik yerine **marj** esaslı karar daha güvenilirdir.
const CLASSIFIER_MIN_PROB: f32 = 0.35;
const CLASSIFIER_MIN_MARGIN: f32 = 0.15;

static ORIENTATION_SESSION: OnceLock<Option<Mutex<Session>>> = OnceLock::new();

/// ONNX oturumunu (süreç başına bir kez) yükler; model yoksa `None` döner ve
/// çağıran taraf projeksiyon profiline düşer (en iyi çaba).
fn orientation_session() -> Option<&'static Mutex<Session>> {
    ORIENTATION_SESSION
        .get_or_init(|| {
            let path = std::env::var(MODEL_PATH_ENV)
                .unwrap_or_else(|_| DEFAULT_MODEL_PATH.to_string());
            match load_orientation_session(&path) {
                Ok(session) => {
                    tracing::info!("Sayfa yönü ONNX modeli yüklendi: {path}");
                    Some(Mutex::new(session))
                }
                Err(e) => {
                    tracing::warn!(
                        "Sayfa yönü ONNX modeli yüklenemedi ({path}): {e:#}. Projeksiyon yöntemine düşülüyor."
                    );
                    None
                }
            }
        })
        .as_ref()
}

fn load_orientation_session(path: &str) -> Result<Session> {
    use ort::session::builder::GraphOptimizationLevel;
    // NOT: `ort::Error` `std::error::Error` implement etmediği için `anyhow::Context`
    // uygulanamaz; hata mesajları elle çevrilir.
    let mut builder = Session::builder()
        .map_err(|e| anyhow::anyhow!("ONNX Runtime oturum oluşturucu açılamadı: {e}"))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| anyhow::anyhow!("ONNX optimizasyon seviyesi ayarlanamadı: {e}"))?;
    builder
        .commit_from_file(path)
        .map_err(|e| anyhow::anyhow!("ONNX modeli açılamadı ({path}): {e}"))
}

/// Sayfanın ekseni: projeksiyon profiliyle güvenilir biçimde belirlenir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PageAxis {
    Horizontal,
    Vertical,
    Unknown,
}

/// Sınıflandırıcı sonucu: döndürme + top-1 olasılığı + top1-top2 marjı.
#[derive(Debug, Clone, Copy)]
struct Classification {
    rotation: Rotation,
    prob: f32,
    margin: f32,
}

/// Nihai yön kararı: ONNX sınıflandırıcı (olasılık + marj + eksen çapraz
/// doğrulamasıyla) → projeksiyon profili (yalnızca yeterince güvenliyse).
///
/// **Güvenlik ilkesi:** yanlış döndürme doğru veriyi bozar. Bu yüzden karar
/// yalnızca güvenilir olduğunda bir döndürme döner; aksi halde `Rotation::None`.
fn decide_orientation(img: &DynamicImage) -> PageOrientation {
    let gray = img.to_luma8();

    if let Some(c) = classify(img) {
        let trusted = c.prob >= CLASSIFIER_MIN_PROB && c.margin >= CLASSIFIER_MIN_MARGIN;
        if trusted && axis_agrees(&gray, c.rotation) {
            return PageOrientation {
                rotation: c.rotation,
                confidence: c.prob,
            };
        }
        tracing::debug!(
            "ONNX yön kararı güvenilmez (olasılık={:.2}, marj={:.2}), projeksiyona düşülüyor",
            c.prob,
            c.margin
        );
    }

    // Projeksiyon yalnızca yeterli güvende uygulanır (yön sinyali genelde yoktur).
    let projection = detect_orientation(&gray);
    if projection.confidence >= MIN_CONFIDENCE {
        projection
    } else {
        PageOrientation {
            rotation: Rotation::None,
            confidence: 0.0,
        }
    }
}

/// Sınıflandırıcının ekseni projeksiyon ekseniyle uyuşuyor mu? (eksen belirsizse evet)
fn axis_agrees(gray: &GrayImage, rotation: Rotation) -> bool {
    let classifier_horizontal = matches!(rotation, Rotation::None | Rotation::Cw180);
    match detect_axis(gray) {
        PageAxis::Horizontal => classifier_horizontal,
        PageAxis::Vertical => !classifier_horizontal,
        PageAxis::Unknown => true,
    }
}

/// Projeksiyon profiliyle sayfa ekseni (satır/sütun varyans oranı).
fn detect_axis(img: &GrayImage) -> PageAxis {
    let small = downscale(img);
    let (w, h) = (small.width() as usize, small.height() as usize);
    if w < 8 || h < 8 {
        return PageAxis::Unknown;
    }
    let ink = otsu_ink_mask(&small);
    let row_profile: Vec<f32> = (0..h)
        .map(|y| (0..w).filter(|&x| ink[y * w + x]).count() as f32)
        .collect();
    let col_profile: Vec<f32> = (0..w)
        .map(|x| (0..h).filter(|&y| ink[y * w + x]).count() as f32)
        .collect();
    let ratio = variance(&row_profile) / variance(&col_profile).max(f32::EPSILON);
    if ratio >= UPRIGHT_RATIO {
        PageAxis::Horizontal
    } else if ratio <= ROTATED_RATIO {
        PageAxis::Vertical
    } else {
        PageAxis::Unknown
    }
}

/// ONNX sınıflandırıcı sonucu (top-1 + marj); model yok/başarısızsa `None`.
fn classify(img: &DynamicImage) -> Option<Classification> {
    let probs = classify_probs(img)?;
    let (idx, &prob) = probs
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))?;
    let second = probs
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != idx)
        .map(|(_, &p)| p)
        .fold(f32::NEG_INFINITY, f32::max);
    Some(Classification {
        rotation: CLASS_ROTATIONS[idx],
        prob,
        margin: prob - second,
    })
}

/// (Yalnızca kalibrasyon testi için) sınıflandırıcı sonucu `PageOrientation` olarak.
#[cfg(test)]
fn classify_orientation(img: &DynamicImage) -> Option<PageOrientation> {
    classify(img).map(|c| PageOrientation {
        rotation: c.rotation,
        confidence: c.prob,
    })
}

/// 4 sınıfın softmax olasılıklarını döndürür (kalibrasyon testi de kullanır).
fn classify_probs(img: &DynamicImage) -> Option<Vec<f32>> {
    let session = orientation_session()?;
    let input = ort::value::Value::from_array(model_input_array(img)).ok()?;

    let mut guard = session.lock().ok()?;
    let input_name = guard.inputs().first()?.name().to_string();
    let outputs = guard.run(ort::inputs![input_name.as_str() => input]).ok()?;
    let (_, logits) = outputs[0].try_extract_tensor::<f32>().ok()?;
    if logits.len() < CLASS_ROTATIONS.len() {
        return None;
    }
    Some(softmax(&logits[..CLASS_ROTATIONS.len()]))
}

/// `PP-LCNet_x1_0_doc_ori` girdisi: kısa kenarı 256'ya ölçekle → ortadan 224
/// kırp → ImageNet normalize → CHW `[1,3,224,224]`.
fn model_input_array(img: &DynamicImage) -> ndarray::Array4<f32> {
    let rgb = img.to_rgb8();
    let (w, h) = (rgb.width().max(1), rgb.height().max(1));

    // 1) resize_short=256 (en-boy oranı korunur; kısa kenar 256 olur).
    let scale = MODEL_RESIZE_SHORT as f32 / w.min(h) as f32;
    let nw = ((w as f32 * scale).round() as u32).max(1);
    let nh = ((h as f32 * scale).round() as u32).max(1);
    let resized = image::imageops::resize(&rgb, nw, nh, image::imageops::FilterType::Triangle);

    // 2) center crop 224 (kısa kenar 256 olduğundan crop her zaman 224'tür).
    let side_u32 = MODEL_INPUT_SIZE.min(nw).min(nh);
    let left = (nw - side_u32) / 2;
    let top = (nh - side_u32) / 2;
    let cropped = image::imageops::crop_imm(&resized, left, top, side_u32, side_u32).to_image();

    // 3) ImageNet normalize + 4) CHW.
    let side = side_u32 as usize;
    let plane = side * side;
    let mut data = vec![0f32; 3 * plane];
    for (x, y, px) in cropped.enumerate_pixels() {
        let idx = y as usize * side + x as usize;
        for c in 0..3 {
            let v = px.0[c] as f32 / 255.0;
            data[c * plane + idx] = (v - MODEL_MEAN[c]) / MODEL_STD[c];
        }
    }

    // Şekil ve veri uzunluğu inşaat gereği tutarlıdır.
    ndarray::Array4::from_shape_vec((1, 3, side, side), data)
        .expect("ONNX girdi tensörü şekli veriyle tutarlı olmalı")
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let max = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = logits.iter().map(|v| (v - max).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.into_iter().map(|e| e / sum).collect()
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
    let exif_needed = exif_orientation != image::metadata::Orientation::NoTransforms;
    let mut img = DynamicImage::from_decoder(decoder).context("Görsel çözülemedi")?;
    img.apply_orientation(exif_orientation);

    // 2./3. Katman: yön algılama (ONNX sınıflandırıcı → projeksiyon profili).
    // `decide_orientation` kararı zaten güvenli biçimde verir (belirsizse None).
    let decision = decide_orientation(&img);
    let applied = decision.rotation;
    if applied == Rotation::None {
        tracing::debug!("sayfa 1/1: yön belirsiz, dokunulmadı");
    }

    // Ne yön düzeltmesi ne EXIF normalizasyonu gerekiyorsa orijinal baytlar döner.
    if applied == Rotation::None && !exif_needed {
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

    if applied != Rotation::None {
        tracing::info!(
            "sayfa 1/1: algılanan yön={}°, düzeltildi (güven={:.2})",
            applied.degrees(),
            decision.confidence
        );
    } else {
        // Yalnızca EXIF normalizasyonu uygulandı (pikseller yeniden kodlanır).
        tracing::debug!("sayfa 1/1: EXIF yönü normalize edildi (ek döndürme yok)");
    }

    let out_mime = encode_image(&rotated, mime_type)?;
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

/// PDF yön düzeltmesi için en fazla sayfa sayısı (CPU/maliyet sınırı).
fn max_pdf_pages() -> usize {
    std::env::var("ORIENTATION_MAX_PAGES")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(50)
}

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

    // Kaynak sınırı: çok sayfalı PDF'lerde rasterizasyon maliyetini sınırla.
    let page_count = doc.pages().len() as usize;
    let max_pages = max_pdf_pages();
    if page_count > max_pages {
        anyhow::bail!(
            "PDF sayfa sayısı üst sınırı aşıldı ({} > {}); yön düzeltmesi atlanıyor",
            page_count,
            max_pages
        );
    }

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
        out.push(decide_orientation(&img));
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
    /// Yatay 4.00, dikey 0.25 çıkar ve 16 kat ayrım oluşur.
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
    #[ignore = "projeksiyon yöntemi yön ayrımı yapamaz; sınıflandırıcı için calibrate_model_classes"]
    fn test_detects_180_page() {
        let page = image::imageops::rotate180(&synthetic_page(400, 600));
        let o = detect_orientation(&page);
        assert_eq!(o.rotation, Rotation::Cw180, "ters sayfa 180° düzeltilmeli");
    }

    #[test]
    #[ignore = "projeksiyon yöntemi yön ayrımı yapamaz; sınıflandırıcı için calibrate_model_classes"]
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

    /// ONNX sınıflandırıcıyı gerçek modelle kalibre eder (model yolu env'de olmalı).
    ///
    /// `#[ignore]`: model dosyası + ONNX Runtime gerektirir. Çalıştırma:
    /// ```text
    /// ORIENTATION_MODEL_PATH=/tmp/page_orientation.onnx \
    ///   cargo test -p worker calibrate_model_classes -- --ignored --nocapture
    /// ```
    /// Dik bir sayfayı 0/90/180/270 döndürüp her biri için olasılıkları basar;
    /// `CLASS_ROTATIONS` eşlemesi bu çıktıya göre doğrulanır.
    #[test]
    #[ignore = "ONNX modeli gerektirir (ORIENTATION_MODEL_PATH)"]
    fn calibrate_model_classes() {
        // Gerçek bir belge sayfası (`ORIENTATION_CALIB_IMAGE`) verilirse onu,
        // yoksa sentetik sayfayı kullan (sentetik sayfa model için dejenere olabilir).
        let base = std::env::var("ORIENTATION_CALIB_IMAGE")
            .ok()
            .and_then(|p| image::open(&p).ok())
            .map(|i| i.to_luma8())
            .unwrap_or_else(|| synthetic_page(400, 600));

        for (name, angle) in [
            ("upright", 0u16),
            ("cw90", 90),
            ("cw180", 180),
            ("cw270", 270),
        ] {
            let img = match angle {
                90 => DynamicImage::ImageLuma8(image::imageops::rotate90(&base)),
                180 => DynamicImage::ImageLuma8(image::imageops::rotate180(&base)),
                270 => DynamicImage::ImageLuma8(image::imageops::rotate270(&base)),
                _ => DynamicImage::ImageLuma8(base.clone()),
            };
            match classify_probs(&img) {
                Some(probs) => {
                    let mapped = classify_orientation(&img);
                    let decided = decide_orientation(&img);
                    println!(
                        "{name} ({angle}°): probs={:?} -> map={:?} | karar={:?}",
                        probs
                            .iter()
                            .map(|p| (p * 1000.0).round() / 1000.0)
                            .collect::<Vec<_>>(),
                        mapped.map(|m| (m.rotation, m.confidence)),
                        (decided.rotation, decided.confidence)
                    );
                }
                None => println!("{name} ({angle}°): model yok/başarısız"),
            }
        }
    }

    /// PDF yön düzeltme boru hattını (pdfium + ONNX) gerçek bir PDF üzerinde çalıştırır.
    ///
    /// `#[ignore]`: libpdfium.so gerektirir. Yerelde:
    /// ```text
    /// LD_LIBRARY_PATH=/tmp/pdfium-x64/lib ORIENTATION_CALIB_PDF=data/.../menu.pdf \
    ///   cargo test -p worker probe_correct_pdf -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "libpdfium.so + PDF gerektirir (ORIENTATION_CALIB_PDF)"]
    fn probe_correct_pdf() {
        let Ok(path) = std::env::var("ORIENTATION_CALIB_PDF") else {
            eprintln!("PROBE atlandı: ORIENTATION_CALIB_PDF ayarlı değil.");
            return;
        };
        let bytes = std::fs::read(&path).expect("PDF okunamadı");
        let result = correct_document(&bytes, "application/pdf");
        eprintln!(
            "PDF PROBE: dosya={path} | {} sayfa | {} düzeltildi | çıktı {} bayt",
            result.pages,
            result.corrected_pages,
            result.bytes.len()
        );
        assert!(result.pages > 0, "en az bir sayfa bulunmalı");
    }
}
