//! Image decoding, format detection, and export.

use image::{DynamicImage, GenericImageView, ImageFormat as ImgFormat};
use qshield_core::{ImageDimensions, ImageFormat, QShieldError, Result};
use std::io::Cursor;

/// Decodes an image from raw file bytes, returning raw RGB/RGBA pixels, dimensions, and detected format
pub fn decode_image(bytes: &[u8]) -> Result<(Vec<u8>, ImageDimensions, ImageFormat)> {
    let guessed_format = image::guess_format(bytes)
        .map_err(|e| QShieldError::ImageError(format!("Failed to determine image format: {e}")))?;

    let format = match guessed_format {
        ImgFormat::Png => ImageFormat::Png,
        ImgFormat::Jpeg => ImageFormat::Jpeg,
        ImgFormat::Tiff => ImageFormat::Tiff,
        _ => ImageFormat::Png,
    };

    let img = image::load_from_memory(bytes)
        .map_err(|e| QShieldError::ImageError(format!("Image decoding error: {e}")))?;

    let (width, height) = img.dimensions();
    let rgb = img.to_rgb8();
    let raw_pixels = rgb.into_raw();

    let dimensions = ImageDimensions {
        width,
        height,
        channels: 3,
        bit_depth: 8,
    };

    Ok((raw_pixels, dimensions, format))
}

/// Encodes raw RGB pixels back to PNG bytes
pub fn encode_to_png(raw_pixels: &[u8], dimensions: &ImageDimensions) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let img_buf: image::RgbImage = image::ImageBuffer::from_raw(
        dimensions.width,
        dimensions.height,
        raw_pixels.to_vec(),
    )
    .ok_or_else(|| QShieldError::ImageError("Failed to build image buffer from raw pixels".into()))?;

    let dynamic = DynamicImage::ImageRgb8(img_buf);
    dynamic
        .write_to(&mut Cursor::new(&mut buffer), ImgFormat::Png)
        .map_err(|e| QShieldError::ImageError(format!("Failed to encode image to PNG: {e}")))?;

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_and_decode_png_roundtrip() {
        let dimensions = ImageDimensions {
            width: 16,
            height: 16,
            channels: 3,
            bit_depth: 8,
        };
        let mut pixels = vec![0u8; 16 * 16 * 3];
        for (i, p) in pixels.iter_mut().enumerate() {
            *p = (i * 7 % 256) as u8;
        }

        let png_bytes = encode_to_png(&pixels, &dimensions).expect("encode failed");
        let (decoded_pixels, decoded_dim, format) = decode_image(&png_bytes).expect("decode failed");

        assert_eq!(format, ImageFormat::Png);
        assert_eq!(decoded_dim, dimensions);
        assert_eq!(pixels, decoded_pixels);
    }
}
