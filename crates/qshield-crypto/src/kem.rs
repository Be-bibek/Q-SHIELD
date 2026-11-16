//! ML-KEM-768 Post-Quantum Key Encapsulation (NIST FIPS 203).

use hybrid_array::Array;
use ml_kem::{
    kem::{Decapsulate, Encapsulate, Kem},
    DecapsulationKey, EncapsulationKey, KeyExport, MlKem768,
};
use qshield_core::{QShieldError, Result};
use zeroize::{Zeroize, ZeroizeOnDrop};

use serde::{Deserialize, Serialize};

/// Post-quantum shared secret (32 bytes) with zeroize-on-drop protection
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct PqcSharedSecret {
    secret: [u8; 32],
}

impl PqcSharedSecret {
    pub fn new(secret: [u8; 32]) -> Self {
        Self { secret }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.secret
    }
}

/// Serialized ML-KEM-768 Public Encapsulation Key (1184 bytes)
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PqcPublicKey {
    pub bytes: Vec<u8>,
}

/// Serialized ML-KEM-768 Ciphertext (1088 bytes)
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PqcCiphertext {
    pub bytes: Vec<u8>,
}

/// Post-quantum KEM keypair
pub struct PqcKeyPair {
    decapsulation_key: DecapsulationKey<MlKem768>,
    encapsulation_key: EncapsulationKey<MlKem768>,
}

impl PqcKeyPair {
    /// Generate a fresh ML-KEM-768 keypair using the system CSPRNG
    pub fn generate() -> Self {
        let (dk, ek) = MlKem768::generate_keypair();
        Self {
            decapsulation_key: dk,
            encapsulation_key: ek,
        }
    }

    /// Export the public encapsulation key as bytes
    pub fn public_key(&self) -> PqcPublicKey {
        PqcPublicKey {
            bytes: self.encapsulation_key.to_bytes().to_vec(),
        }
    }

    /// Decapsulate a shared secret from ciphertext
    pub fn decapsulate(&self, ciphertext: &PqcCiphertext) -> Result<PqcSharedSecret> {
        let ct_array = Array::try_from(ciphertext.bytes.as_slice())
            .map_err(|_| QShieldError::CryptoError("Invalid ML-KEM-768 ciphertext length".into()))?;

        let shared_key = self.decapsulation_key.decapsulate(&ct_array);
        let mut secret = [0u8; 32];
        secret.copy_from_slice(shared_key.as_slice());
        Ok(PqcSharedSecret::new(secret))
    }
}

/// Encapsulate a shared secret to a recipient's public key
pub fn encapsulate(public_key: &PqcPublicKey) -> Result<(PqcCiphertext, PqcSharedSecret)> {
    let ek_array = Array::try_from(public_key.bytes.as_slice())
        .map_err(|_| QShieldError::CryptoError("Invalid ML-KEM-768 public key length".into()))?;

    let ek = EncapsulationKey::<MlKem768>::new(&ek_array)
        .map_err(|_| QShieldError::CryptoError("Failed to decode ML-KEM-768 public key".into()))?;

    let (ct, shared_key) = ek.encapsulate();
    let ciphertext = PqcCiphertext {
        bytes: ct.to_vec(),
    };

    let mut secret = [0u8; 32];
    secret.copy_from_slice(shared_key.as_slice());
    Ok((ciphertext, PqcSharedSecret::new(secret)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_kem_768_roundtrip() {
        // Alice generates keypair
        let alice_keypair = PqcKeyPair::generate();
        let alice_pk = alice_keypair.public_key();

        // Bob encapsulates to Alice's public key
        let (ciphertext, bob_secret) = encapsulate(&alice_pk).expect("encapsulation failed");

        // Alice decapsulates Bob's ciphertext
        let alice_secret = alice_keypair.decapsulate(&ciphertext).expect("decapsulation failed");

        // Verify shared secrets match exactly
        assert_eq!(bob_secret.as_bytes(), alice_secret.as_bytes());
        assert_eq!(alice_pk.bytes.len(), 1184); // Standard ML-KEM-768 public key size
        assert_eq!(ciphertext.bytes.len(), 1088); // Standard ML-KEM-768 ciphertext size
    }

    #[test]
    fn test_ml_kem_invalid_ciphertext_rejection() {
        let keypair = PqcKeyPair::generate();
        let mut corrupted_ct = PqcCiphertext {
            bytes: vec![0u8; 1088],
        };
        // Corrupted ciphertext produces differing shared secret (implicit rejection property of ML-KEM)
        let (valid_ct, shared_secret) = encapsulate(&keypair.public_key()).unwrap();
        corrupted_ct.bytes = valid_ct.bytes.clone();
        corrupted_ct.bytes[0] ^= 0xFF; // flip bits

        let decaps_corrupted = keypair.decapsulate(&corrupted_ct).unwrap();
        assert_ne!(shared_secret.as_bytes(), decaps_corrupted.as_bytes());
    }
}
