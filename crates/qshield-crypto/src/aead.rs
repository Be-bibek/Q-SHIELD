//! AES-256-GCM Authenticated Encryption with Associated Data (AEAD) and Nonce Management.
//!
//! Cryptographic Standard: NIST SP 800-38D.
//! Authentication: Uses GHASH over GF(2^128) for the 128-bit authentication tag (never Poly1305).
//! Nonce Format: 96-bit (12 bytes) composed of [4-byte session salt || 8-byte monotonic counter].

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use qshield_core::{QShieldError, Result};
use rand_core::{OsRng, RngCore};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub const NONCE_SIZE: usize = 12; // 96 bits standard GCM nonce
pub const TAG_SIZE: usize = 16;   // 128 bits GHASH authentication tag

/// Thread-safe Nonce Manager that enforces monotonic nonce allocation and prevents reuse
#[derive(Clone, Default)]
pub struct NonceManager {
    used_nonces: Arc<Mutex<HashSet<[u8; NONCE_SIZE]>>>,
    counter: Arc<Mutex<u64>>,
    session_prefix: [u8; 4],
}

impl NonceManager {
    pub fn new(session_prefix: [u8; 4]) -> Self {
        Self {
            used_nonces: Arc::new(Mutex::new(HashSet::new())),
            counter: Arc::new(Mutex::new(0)),
            session_prefix,
        }
    }

    /// Instantiate with cryptographically secure random 4-byte session salt
    pub fn random_session() -> Self {
        let mut prefix = [0u8; 4];
        OsRng.fill_bytes(&mut prefix);
        Self::new(prefix)
    }

    /// Restore nonce manager from persistent storage with saved counter
    pub fn restore(session_prefix: [u8; 4], initial_counter: u64) -> Self {
        Self {
            used_nonces: Arc::new(Mutex::new(HashSet::new())),
            counter: Arc::new(Mutex::new(initial_counter)),
            session_prefix,
        }
    }

    /// Read current monotonic counter value for persistence checkpoints
    pub fn current_counter(&self) -> u64 {
        *self.counter.lock().unwrap()
    }

    /// Allocate next deterministic unique 96-bit nonce: [4 bytes prefix || 8 bytes monotonic counter]
    pub fn next_nonce(&self) -> Result<[u8; NONCE_SIZE]> {
        let mut count_guard = self.counter.lock().unwrap();
        let counter_val = *count_guard;
        *count_guard = counter_val.checked_add(1)
            .ok_or_else(|| QShieldError::CryptoError("Nonce counter exhausted (u64 overflow)".into()))?;

        let mut nonce = [0u8; NONCE_SIZE];
        nonce[..4].copy_from_slice(&self.session_prefix);
        nonce[4..].copy_from_slice(&counter_val.to_be_bytes());

        let mut used = self.used_nonces.lock().unwrap();
        if !used.insert(nonce) {
            return Err(QShieldError::CryptoError(format!(
                "CRITICAL: Nonce collision detected for nonce {:x?}",
                nonce
            )));
        }

        Ok(nonce)
    }

    /// Explicitly register an externally provided nonce and verify it has never been used
    pub fn register_nonce(&self, nonce: &[u8; NONCE_SIZE]) -> Result<()> {
        let mut used = self.used_nonces.lock().unwrap();
        if !used.insert(*nonce) {
            return Err(QShieldError::CryptoError(format!(
                "CRITICAL: Nonce reuse attempt detected for nonce {:x?}",
                nonce
            )));
        }
        Ok(())
    }
}

/// Encrypt a plaintext buffer with AES-256-GCM using authenticated associated data (AAD)
pub fn encrypt_aead(
    key: &[u8; 32],
    nonce: &[u8; NONCE_SIZE],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| QShieldError::CryptoError(format!("Failed to initialize AES-256-GCM cipher: {e}")))?;

    let gcm_nonce = Nonce::from(*nonce);
    let payload = Payload {
        msg: plaintext,
        aad,
    };

    let ciphertext_and_tag = cipher
        .encrypt(&gcm_nonce, payload)
        .map_err(|e| QShieldError::CryptoError(format!("AES-GCM encryption failed: {e}")))?;

    Ok(ciphertext_and_tag)
}

/// Decrypt a ciphertext buffer with AES-256-GCM, verifying authentication tag and AAD
pub fn decrypt_aead(
    key: &[u8; 32],
    nonce: &[u8; NONCE_SIZE],
    ciphertext_and_tag: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| QShieldError::CryptoError(format!("Failed to initialize AES-256-GCM cipher: {e}")))?;

    let gcm_nonce = Nonce::from(*nonce);
    let payload = Payload {
        msg: ciphertext_and_tag,
        aad,
    };

    let plaintext = cipher
        .decrypt(&gcm_nonce, payload)
        .map_err(|e| QShieldError::AuthTagMismatch(format!("AES-GCM authentication tag mismatch / corrupted ciphertext: {e}")))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_256_gcm_roundtrip() {
        let key = [0x7A; 32];
        let nonce_mgr = NonceManager::new([0x01, 0x02, 0x03, 0x04]);
        let nonce = nonce_mgr.next_nonce().unwrap();

        let plaintext = b"High-Resolution Multi-Spectral Satellite Tile Band 4";
        let aad = b"TransferId:12345|TileId:0042|Coords:(0,0)";

        let ciphertext = encrypt_aead(&key, &nonce, plaintext, aad).expect("encryption failed");
        let decrypted = decrypt_aead(&key, &nonce, &ciphertext, aad).expect("decryption failed");

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_aes_gcm_auth_tag_rejection_on_ciphertext_tampering() {
        let key = [0x55; 32];
        let nonce = [0x11; 12];
        let plaintext = b"Confidential Earth Observation Tile Data";
        let aad = b"AuthenticatedHeaderData";

        let mut ciphertext = encrypt_aead(&key, &nonce, plaintext, aad).unwrap();
        // Tamper with one byte in ciphertext
        ciphertext[5] ^= 0x01;

        let result = decrypt_aead(&key, &nonce, &ciphertext, aad);
        assert!(result.is_err());
    }

    #[test]
    fn test_aes_gcm_aad_mismatch_rejection() {
        let key = [0x55; 32];
        let nonce = [0x22; 12];
        let plaintext = b"Tile Data Payload";
        let aad_valid = b"TransferId:100";
        let aad_tampered = b"TransferId:200";

        let ciphertext = encrypt_aead(&key, &nonce, plaintext, aad_valid).unwrap();
        let result = decrypt_aead(&key, &nonce, &ciphertext, aad_tampered);
        assert!(result.is_err());
    }

    #[test]
    fn test_nonce_manager_prevents_duplicate() {
        let nonce_mgr = NonceManager::new([0xDE, 0xAD, 0xBE, 0xEF]);
        let n1 = nonce_mgr.next_nonce().unwrap();
        let n2 = nonce_mgr.next_nonce().unwrap();

        assert_ne!(n1, n2);

        // Attempting to register n1 again must fail
        let reuse_res = nonce_mgr.register_nonce(&n1);
        assert!(reuse_res.is_err());
    }
}
