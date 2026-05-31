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

    img.resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::ResizeParams;

    // Build a 200x100 solid-colour JPEG in memory as test input
    fn make_test_jpeg(w: u32, h: u32) -> Vec<u8> {
        let img = DynamicImage::new_rgb8(w, h);
        encode_jpeg(&img, 90).expect("encode test image")
    }

    fn decoded_dims(jpeg: &[u8]) -> (u32, u32) {
        let img = image::load_from_memory(jpeg).expect("decode output");
        (img.width(), img.height())
    }

    fn params(width: Option<u32>, height: Option<u32>, quality: u8) -> ResizeParams {
        ResizeParams {
            src: "https://example.com/img.jpg".into(),
            width,
            height,
            quality,
        }
    }

    #[test]
    fn resize_by_width_preserves_aspect_ratio() {
        let src = make_test_jpeg(200, 100);
        let out = process(&src, &params(Some(100), None, 85)).unwrap();
        let (w, h) = decoded_dims(&out);
        assert_eq!(w, 100);
        assert_eq!(h, 50); // 100 * (100/200) = 50
    }

    #[test]
    fn resize_by_height_preserves_aspect_ratio() {
        let src = make_test_jpeg(200, 100);
        let out = process(&src, &params(None, Some(50), 85)).unwrap();
        let (w, h) = decoded_dims(&out);
        assert_eq!(h, 50);
        assert_eq!(w, 100); // 200 * (50/100) = 100
    }

    #[test]
    fn resize_both_dims_exact() {
        let src = make_test_jpeg(200, 100);
        let out = process(&src, &params(Some(80), Some(40), 85)).unwrap();
        let (w, h) = decoded_dims(&out);
        assert_eq!(w, 80);
        assert_eq!(h, 40);
    }

    #[test]
    fn upscale_is_skipped() {
        let src = make_test_jpeg(100, 50);
        let out = process(&src, &params(Some(500), None, 85)).unwrap();
        let (w, h) = decoded_dims(&out); // should stay at original size
        assert_eq!(w, 100);
        assert_eq!(h, 50);
    }

    #[test]
    fn lower_quality_produces_smaller_file() {
        let src = make_test_jpeg(200, 200);
        let high = process(&src, &params(Some(200), None, 95)).unwrap();
        let low  = process(&src, &params(Some(200), None, 10)).unwrap();
        assert!(low.len() < high.len(), "q=10 should be smaller than q=95");
    }

    #[test]
    fn invalid_image_bytes_returns_error() {
        let bad = b"not an image";
        let result = process(bad, &params(Some(100), None, 85));
        assert!(result.is_err());
    }
}
