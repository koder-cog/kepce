//! Sayfa eğiklik düzeltme (deskew) modülü
use anyhow::Result;
use image::DynamicImage;

pub fn deskew_image(img: &DynamicImage) -> Result<DynamicImage> {
    Ok(img.clone())
}
