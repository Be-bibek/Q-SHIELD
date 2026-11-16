//! Key Derivation using HKDF-SHA-256 with strong contextual binding.

use hkdf::Hkdf;
use qshield_core::{TransferId, CRYPTO_SUITE_ID, PROTOCOL_VERSION};
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// 256-bit symmetric key derived for AES-256-GCM with zeroization on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct DerivedKey {
    key: [u8; 32],
}

impl DerivedKey {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

/// Context for HKDF expansion, ensuring cryptographic domain separation
pub struct KeyDerivationContext<'a> {
    pub protocol_id: &'a str,
    pub protocol_version: &'a str,
    pub sender_id: &'a str,
    pub receiver_id: &'a str,
    pub transfer_id: TransferId,
    pub suite_id: &'a str,
}

impl<'a> KeyDerivationContext<'a> {
    pub fn standard(sender_id: &'a str, receiver_id: &'a str, transfer_id: TransferId) -> Self {
        Self {
            protocol_id: "QSHIELD",
            protocol_version: PROTOCOL_VERSION,
            sender_id,
            receiver_id,
            transfer_id,
            suite_id: CRYPTO_SUITE_ID,
        }
    }

    /// Construct canonical info string for HKDF-Expand
    pub fn to_info_bytes(&self) -> Vec<u8> {
        format!(
            "{}:{}:{}:{}:{}:{}",
            self.protocol_id,
            self.protocol_version,
            self.sender_id,
            self.receiver_id,
            self.transfer_id,
            self.suite_id
        )
        .into_bytes()
    }
}

/// Derive AES-256-GCM symmetric encryption key from post-quantum shared secret
pub fn derive_session_key(
    shared_secret: &[u8; 32],
    context: &KeyDerivationContext,
) -> DerivedKey {
    let salt = format!("QSHIELD-HKDF-SALT:{}", context.transfer_id).into_bytes();
    let hk = Hkdf::<Sha256>::new(Some(&salt), shared_secret);

    let info = context.to_info_bytes();
    let mut okm = [0u8; 32];
    hk.expand(&info, &mut okm)
        .expect("32 bytes is valid length for HKDF-SHA-256 expansion");

    DerivedKey::new(okm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hkdf_domain_binding() {
        let secret = [0x42u8; 32];
        let tid1 = TransferId::new_v4();
        let tid2 = TransferId::new_v4();

        let ctx1 = KeyDerivationContext::standard("SAT_LEO_01", "GS_SVALBARD", tid1);
        let ctx2 = KeyDerivationContext::standard("SAT_LEO_01", "GS_SVALBARD", tid2);
        let ctx3 = KeyDerivationContext::standard("SAT_LEO_02", "GS_SVALBARD", tid1);

        let k1 = derive_session_key(&secret, &ctx1);
        let k2 = derive_session_key(&secret, &ctx2);
        let k3 = derive_session_key(&secret, &ctx3);

        // Different transfer IDs produce different keys
        assert_ne!(k1.as_bytes(), k2.as_bytes());
        // Different sender IDs produce different keys
        assert_ne!(k1.as_bytes(), k3.as_bytes());
        // Same inputs produce identical keys
        let k1_replay = derive_session_key(&secret, &ctx1);
        assert_eq!(k1.as_bytes(), k1_replay.as_bytes());
    }
}
