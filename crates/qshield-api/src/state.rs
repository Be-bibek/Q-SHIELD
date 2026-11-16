//! Application state for the Q-SHIELD REST API service.

use qshield_core::{ImageDimensions, ImageFormat, ImageId, TileVerificationStatus, TransferId};
use qshield_fragment::EncryptedFragment;
use qshield_manifest::SignedManifest;
use qshield_scheduler::{ContactWindowResource, ScheduleResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredImage {
    pub image_id: ImageId,
    pub dimensions: ImageDimensions,
    pub format: ImageFormat,
    pub total_raw_bytes: usize,
    pub sha256_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferSummary {
    pub transfer_id: TransferId,
    pub image_id: ImageId,
    pub sender_id: String,
    pub receiver_id: String,
    pub status: String,
    pub progress_ratio: f32,
    pub total_fragments: usize,
    pub acknowledged_fragments: usize,
    pub transmitted_bytes: u64,
    pub retransmitted_bytes: u64,
    pub interruption_count: u32,
    pub verified_tile_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationDetails {
    pub transfer_id: TransferId,
    pub image_id: ImageId,
    pub signature_algorithm: String,
    pub signature_verified: bool,
    pub kem_algorithm: String,
    pub kem_established: bool,
    pub merkle_root: String,
    pub merkle_verified: bool,
    pub total_tiles: usize,
    pub verified_tiles: usize,
    pub tile_statuses: HashMap<String, TileVerificationStatus>,
}

#[derive(Clone)]
pub struct AppState {
    pub images: Arc<RwLock<HashMap<ImageId, (RegisteredImage, Vec<u8>)>>>,
    pub transfers: Arc<RwLock<HashMap<TransferId, TransferSummary>>>,
    pub transfer_fragments: Arc<RwLock<HashMap<TransferId, Vec<EncryptedFragment>>>>,
    pub transfer_manifests: Arc<RwLock<HashMap<TransferId, SignedManifest>>>,
    pub transfer_verifications: Arc<RwLock<HashMap<TransferId, VerificationDetails>>>,
    pub contact_windows: Arc<RwLock<Vec<ContactWindowResource>>>,
    pub simulation_results: Arc<RwLock<Vec<ScheduleResult>>>,
}

impl AppState {
    pub fn new() -> Self {
        // Pre-populate realistic LEO Svalbard contact windows
        let default_windows = vec![
            ContactWindowResource {
                window_id: "CW_SENTINEL2A_SVALBARD_001".into(),
                start_seconds: 0.0,
                end_seconds: 620.0,
                duration_seconds: 620.0,
                capacity_bytes: (150_000_000.0 * 620.0 / 8.0 * 0.95) as u64, // 150 Mbps downlink
                data_rate_bps: 150_000_000.0,
            },
            ContactWindowResource {
                window_id: "CW_SENTINEL2A_SVALBARD_002".into(),
                start_seconds: 5940.0, // next orbit (~99 mins later)
                end_seconds: 6580.0,
                duration_seconds: 640.0,
                capacity_bytes: (150_000_000.0 * 640.0 / 8.0 * 0.95) as u64,
                data_rate_bps: 150_000_000.0,
            },
        ];

        Self {
            images: Arc::new(RwLock::new(HashMap::new())),
            transfers: Arc::new(RwLock::new(HashMap::new())),
            transfer_fragments: Arc::new(RwLock::new(HashMap::new())),
            transfer_manifests: Arc::new(RwLock::new(HashMap::new())),
            transfer_verifications: Arc::new(RwLock::new(HashMap::new())),
            contact_windows: Arc::new(RwLock::new(default_windows)),
            simulation_results: Arc::new(RwLock::new(Vec::new())),
        }
    }
}
