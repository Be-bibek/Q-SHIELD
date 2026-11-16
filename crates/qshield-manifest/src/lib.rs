//! Q-SHIELD Canonical Image Manifest and Post-Quantum Digital Signature Verification.

use chrono::Utc;
use qshield_core::{
    ImageDimensions, ImageFormat, ImageId, QShieldError, Result, TileCoordinate, TransferId,
    CRYPTO_SUITE_ID, PROTOCOL_VERSION,
};
use qshield_crypto::{
    sha256_digest, PqcSignature, PqcSigningKey, PqcVerifyingKey, MANIFEST_HASH_PREFIX,
};
use serde::{Deserialize, Serialize};

/// Canonical representation of an image transfer manifest
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageManifest {
    pub protocol_version: String,
    pub transfer_id: TransferId,
    pub image_id: ImageId,
    pub source_id: String,
    pub destination_id: String,
    pub dimensions: ImageDimensions,
    pub format: ImageFormat,
    pub original_image_hash: [u8; 32],
    pub tile_count: u32,
    pub tile_coordinates: Vec<TileCoordinate>,
    pub tile_hashes: Vec<[u8; 32]>,
    pub merkle_root: [u8; 32],
    pub watermark_id: Option<String>,
    pub crypto_suite: String,
    pub created_at_utc: String,
    pub image_priority: f32,
}

impl ImageManifest {
    pub fn new(
        transfer_id: TransferId,
        image_id: ImageId,
        source_id: impl Into<String>,
        destination_id: impl Into<String>,
        dimensions: ImageDimensions,
        format: ImageFormat,
        original_image_hash: [u8; 32],
        tile_coordinates: Vec<TileCoordinate>,
        tile_hashes: Vec<[u8; 32]>,
        merkle_root: [u8; 32],
        watermark_id: Option<String>,
        image_priority: f32,
    ) -> Result<Self> {
        if tile_coordinates.len() != tile_hashes.len() {
            return Err(QShieldError::ManifestError(
                "Tile coordinates count does not match tile hashes count".into(),
            ));
        }

        Ok(Self {
            protocol_version: PROTOCOL_VERSION.to_string(),
            transfer_id,
            image_id,
            source_id: source_id.into(),
            destination_id: destination_id.into(),
            dimensions,
            format,
            original_image_hash,
            tile_count: tile_coordinates.len() as u32,
            tile_coordinates,
            tile_hashes,
            merkle_root,
            watermark_id,
            crypto_suite: CRYPTO_SUITE_ID.to_string(),
            created_at_utc: Utc::now().to_rfc3339(),
            image_priority,
        })
    }

    /// Canonical, deterministic serialization to prevent duplicate-field interpretation and ambiguity
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();

        out.extend_from_slice(MANIFEST_HASH_PREFIX);
        out.extend_from_slice(format!("V:{}\n", self.protocol_version).as_bytes());
        out.extend_from_slice(format!("TID:{}\n", self.transfer_id).as_bytes());
        out.extend_from_slice(format!("IMG:{}\n", self.image_id).as_bytes());
        out.extend_from_slice(format!("SRC:{}\n", self.source_id).as_bytes());
        out.extend_from_slice(format!("DST:{}\n", self.destination_id).as_bytes());
        out.extend_from_slice(
            format!(
                "DIM:{}x{}x{}b{}\n",
                self.dimensions.width,
                self.dimensions.height,
                self.dimensions.channels,
                self.dimensions.bit_depth
            )
            .as_bytes(),
        );
        out.extend_from_slice(format!("FMT:{}\n", self.format).as_bytes());
        out.extend_from_slice(
            format!("ORIG_HASH:{}\n", hex::encode(self.original_image_hash)).as_bytes(),
        );
        out.extend_from_slice(format!("TILES_N:{}\n", self.tile_count).as_bytes());
        out.extend_from_slice(format!("M_ROOT:{}\n", hex::encode(self.merkle_root)).as_bytes());
        out.extend_from_slice(
            format!("WM_ID:{}\n", self.watermark_id.as_deref().unwrap_or("NONE")).as_bytes(),
        );
        out.extend_from_slice(format!("CSUITE:{}\n", self.crypto_suite).as_bytes());
        out.extend_from_slice(format!("TIME:{}\n", self.created_at_utc).as_bytes());
        out.extend_from_slice(format!("PRIO:{:.4}\n", self.image_priority).as_bytes());

