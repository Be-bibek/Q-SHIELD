//! Q-SHIELD Cryptographic Engine
//!
//! Standardized Post-Quantum and Authenticated Cryptography:
//! - ML-KEM-768 (NIST FIPS 203) for Post-Quantum Key Encapsulation
//! - ML-DSA-65 (NIST FIPS 204) for Post-Quantum Digital Signatures
//! - AES-256-GCM for Authenticated Symmetric Tile Encryption
//! - HKDF-SHA-256 for Cryptographically Bound Key Derivation
//! - SHA-256 with Domain Separation for Image and Fragment Integrity

pub mod aead;
pub mod dsa;
pub mod hash;
pub mod kdf;
pub mod kem;

// Re-export core cryptographic types
pub use aead::{decrypt_aead, encrypt_aead, NonceManager, NONCE_SIZE, TAG_SIZE};
pub use dsa::{PqcSignature, PqcSigningKey, PqcVerifyingKey};
pub use hash::{
    hash_merkle_leaf, hash_merkle_node, hash_tile, sha256_digest, sha256_hex,
    MANIFEST_HASH_PREFIX, MERKLE_LEAF_PREFIX, MERKLE_NODE_PREFIX, TILE_HASH_PREFIX,
};
pub use kdf::{derive_session_key, DerivedKey, KeyDerivationContext};
pub use kem::{encapsulate, PqcCiphertext, PqcKeyPair, PqcPublicKey, PqcSharedSecret};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use qshield_core::TransferId;

    #[test]
    fn test_full_pqc_session_key_and_aead_pipeline() {
        // 1. Recipient (Ground Station) generates ML-KEM-768 keypair
        let gs_pqc_keypair = PqcKeyPair::generate();
        let gs_public_key = gs_pqc_keypair.public_key();

        // 2. Sender (LEO Satellite) encapsulates shared secret to Ground Station
        let (kem_ciphertext, sat_shared_secret) = encapsulate(&gs_public_key).unwrap();

        // 3. Ground Station decapsulates to recover shared secret
        let gs_shared_secret = gs_pqc_keypair.decapsulate(&kem_ciphertext).unwrap();
        assert_eq!(sat_shared_secret.as_bytes(), gs_shared_secret.as_bytes());

        // 4. Both derive AES-256-GCM session key bound to transfer context
        let transfer_id = TransferId::new_v4();
        let sat_ctx = KeyDerivationContext::standard("SAT_LEO_SENTINEL", "GS_SVALBARD", transfer_id);
        let gs_ctx = KeyDerivationContext::standard("SAT_LEO_SENTINEL", "GS_SVALBARD", transfer_id);

        let sat_aes_key = derive_session_key(sat_shared_secret.as_bytes(), &sat_ctx);
        let gs_aes_key = derive_session_key(gs_shared_secret.as_bytes(), &gs_ctx);
        assert_eq!(sat_aes_key.as_bytes(), gs_aes_key.as_bytes());

        // 5. Satellite signs metadata with ML-DSA-65
        let sat_signing_key = PqcSigningKey::generate();
        let sat_verifying_key = sat_signing_key.verifying_key();
        let manifest_digest = sha256_digest(b"ImageManifest:2048x2048:16tiles");
        let manifest_sig = sat_signing_key.sign(&manifest_digest);
        assert!(sat_verifying_key.verify(&manifest_digest, &manifest_sig).is_ok());

        // 6. Satellite encrypts a tile with AEAD
        let nonce_mgr = NonceManager::new([0x01, 0x02, 0x03, 0x04]);
        let nonce = nonce_mgr.next_nonce().unwrap();
        let tile_raw_pixels = vec![128u8; 1024]; // simulated raw tile
        let aad = format!("TransferId:{}|TileId:0001", transfer_id).into_bytes();

        let encrypted_tile = encrypt_aead(sat_aes_key.as_bytes(), &nonce, &tile_raw_pixels, &aad).unwrap();

        // 7. Ground station decrypts tile
        let decrypted_tile = decrypt_aead(gs_aes_key.as_bytes(), &nonce, &encrypted_tile, &aad).unwrap();
        assert_eq!(tile_raw_pixels, decrypted_tile);
    }
}
