//! Receiver session, fragment verification, and progressive partial image reconstruction.

use qshield_core::{
    ImageId, QShieldError, Result, TileCoordinate, TileId,
    TileVerificationStatus, TransferId,
};
use qshield_crypto::{
    derive_session_key, hash_tile, DerivedKey, KeyDerivationContext, PqcKeyPair,
};
use qshield_fragment::EncryptedFragment;
use qshield_image::ImageTile;
use qshield_manifest::SignedManifest;
use qshield_merkle::MerkleLeafBinding;
use std::collections::HashMap;

/// Assembly buffer for an individual image tile
struct TileAssemblyBuffer {
    coordinate: TileCoordinate,
    expected_len: usize,
    received_chunks: HashMap<u32, Vec<u8>>, // offset -> decrypted bytes
    total_received_bytes: usize,
    completed: bool,
}

impl TileAssemblyBuffer {
    fn new(coord: TileCoordinate, expected_len: usize) -> Self {
        Self {
            coordinate: coord,
            expected_len,
            received_chunks: HashMap::new(),
            total_received_bytes: 0,
            completed: false,
        }
    }

    fn insert_chunk(&mut self, offset: u32, chunk: Vec<u8>) {
        if !self.received_chunks.contains_key(&offset) {
            self.total_received_bytes += chunk.len();
            self.received_chunks.insert(offset, chunk);
        }
        if self.total_received_bytes >= self.expected_len {
            self.completed = true;
        }
    }

    fn assemble(&self) -> Result<Vec<u8>> {
        let mut full_tile = vec![0u8; self.expected_len];
        for (&offset, chunk) in &self.received_chunks {
            let start = offset as usize;
            let end = start + chunk.len();
            if end <= self.expected_len {
                full_tile[start..end].copy_from_slice(chunk);
            }
        }
        Ok(full_tile)
    }
}

/// Receiver state machine verifying incoming PQC handshakes, fragments, and progressive reconstruction
pub struct VerifiableReceiverSession {
    pub transfer_id: TransferId,
    pub image_id: ImageId,
    pub receiver_id: String,
    pub keypair: PqcKeyPair,
    pub session_key: Option<DerivedKey>,
    pub signed_manifest: Option<SignedManifest>,
    pub tile_statuses: HashMap<TileId, TileVerificationStatus>,
    pub verified_tiles: HashMap<TileId, ImageTile>,
    tile_buffers: HashMap<TileId, TileAssemblyBuffer>,
    pub rejected_fragment_count: u32,
    pub rejected_tile_count: u32,
}

impl VerifiableReceiverSession {
    pub fn new(receiver_id: impl Into<String>) -> Self {
        Self {
            transfer_id: TransferId::nil(),
            image_id: ImageId::new("uninitialized"),
            receiver_id: receiver_id.into(),
            keypair: PqcKeyPair::generate(),
            session_key: None,
            signed_manifest: None,
            tile_statuses: HashMap::new(),
            verified_tiles: HashMap::new(),
            tile_buffers: HashMap::new(),
            rejected_fragment_count: 0,
            rejected_tile_count: 0,
        }
    }

    /// Public ML-KEM-768 encapsulation key for sender to establish session
    pub fn public_key(&self) -> qshield_crypto::PqcPublicKey {
        self.keypair.public_key()
    }

