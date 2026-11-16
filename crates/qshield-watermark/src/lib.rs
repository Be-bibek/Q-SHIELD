//! Q-SHIELD Watermark Verification Interfaces and Provenance Metadata.

use qshield_core::TileId;
use serde::{Deserialize, Serialize};

/// Record of transform-domain and fragile watermark verification results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatermarkVerificationRecord {
    pub watermark_id: String,
    pub robust_verified: bool,
    pub robust_normalized_correlation: f32,
    pub fragile_tampered: bool,
    pub fragile_tamper_ratio: f32,
    pub tampered_tiles: Vec<TileId>,
}

impl WatermarkVerificationRecord {
    pub fn authentic(watermark_id: impl Into<String>, nc: f32) -> Self {
        Self {
            watermark_id: watermark_id.into(),
            robust_verified: nc >= 0.85,
            robust_normalized_correlation: nc,
            fragile_tampered: false,
            fragile_tamper_ratio: 0.0,
            tampered_tiles: Vec::new(),
        }
    }

    pub fn tampered(
        watermark_id: impl Into<String>,
        nc: f32,
        tamper_ratio: f32,
        tampered_tiles: Vec<TileId>,
    ) -> Self {
        Self {
            watermark_id: watermark_id.into(),
            robust_verified: nc >= 0.85,
            robust_normalized_correlation: nc,
            fragile_tampered: tamper_ratio > 0.001,
            fragile_tamper_ratio: tamper_ratio,
            tampered_tiles,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermark_record() {
        let rec = WatermarkVerificationRecord::authentic("WM_2026_SAT", 0.98);
        assert!(rec.robust_verified);
        assert!(!rec.fragile_tampered);
    }
}
