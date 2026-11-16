# Q-SHIELD Post-Quantum Cryptography Specification

## 1. Selected Algorithms & Standards
- **Key Encapsulation**: NIST FIPS 203 **ML-KEM-768** (Security Category 3).
  - Public Key: 1184 bytes
  - Ciphertext: 1088 bytes
  - Shared Secret: 32 bytes (256 bits)
- **Digital Signatures**: NIST FIPS 204 **ML-DSA-65** (Security Category 3).
  - Public Key: 1952 bytes
  - Signature: 3309 bytes
- **Symmetric Cipher**: **AES-256-GCM** (NIST SP 800-38D).
  - Key: 256 bits
  - Nonce: 96 bits with monotonic counter manager ensuring zero nonce-reuse
  - Tag: 128 bits
- **Key Derivation**: **HKDF-SHA-256** (RFC 5869)
  - Bound context: `QSHIELD:VERSION:SENDER:RECEIVER:TRANSFER_ID:SUITE_ID`
- **Integrity**: Domain-separated **SHA-256** (FIPS 180-4)
  - Leaf hash: `SHA-256(0x00 || leaf_data)`
  - Node hash: `SHA-256(0x01 || left || right)`

## 2. Formal Assurance Notice
Implemented via pure-Rust crates (`ml-kem` 0.3.4 and `ml-dsa` 0.1.2) from the RustCrypto project.
*Notice: These implementations are research-grade and not independently formally certified under FIPS 140-3.*