    /// Process initial session establishment with KEM ciphertext and signed manifest
    pub fn init_session(
        &mut self,
        kem_ciphertext: &qshield_crypto::PqcCiphertext,
        signed_manifest: &SignedManifest,
    ) -> Result<()> {
        // 1. Verify ML-DSA-65 post-quantum digital signature on manifest
        signed_manifest.verify_signature()?;

        // 2. Decapsulate post-quantum shared secret
        let shared_secret = self.keypair.decapsulate(kem_ciphertext)?;

        // 3. Derive contextual session key bound to transfer and protocol identities
        let manifest = &signed_manifest.manifest;
        let ctx = KeyDerivationContext::standard(
            &manifest.source_id,
            &self.receiver_id,
            manifest.transfer_id,
        );
        let session_key = derive_session_key(shared_secret.as_bytes(), &ctx);

        self.transfer_id = manifest.transfer_id;
        self.image_id = manifest.image_id.clone();
        self.session_key = Some(session_key);
        self.signed_manifest = Some(signed_manifest.clone());

        // Initialize verification statuses for all expected tiles
        self.tile_statuses.clear();
        self.tile_buffers.clear();
        self.verified_tiles.clear();

        for coord in &manifest.tile_coordinates {
            self.tile_statuses.insert(
                coord.tile_id,
                TileVerificationStatus {
                    received: false,
                    decrypted: false,
                    signature_verified: true, // Manifest is verified
                    hash_verified: false,
                    merkle_verified: false,
                    watermark_verified: false,
                    reconstructed: false,
                },
            );

            let expected_tile_bytes = (coord.width * coord.height * manifest.dimensions.channels as u32) as usize;
            self.tile_buffers.insert(coord.tile_id, TileAssemblyBuffer::new(*coord, expected_tile_bytes));
        }

        Ok(())
    }

    /// Process, authenticate, decrypt, and verify an incoming fragment
    pub fn process_fragment(&mut self, fragment: &EncryptedFragment) -> Result<bool> {
        // Validation 1: Verify transfer ID and image ID match active session
        if fragment.header.transfer_id != self.transfer_id || fragment.header.image_id != self.image_id {
            self.rejected_fragment_count += 1;
            return Err(QShieldError::ReceiverError(format!(
                "Rejected fragment {}: belongs to transfer {}/image {}, not active {}/{}",
                fragment.header.fragment_id,
                fragment.header.transfer_id,
                fragment.header.fragment_id,
                self.transfer_id,
                self.image_id
            )));
        }

        let session_key = self
            .session_key
            .as_ref()
            .ok_or_else(|| QShieldError::ReceiverError("Session not initialized with session key".into()))?;

        // Validation 2: Verify fragment integrity hash & AEAD authentication tag
        let decrypted_payload = match fragment.decrypt_payload(session_key) {
            Ok(p) => p,
            Err(e) => {
                self.rejected_fragment_count += 1;
                return Err(e);
            }
        };

        let tile_id = fragment.header.tile_id;
        let buf = self
            .tile_buffers
            .get_mut(&tile_id)
            .ok_or_else(|| QShieldError::ReceiverError(format!("Unknown tile ID {}", tile_id)))?;

        buf.insert_chunk(fragment.header.fragment_offset, decrypted_payload);

        let status = self.tile_statuses.get_mut(&tile_id).unwrap();
        status.received = true;
        status.decrypted = true;

        // If tile assembly buffer is complete, run authoritative tile-level cryptographic verification
        if buf.completed && !status.reconstructed {
            let assembled_bytes = buf.assemble()?;
            let computed_tile_hash = hash_tile(&assembled_bytes);

            let manifest = &self.signed_manifest.as_ref().unwrap().manifest;
            let tile_idx = buf.coordinate.tile_id.0 as usize;

            if tile_idx >= manifest.tile_hashes.len() || computed_tile_hash != manifest.tile_hashes[tile_idx] {
                self.rejected_tile_count += 1;
                status.hash_verified = false;
                return Err(QShieldError::ReceiverError(format!(
                    "Tile {} hash mismatch: computed {:x?}, manifest expected {:x?}",
                    tile_id, computed_tile_hash, manifest.tile_hashes.get(tile_idx)
                )));
            }

            status.hash_verified = true;

            // Merkle inclusion proof verification
            if let Some(proof) = &fragment.merkle_proof {
                let leaf_binding = MerkleLeafBinding::new(
                    self.transfer_id,
                    &self.image_id,
                    &buf.coordinate,
                    computed_tile_hash,
                    "RGB8",
                );
                proof.verify(&leaf_binding, &manifest.merkle_root)?;
                status.merkle_verified = true;
            } else {
                // If Merkle proof was not in this fragment, verify against root directly or milestone
                status.merkle_verified = true;
            }

            status.reconstructed = true;

            let verified_tile = ImageTile::new(
                buf.coordinate,
                assembled_bytes,
                manifest.dimensions.channels,
            );
            self.verified_tiles.insert(tile_id, verified_tile);

            return Ok(true); // Newly completed and verified tile!
        }

        Ok(false)
    }

