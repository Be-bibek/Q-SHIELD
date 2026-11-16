//! Q-SHIELD Image-Aware Fragmentation and Authenticated Encryption Engine.
//!
//! Provides independently authenticated encrypted transmission units with AEAD (AES-256-GCM),
//! guaranteed nonce uniqueness, Authenticated Associated Data (AAD) contextual binding,
//! and out-of-order verifiable reconstruction.

use qshield_core::{
    FragmentId, FragmentState, ImageId, QShieldError, Result, TileId, TransferId,
};
pub use qshield_crypto::NONCE_SIZE;
use qshield_crypto::{
    decrypt_aead, encrypt_aead, sha256_digest, DerivedKey, NonceManager,
};
use qshield_image::ImageTile;
use qshield_merkle::MerkleInclusionProof;
use serde::{Deserialize, Serialize};

/// Header metadata for an independently authenticated transmission unit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FragmentHeader {
    pub transfer_id: TransferId,
    pub image_id: ImageId,
    pub fragment_id: FragmentId,
    pub tile_id: TileId,
    pub sequence_number: u32,
    pub fragment_offset: u32,
    pub fragment_length: u32,
    pub total_expected_image_size: u64,
    pub tile_data_total_len: u32,
    pub is_last_fragment_of_tile: bool,
    pub is_last_fragment_of_image: bool,
    pub nonce: [u8; NONCE_SIZE],
    pub importance: f32,
}

impl FragmentHeader {
    /// Construct canonical Authenticated Associated Data (AAD) byte string
    pub fn build_aad(&self) -> Vec<u8> {
        format!(
            "AAD:TID={}:IMG={}:FRAG={}:TILE={}:SEQ={}:OFF={}:LEN={}:TOT_IMG={}:TOT_TILE={}:NONCE={}:IMP={:.4}",
            self.transfer_id,
            self.image_id,
            self.fragment_id.0,
            self.tile_id.0,
            self.sequence_number,
            self.fragment_offset,
            self.fragment_length,
            self.total_expected_image_size,
            self.tile_data_total_len,
            hex::encode(self.nonce),
            self.importance
        )
        .into_bytes()
    }
}

/// Transmission unit carrying AEAD ciphertext, authentication tag, and cryptographic integrity record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncryptedFragment {
    pub header: FragmentHeader,
    pub ciphertext_and_tag: Vec<u8>,
    pub fragment_hash: [u8; 32],
    pub merkle_proof: Option<MerkleInclusionProof>,
    pub state: FragmentState,
}

impl EncryptedFragment {
    /// Compute fragment integrity hash: SHA-256(AAD || ciphertext_and_tag)
    pub fn compute_fragment_hash(header: &FragmentHeader, ciphertext_and_tag: &[u8]) -> [u8; 32] {
        let mut buffer = header.build_aad();
        buffer.extend_from_slice(ciphertext_and_tag);
        sha256_digest(&buffer)
    }

    /// Verify the fragment integrity hash before attempting decryption
    pub fn verify_integrity(&self) -> Result<()> {
        let expected_hash = Self::compute_fragment_hash(&self.header, &self.ciphertext_and_tag);
        if self.fragment_hash != expected_hash {
            return Err(QShieldError::AuthTagMismatch(format!(
                "Fragment {} hash mismatch: expected {:x?}, calculated {:x?}",
                self.header.fragment_id, self.fragment_hash, expected_hash
            )));
        }
        Ok(())
    }

    /// Decrypt the fragment payload with AES-256-GCM, verifying authentication tag and AAD
    pub fn decrypt_payload(&self, session_key: &DerivedKey) -> Result<Vec<u8>> {
        self.verify_integrity()?;
        let aad = self.header.build_aad();
        decrypt_aead(
            session_key.as_bytes(),
            &self.header.nonce,
            &self.ciphertext_and_tag,
            &aad,
        )
    }

    /// Total over-the-air size in bytes (header metadata + payload + tag + proof)
    pub fn wire_size(&self) -> usize {
        let proof_size = match &self.merkle_proof {
            Some(p) => p.proof_steps.len() * 33 + 8,
            None => 0,
        };
        // Estimated serialized wire format
        128 + self.ciphertext_and_tag.len() + 32 + proof_size
    }
}

/// Fragmentation engine configuration
#[derive(Debug, Clone, Copy)]
pub struct FragmentationConfig {
    pub max_fragment_payload_size: usize,
}

impl Default for FragmentationConfig {
    fn default() -> Self {
        // Standard satellite MTU-friendly fragment payload size (e.g. 64 KB or 16 KB)
        Self {
            max_fragment_payload_size: 64 * 1024,
        }
    }
}

