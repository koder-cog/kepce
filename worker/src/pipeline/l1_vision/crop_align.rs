//! Belge kırpma ve perspektif hizalama (crop & perspective align) modülü
use anyhow::Result;
use image::DynamicImage;

pub fn align_document_image(img: &DynamicImage) -> Result<DynamicImage> {
    // Gelecekte DocAligner/ONNX perspektif düzeltme burada işletilebilir.
    Ok(img.clone())
}
