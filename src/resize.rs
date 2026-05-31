use image::{DynamicImage, ImageEncoder, codecs::jpeg::JpegEncoder};
use std::io::Cursor;
use crate::params::ResizeParams;

pub fn process(bytes: &[u8], params: &ResizeParams) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| format!("failed to decode image: {e}"))?;

    let resized = apply_resize(img, params.width, params.height);

    encode_jpeg(&resized, params.quality)
}

fn apply_resize(img: DynamicImage, width: Option<u32>, height: Option<u32>) -> DynamicImage {
    let (orig_w, orig_h) = (img.width(), img.height());

    let (target_w, target_h) = match (width, height) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => {
            let h = (orig_h as f64 * w as f64 / orig_w as f64).round() as u32;
            (w, h.max(1))
        }
        (None, Some(h)) => {
            let w = (orig_w as f64 * h as f64 / orig_h as f64).round() as u32;
            (w.max(1), h)
        }
        (None, None) => return img,
    };

    // Skip upscaling
    if target_w >= orig_w && target_h >= orig_h {
        return img;
    }

    img.resize(target_w, target_h, image::imageops::FilterType::Lanczos3)
}

fn encode_jpeg(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, String> {
    let rgb = img.to_rgb8();
    let mut buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(Cursor::new(&mut buf), quality);
    encoder
        .write_image(rgb.as_raw(), rgb.width(), rgb.height(), image::ExtendedColorType::Rgb8)
        .map_err(|e| format!("failed to encode jpeg: {e}"))?;
    Ok(buf)
}