        // Deterministic listing of tile coordinates and hashes
        for (i, coord) in self.tile_coordinates.iter().enumerate() {
            out.extend_from_slice(
                format!(
                    "TILE_{}:{}:{},{}:{},{}:{}x{}:{}\n",
                    i,
                    coord.tile_id.0,
                    coord.row,
                    coord.col,
                    coord.x,
                    coord.y,
                    coord.width,
                    coord.height,
                    hex::encode(self.tile_hashes[i])
                )
                .as_bytes(),
            );
        }

        out
    }

    /// Compute canonical digest of the manifest
    pub fn digest(&self) -> [u8; 32] {
        sha256_digest(&self.canonical_bytes())
    }

    /// Sign the manifest canonically with ML-DSA-65
    pub fn sign(&self, signing_key: &PqcSigningKey) -> SignedManifest {
        let digest = self.digest();
        let signature = signing_key.sign(&digest);
        let signer_verifying_key = signing_key.verifying_key();

        SignedManifest {
            manifest: self.clone(),
            signature,
            signer_verifying_key,
        }
    }
}

/// Digitally signed image manifest containing ML-DSA-65 post-quantum proof of authenticity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedManifest {
    pub manifest: ImageManifest,
    pub signature: PqcSignature,
    pub signer_verifying_key: PqcVerifyingKey,
}

impl SignedManifest {
    /// Verify ML-DSA-65 signature on the canonical manifest digest
    pub fn verify_signature(&self) -> Result<()> {
        let digest = self.manifest.digest();
        self.signer_verifying_key
            .verify(&digest, &self.signature)
            .map_err(|e| {
                QShieldError::SignatureVerificationFailed(format!(
                    "Manifest post-quantum signature verification failed: {e}"
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qshield_core::TileId;

    #[test]
    fn test_manifest_creation_canonical_signing_and_verification() {
        let tid = TransferId::new_v4();
        let iid = ImageId::new("img_sat_sentinel_001");
        let dims = ImageDimensions {
            width: 512,
            height: 512,
            channels: 3,
            bit_depth: 8,
        };
        let coord = TileCoordinate {
            tile_id: TileId(0),
            row: 0,
            col: 0,
            x: 0,
            y: 0,
            width: 256,
            height: 256,
        };

        let orig_hash = [0xAA; 32];
        let tile_hash = [0xBB; 32];
        let merkle_root = [0xCC; 32];

        let manifest = ImageManifest::new(
            tid,
            iid,
            "SAT_LEO_01",
            "GS_TROMSO",
            dims,
            ImageFormat::Png,
            orig_hash,
            vec![coord],
            vec![tile_hash],
            merkle_root,
            Some("WM_PROV_2026_01".into()),
            0.95,
        )
        .unwrap();

        // Sender signs with ML-DSA-65
        let sk = PqcSigningKey::generate();
        let signed_manifest = manifest.sign(&sk);

        // Verification must succeed
        assert!(signed_manifest.verify_signature().is_ok());

        // Tampering with any field in manifest must cause signature verification failure
        let mut tampered = signed_manifest.clone();
        tampered.manifest.dimensions.width = 1024;
        assert!(tampered.verify_signature().is_err());

        // Tampering with tile hash must cause failure
        let mut tampered_tiles = signed_manifest.clone();
        tampered_tiles.manifest.tile_hashes[0][0] ^= 0x01;
        assert!(tampered_tiles.verify_signature().is_err());
    }
}
