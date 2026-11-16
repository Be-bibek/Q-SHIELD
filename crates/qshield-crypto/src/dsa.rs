//! ML-DSA-65 Post-Quantum Digital Signatures (NIST FIPS 204).

use hybrid_array::Array;
use ml_dsa::{
    signature::SignatureEncoding,
    Generate, KeyExport, Keypair, MlDsa65, Signer, SigningKey, Verifier, VerifyingKey,
};
use qshield_core::{QShieldError, Result};

use serde::{Deserialize, Serialize};

/// Post-quantum signing key for ML-DSA-65 with zeroization
pub struct PqcSigningKey {
    inner: SigningKey<MlDsa65>,
}

/// Serialized ML-DSA-65 Verifying (Public) Key (1952 bytes)
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PqcVerifyingKey {
    pub bytes: Vec<u8>,
}

/// Serialized ML-DSA-65 Signature (3309 bytes)
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PqcSignature {
    pub bytes: Vec<u8>,
}

impl PqcSigningKey {
    /// Generate a fresh ML-DSA-65 signing key using the system CSPRNG
    pub fn generate() -> Self {
        Self {
            inner: SigningKey::<MlDsa65>::generate(),
        }
    }

    /// Sign a canonical message buffer
    pub fn sign(&self, message: &[u8]) -> PqcSignature {
        let sig = self.inner.sign(message);
        PqcSignature {
            bytes: sig.to_bytes().to_vec(),
        }
    }

    /// Get the associated verifying (public) key
    pub fn verifying_key(&self) -> PqcVerifyingKey {
        let vk = self.inner.verifying_key();
        PqcVerifyingKey {
            bytes: vk.to_bytes().to_vec(),
        }
    }
}

impl PqcVerifyingKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let array = Array::try_from(bytes)
            .map_err(|_| QShieldError::CryptoError("Invalid ML-DSA-65 verifying key size".into()))?;
        let _ = VerifyingKey::<MlDsa65>::decode(&array);
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Verify a signature against a message
    pub fn verify(&self, message: &[u8], signature: &PqcSignature) -> Result<()> {
        let array = Array::try_from(self.bytes.as_slice())
            .map_err(|_| QShieldError::CryptoError("Invalid ML-DSA-65 verifying key size".into()))?;
        let vk = VerifyingKey::<MlDsa65>::decode(&array);

        let sig = ml_dsa::Signature::<MlDsa65>::try_from(signature.bytes.as_slice())
            .map_err(|_| QShieldError::SignatureVerificationFailed("Invalid signature encoding".into()))?;

        vk.verify(message, &sig)
            .map_err(|e| QShieldError::SignatureVerificationFailed(format!("Signature verification failed: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_dsa_65_sign_and_verify() {
        let sk = PqcSigningKey::generate();
        let vk = sk.verifying_key();

        let message = b"Q-SHIELD Satellite Canonical Image Manifest 2026";
        let signature = sk.sign(message);

        // Verification must succeed
        assert!(vk.verify(message, &signature).is_ok());

        // Verifying with tampered message must fail
        let mut tampered_message = message.to_vec();
        tampered_message[0] ^= 0x01;
        assert!(vk.verify(&tampered_message, &signature).is_err());

        // Verifying with tampered signature must fail
        let mut tampered_sig = signature.clone();
        tampered_sig.bytes[10] ^= 0xAA;
        assert!(vk.verify(message, &tampered_sig).is_err());
    }

    #[test]
    fn test_ml_dsa_65_key_sizes() {
        let sk = PqcSigningKey::generate();
        let vk = sk.verifying_key();
        let sig = sk.sign(b"Test");

        assert_eq!(vk.bytes.len(), 1952); // NIST FIPS 204 ML-DSA-65 public key size
        assert_eq!(sig.bytes.len(), 3309); // NIST FIPS 204 ML-DSA-65 signature size
    }
}
