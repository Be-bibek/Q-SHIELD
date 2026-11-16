//! Q-SHIELD Core: Fundamental types, error definitions, identifiers, and state models.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

pub const PROTOCOL_VERSION: &str = "QSHIELD-V1.0";
pub const CRYPTO_SUITE_ID: &str = "ML-KEM-768/ML-DSA-65/AES-256-GCM/HKDF-SHA256";

#[derive(thiserror::Error, Debug)]
pub enum QShieldError {
    #[error("Cryptographic operation failed: {0}")]
    CryptoError(String),

    #[error("Post-quantum signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    #[error("Authentication tag validation failed: {0}")]
    AuthTagMismatch(String),

    #[error("Merkle inclusion proof failed for leaf {0}: {1}")]
    MerkleVerificationFailed(usize, String),

    #[error("Invalid canonical manifest: {0}")]
    ManifestError(String),

    #[error("Image processing error: {0}")]
    ImageError(String),

    #[error("Fragmentation error: {0}")]
    FragmentationError(String),

    #[error("Transfer protocol error: {0}")]
    TransferError(String),

    #[error("Scheduler error: {0}")]
    SchedulerError(String),

    #[error("Receiver verification error: {0}")]
    ReceiverError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, QShieldError>;

/// Strongly-typed Transfer Identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransferId(pub Uuid);

impl TransferId {
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

impl fmt::Display for TransferId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for TransferId {
    fn default() -> Self {
        Self::new_v4()
    }
}

/// Strongly-typed Image Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImageId(pub String);

impl ImageId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn random() -> Self {
        Self(format!("img_{}", Uuid::new_v4().simple()))
    }
}

impl fmt::Display for ImageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Strongly-typed Tile Identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TileId(pub u32);

impl fmt::Display for TileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tile_{:04}", self.0)
    }
}

/// Strongly-typed Fragment Identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FragmentId(pub u32);

impl fmt::Display for FragmentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "frag_{:05}", self.0)
    }
}

/// Tile Coordinate in image space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoordinate {
    pub tile_id: TileId,
    pub row: u32,
    pub col: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Dimensions of an image
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub bit_depth: u8,
}

/// Format of the image
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Tiff,
    GeoTiff,
    RawRgb,
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Png => write!(f, "PNG"),
            Self::Jpeg => write!(f, "JPEG"),
            Self::Tiff => write!(f, "TIFF"),
            Self::GeoTiff => write!(f, "GeoTIFF"),
            Self::RawRgb => write!(f, "RawRGB"),
        }
    }
}

/// Complete lifecycle state of a transmission fragment
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FragmentState {
    Created,
    Encrypted,
    Queued,
    Transmitted,
    Received,
    Authenticated,
    Verified,
    Reconstructed,
    Failed,
}

impl fmt::Display for FragmentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "CREATED"),
            Self::Encrypted => write!(f, "ENCRYPTED"),
            Self::Queued => write!(f, "QUEUED"),
            Self::Transmitted => write!(f, "TRANSMITTED"),
            Self::Received => write!(f, "RECEIVED"),
            Self::Authenticated => write!(f, "AUTHENTICATED"),
            Self::Verified => write!(f, "VERIFIED"),
            Self::Reconstructed => write!(f, "RECONSTRUCTED"),
            Self::Failed => write!(f, "FAILED"),
        }
    }
}

/// Operational Scenario
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalScenario {
    /// Scenario A: Earth observation satellite downlinking to ground station
    ScenarioA,
    /// Scenario B: Field device relaying via satellite network to gateway
    ScenarioB,
}

/// Verification metrics and indicators
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TileVerificationStatus {
    pub received: bool,
    pub decrypted: bool,
    pub signature_verified: bool,
    pub hash_verified: bool,
    pub merkle_verified: bool,
    pub watermark_verified: bool,
    pub reconstructed: bool,
}

impl TileVerificationStatus {
    pub fn is_fully_verified(&self) -> bool {
        self.received
            && self.decrypted
            && self.signature_verified
            && self.hash_verified
            && self.merkle_verified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifiers_and_states() {
        let tid = TransferId::new_v4();
        let iid = ImageId::new("img_test_01");
        let tile_id = TileId(42);
        let frag_id = FragmentId(101);

        assert_eq!(tile_id.to_string(), "tile_0042");
        assert_eq!(frag_id.to_string(), "frag_00101");
        assert_eq!(iid.to_string(), "img_test_01");
        assert!(!tid.to_string().is_empty());

        let state = FragmentState::Verified;
        assert_eq!(state.to_string(), "VERIFIED");
    }

    #[test]
    fn test_verification_status() {
        let mut status = TileVerificationStatus::default();
        assert!(!status.is_fully_verified());

        status.received = true;
        status.decrypted = true;
        status.signature_verified = true;
        status.hash_verified = true;
        status.merkle_verified = true;
        assert!(status.is_fully_verified());
    }
}