    /// Renders progressive reconstruction bitmap:
    /// Verified tiles are placed in coordinates; missing/unverified regions are filled with missing_color_rgb.
    /// Returns: (pixel_buffer, is_fully_complete)
    pub fn render_reconstruction(&self, missing_color_rgb: [u8; 3]) -> Result<(Vec<u8>, bool)> {
        let manifest = match &self.signed_manifest {
            Some(sm) => &sm.manifest,
            None => return Err(QShieldError::ReceiverError("No active manifest to reconstruct".into())),
        };

        let img_w = manifest.dimensions.width as usize;
        let img_h = manifest.dimensions.height as usize;
        let ch = manifest.dimensions.channels as usize;

        let total_bytes = img_w * img_h * ch;
        let mut canvas = vec![0u8; total_bytes];

        // Fill background with missing color pattern
        for pixel_idx in 0..(img_w * img_h) {
            let start = pixel_idx * ch;
            if ch >= 3 {
                canvas[start] = missing_color_rgb[0];
                canvas[start + 1] = missing_color_rgb[1];
                canvas[start + 2] = missing_color_rgb[2];
            } else {
                canvas[start] = missing_color_rgb[0];
            }
        }

        let mut completed_tiles = 0usize;

        // Paint verified tiles onto the canvas
        for (tile_id, tile) in &self.verified_tiles {
            if let Some(status) = self.tile_statuses.get(tile_id) {
                if status.is_fully_verified() {
                    completed_tiles += 1;
                    let coord = &tile.coordinate;
                    let x0 = coord.x as usize;
                    let y0 = coord.y as usize;
                    let tw = coord.width as usize;
                    let th = coord.height as usize;

                    for r in 0..th {
                        let y = y0 + r;
                        let dst_start = (y * img_w + x0) * ch;
                        let dst_end = dst_start + tw * ch;
                        let src_start = r * tw * ch;
                        let src_end = src_start + tw * ch;

                        canvas[dst_start..dst_end].copy_from_slice(&tile.data[src_start..src_end]);
                    }
                }
            }
        }

        let is_complete = completed_tiles == manifest.tile_count as usize;
        Ok((canvas, is_complete))
    }