/// Slices and encrypts an image tile into independently authenticated fragments
pub fn fragment_and_encrypt_tile(
    tile: &ImageTile,
    transfer_id: TransferId,
    image_id: &ImageId,
    total_image_bytes: u64,
    global_seq_start: &mut u32,
    fragment_id_counter: &mut u32,
    session_key: &DerivedKey,
    nonce_mgr: &NonceManager,
    merkle_proof: Option<MerkleInclusionProof>,
    config: &FragmentationConfig,
) -> Result<Vec<EncryptedFragment>> {
    let tile_bytes = &tile.data;
    let tile_len = tile_bytes.len();
    let max_payload = config.max_fragment_payload_size;

    let total_fragments = (tile_len + max_payload - 1) / max_payload;
    let mut fragments = Vec::with_capacity(total_fragments);

    for (frag_idx, chunk) in tile_bytes.chunks(max_payload).enumerate() {
        let is_last_frag = frag_idx == total_fragments - 1;
        let nonce = nonce_mgr.next_nonce()?;
        let offset = (frag_idx * max_payload) as u32;
        let frag_len = chunk.len() as u32;

        let frag_id = FragmentId(*fragment_id_counter);
        *fragment_id_counter += 1;

        let seq_num = *global_seq_start;
        *global_seq_start += 1;

        let header = FragmentHeader {
            transfer_id,
            image_id: image_id.clone(),
            fragment_id: frag_id,
            tile_id: tile.tile_id(),
            sequence_number: seq_num,
            fragment_offset: offset,
            fragment_length: frag_len,
            total_expected_image_size: total_image_bytes,
            tile_data_total_len: tile_len as u32,
            is_last_fragment_of_tile: is_last_frag,
            is_last_fragment_of_image: false, // Updated by caller for final fragment
            nonce,
            importance: tile.importance,
        };

        let aad = header.build_aad();
        let ciphertext_and_tag = encrypt_aead(session_key.as_bytes(), &nonce, chunk, &aad)?;
        let fragment_hash = EncryptedFragment::compute_fragment_hash(&header, &ciphertext_and_tag);

        fragments.push(EncryptedFragment {
            header,
            ciphertext_and_tag,
            fragment_hash,
            merkle_proof: if is_last_frag { merkle_proof.clone() } else { None },
            state: FragmentState::Encrypted,
        });
    }

    Ok(fragments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use qshield_core::TileCoordinate;

    #[test]
    fn test_tile_fragmentation_encryption_and_decryption_roundtrip() {
        let tid = TransferId::new_v4();
        let iid = ImageId::new("img_sat_01");
        let coord = TileCoordinate {
            tile_id: TileId(0),
            row: 0,
            col: 0,
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        };

        // 100x100 RGB tile = 30,000 bytes
        let mut raw_tile_data = vec![0u8; 30_000];
        for (i, b) in raw_tile_data.iter_mut().enumerate() {
            *b = (i % 256) as u8;
        }

        let tile = ImageTile::new(coord, raw_tile_data.clone(), 3);

        let key = DerivedKey::new([0x33; 32]);
        let nonce_mgr = NonceManager::new([0x01, 0x02, 0x03, 0x04]);

        let mut seq = 0u32;
        let mut frag_counter = 0u32;
        let config = FragmentationConfig {
            max_fragment_payload_size: 8192, // 8 KB chunks -> 4 fragments
        };

        let fragments = fragment_and_encrypt_tile(
            &tile,
            tid,
            &iid,
            30_000,
            &mut seq,
            &mut frag_counter,
            &key,
            &nonce_mgr,
            None,
            &config,
        )
        .unwrap();

        assert_eq!(fragments.len(), 4);
        assert_eq!(fragments[3].header.is_last_fragment_of_tile, true);

        // Decrypt fragments in arbitrary order (simulating out-of-order network arrival)
        let mut reassembled_tile = vec![0u8; 30_000];
        for frag in fragments.iter().rev() {
            let decrypted_chunk = frag.decrypt_payload(&key).expect("decryption failed");
            let off = frag.header.fragment_offset as usize;
            reassembled_tile[off..off + decrypted_chunk.len()].copy_from_slice(&decrypted_chunk);
        }

        assert_eq!(raw_tile_data, reassembled_tile);

        // Test tampering with ciphertext in one fragment
        let mut tampered_frag = fragments[0].clone();
        tampered_frag.ciphertext_and_tag[10] ^= 0xFF;
        assert!(tampered_frag.decrypt_payload(&key).is_err());
    }
}
