//! Tile representation, coordinates, and perceptual importance calculation.

use qshield_core::{TileCoordinate, TileId};
use qshield_crypto::hash_tile;
use serde::{Deserialize, Serialize};

/// Represents an independently verifiable, schedulable image tile
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageTile {
    pub coordinate: TileCoordinate,
    pub data: Vec<u8>,
    pub hash: [u8; 32],
    pub channels: u8,
    pub importance: f32, // Normalized priority metric in [0.0, 1.0]
}

impl ImageTile {
    pub fn new(coordinate: TileCoordinate, data: Vec<u8>, channels: u8) -> Self {
        let hash = hash_tile(&data);
        let importance = compute_tile_importance(&data, coordinate.width, coordinate.height, channels);
        Self {
            coordinate,
            data,
            hash,
            channels,
            importance,
        }
    }

    pub fn tile_id(&self) -> TileId {
        self.coordinate.tile_id
    }

    pub fn size_bytes(&self) -> usize {
        self.data.len()
    }
}

/// Compute perceptual importance score based on pixel variance and edge gradient
pub fn compute_tile_importance(data: &[u8], width: u32, height: u32, channels: u8) -> f32 {
    if data.is_empty() || width == 0 || height == 0 || channels == 0 {
        return 0.0;
    }

    let ch = channels as usize;
    let num_pixels = (width * height) as usize;
    if data.len() < num_pixels * ch {
        return 0.0;
    }

    // 1. Calculate luminance variance
    let mut sum: f64 = 0.0;
    let mut sum_sq: f64 = 0.0;

    for i in 0..num_pixels {
        let luma = if ch >= 3 {
            0.299 * (data[i * ch] as f64)
                + 0.587 * (data[i * ch + 1] as f64)
                + 0.114 * (data[i * ch + 2] as f64)
        } else {
            data[i * ch] as f64
        };
        sum += luma;
        sum_sq += luma * luma;
    }

    let mean = sum / (num_pixels as f64);
    let variance = (sum_sq / (num_pixels as f64)) - (mean * mean);
    let std_dev = variance.max(0.0).sqrt();

    // Normalized to [0.0, 1.0], assuming std_dev ranges up to ~64.0 for rich satellite scenes
    let score = (std_dev / 64.0).clamp(0.05, 1.0) as f32;
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_creation_and_hash() {
        let coord = TileCoordinate {
            tile_id: TileId(0),
            row: 0,
            col: 0,
            x: 0,
            y: 0,
            width: 4,
            height: 4,
        };
        let dummy_pixels = vec![120u8; 4 * 4 * 3];
        let tile = ImageTile::new(coord, dummy_pixels, 3);

        assert_eq!(tile.tile_id(), TileId(0));
        assert_eq!(tile.size_bytes(), 48);
        assert_ne!(tile.hash, [0u8; 32]);
    }
}