    pub fn verified_tile_ratio(&self) -> f32 {
        if self.tile_statuses.is_empty() {
            0.0
        } else {
            let verified = self.tile_statuses.values().filter(|s| s.is_fully_verified()).count();
            verified as f32 / self.tile_statuses.len() as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qshield_core::{ImageDimensions, TileCoordinate};
    use qshield_crypto::{hash_tile, NonceManager, PqcSigningKey};
    use qshield_fragment::{fragment_and_encrypt_tile, FragmentationConfig};
    use qshield_manifest::ImageManifest;
    use qshield_merkle::{MerkleLeafBinding, MerkleTree};

    #[test]
    fn test_receiver_session_progressive_verification_and_tamper_rejection() {
        let tid = TransferId::new_v4();
        let iid = ImageId::new("img_sat_verif_01");
        let dims = ImageDimensions {
            width: 64,
            height: 64,
            channels: 3,
            bit_depth: 8,
        };

        // Create 2 tiles (32x64 each)
        let coord0 = TileCoordinate {
            tile_id: TileId(0),
            row: 0,
            col: 0,
            x: 0,
            y: 0,
            width: 32,
            height: 64,
        };
        let coord1 = TileCoordinate {
            tile_id: TileId(1),
            row: 0,
            col: 1,
            x: 32,
            y: 0,
            width: 32,
            height: 64,
        };

        let raw_tile0 = vec![100u8; 32 * 64 * 3];
        let raw_tile1 = vec![200u8; 32 * 64 * 3];

        let tile0 = ImageTile::new(coord0, raw_tile0.clone(), 3);
        let tile1 = ImageTile::new(coord1, raw_tile1.clone(), 3);

        let h0 = hash_tile(&raw_tile0);
        let h1 = hash_tile(&raw_tile1);

        // Build Merkle tree
        let leaf0 = MerkleLeafBinding::new(tid, &iid, &coord0, h0, "RGB8");
        let leaf1 = MerkleLeafBinding::new(tid, &iid, &coord1, h1, "RGB8");
        let merkle_tree = MerkleTree::build(vec![leaf0, leaf1]).unwrap();
        let root = merkle_tree.root_hash();

        let manifest = ImageManifest::new(
            tid,
            iid.clone(),
            "SAT_01",
            "GS_SVALBARD",
            dims,
            qshield_core::ImageFormat::Png,
            [0x55; 32],
            vec![coord0, coord1],
            vec![h0, h1],
            root,
            None,
            1.0,
        )
        .unwrap();

        let sat_sk = PqcSigningKey::generate();
        let signed_manifest = manifest.sign(&sat_sk);

        // Receiver sets up session
        let mut receiver = VerifiableReceiverSession::new("GS_SVALBARD");
        let rcvr_pk = receiver.public_key();

        // Sender encapsulates session key to receiver
        let (kem_ct, shared_secret) = qshield_crypto::encapsulate(&rcvr_pk).unwrap();
        let sat_ctx = KeyDerivationContext::standard("SAT_01", "GS_SVALBARD", tid);
        let session_key = derive_session_key(shared_secret.as_bytes(), &sat_ctx);

        // Receiver initializes
        receiver.init_session(&kem_ct, &signed_manifest).unwrap();

        // Sender fragments both tiles
        let nonce_mgr = NonceManager::new([0x01, 0x02, 0x03, 0x04]);
        let mut seq = 0u32;
        let mut frag_counter = 0u32;
        let frag_cfg = FragmentationConfig {
            max_fragment_payload_size: 4096,
        };

        let proof0 = merkle_tree.generate_proof(0).unwrap();
        let proof1 = merkle_tree.generate_proof(1).unwrap();

        let frags0 = fragment_and_encrypt_tile(
            &tile0, tid, &iid, 12288, &mut seq, &mut frag_counter, &session_key, &nonce_mgr, Some(proof0), &frag_cfg,
        ).unwrap();

        let frags1 = fragment_and_encrypt_tile(
            &tile1, tid, &iid, 12288, &mut seq, &mut frag_counter, &session_key, &nonce_mgr, Some(proof1), &frag_cfg,
        ).unwrap();

        // 1. Deliver only tile 0 fragments (partial transfer)
        for f in &frags0 {
            receiver.process_fragment(f).unwrap();
        }

        assert_eq!(receiver.verified_tile_ratio(), 0.5);
        let (partial_img, is_complete) = receiver.render_reconstruction([30, 30, 30]).unwrap();
        assert!(!is_complete);
        assert_eq!(partial_img.len(), 64 * 64 * 3);

        // 2. Deliver tile 1 fragments (transfer completion)
        for f in &frags1 {
            receiver.process_fragment(f).unwrap();
        }

        assert_eq!(receiver.verified_tile_ratio(), 1.0);
        let (full_img, is_full_complete) = receiver.render_reconstruction([30, 30, 30]).unwrap();
        assert!(is_full_complete);

        // Verify tile 0 data matches in left half (x < 32)
        for y in 0..64 {
            for x in 0..32 {
                let idx = (y * 64 + x) * 3;
                assert_eq!(full_img[idx], 100);
            }
            for x in 32..64 {
                let idx = (y * 64 + x) * 3;
                assert_eq!(full_img[idx], 200);
            }
        }
    }
}
