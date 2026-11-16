//! Adaptive and grid-based image tiling and lossless reassembly.

use crate::tile::ImageTile;
use qshield_core::{ImageDimensions, QShieldError, Result, TileCoordinate, TileId};

#[derive(Debug, Clone, Copy)]
pub struct TilerConfig {
    pub tile_width: u32,
    pub tile_height: u32,
}

impl Default for TilerConfig {
    fn default() -> Self {
        Self {
            tile_width: 256,
            tile_height: 256,
        }
    }
}

/// Slices raw pixel buffer into grid of tiles
pub fn slice_image(
    raw_pixels: &[u8],
    dimensions: &ImageDimensions,
    config: &TilerConfig,
) -> Result<Vec<ImageTile>> {
    let img_w = dimensions.width;
    let img_h = dimensions.height;
    let ch = dimensions.channels as usize;

    let expected_len = (img_w as usize) * (img_h as usize) * ch;
    if raw_pixels.len() != expected_len {
        return Err(QShieldError::ImageError(format!(
            "Raw pixel buffer size mismatch: expected {} bytes, got {}",
            expected_len,
            raw_pixels.len()
        )));
    }

    let cols = (img_w + config.tile_width - 1) / config.tile_width;
    let rows = (img_h + config.tile_height - 1) / config.tile_height;

    let mut tiles = Vec::with_capacity((rows * cols) as usize);
    let mut tile_counter = 0u32;

    for r in 0..rows {
        for c in 0..cols {
            let x0 = c * config.tile_width;
            let y0 = r * config.tile_height;
            let actual_w = (config.tile_width).min(img_w - x0);
            let actual_h = (config.tile_height).min(img_h - y0);

            let mut tile_data = Vec::with_capacity((actual_w * actual_h) as usize * ch);

            for row_offset in 0..actual_h {
                let y = y0 + row_offset;
                let src_start = ((y * img_w + x0) as usize) * ch;
                let src_end = src_start + (actual_w as usize) * ch;
                tile_data.extend_from_slice(&raw_pixels[src_start..src_end]);
            }

            let coordinate = TileCoordinate {
                tile_id: TileId(tile_counter),
                row: r,
                col: c,
                x: x0,
                y: y0,
                width: actual_w,
                height: actual_h,
            };

            tiles.push(ImageTile::new(coordinate, tile_data, dimensions.channels));
            tile_counter += 1;
        }
    }

    Ok(tiles)
}

/// Losslessly reassembles tiles into a full raw image buffer
pub fn reassemble_image(
    tiles: &[ImageTile],
    dimensions: &ImageDimensions,
) -> Result<Vec<u8>> {
    let img_w = dimensions.width as usize;
    let img_h = dimensions.height as usize;
    let ch = dimensions.channels as usize;

    let total_bytes = img_w * img_h * ch;
    let mut reconstructed = vec![0u8; total_bytes];

    for tile in tiles {
        let coord = &tile.coordinate;
        let x0 = coord.x as usize;
        let y0 = coord.y as usize;
        let tw = coord.width as usize;
        let th = coord.height as usize;

        let expected_tile_bytes = tw * th * ch;
        if tile.data.len() != expected_tile_bytes {
            return Err(QShieldError::ImageError(format!(
                "Tile {} corrupted data size: expected {}, got {}",
                coord.tile_id,
                expected_tile_bytes,
                tile.data.len()
            )));
        }

        for row_idx in 0..th {
            let y = y0 + row_idx;
            let dst_start = (y * img_w + x0) * ch;
            let dst_end = dst_start + tw * ch;

            let src_start = row_idx * tw * ch;
            let src_end = src_start + tw * ch;

            reconstructed[dst_start..dst_end].copy_from_slice(&tile.data[src_start..src_end]);
        }
    }

    Ok(reconstructed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiling_and_lossless_reassembly_roundtrip() {
        let dimensions = ImageDimensions {
            width: 500, // Non-multiple of 256 to test boundary handling
            height: 350,
            channels: 3,
            bit_depth: 8,
        };

        // Create deterministic test image pattern
        let mut original_pixels = Vec::with_capacity((500 * 350 * 3) as usize);
        for i in 0..(500 * 350 * 3) {
            original_pixels.push((i % 256) as u8);
        }

        let config = TilerConfig {
            tile_width: 128,
            tile_height: 128,
        };

        let tiles = slice_image(&original_pixels, &dimensions, &config).unwrap();
        // 500 / 128 = 4 cols, 350 / 128 = 3 rows -> 12 tiles
        assert_eq!(tiles.len(), 12);

        let reconstructed = reassemble_image(&tiles, &dimensions).unwrap();
        assert_eq!(original_pixels, reconstructed);
    }
}
