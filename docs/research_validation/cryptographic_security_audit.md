# Q-SHIELD Cryptographic Security Audit

**Document Version**: 2.0  
**Audit Date**: 2026-11-17  
**Scope**: `crates/qshield-crypto`, `crates/qshield-manifest`, `crates/qshield-fragment`, `crates/qshield-receiver`  
**Standard References**: NIST FIPS 203 (ML-KEM), NIST FIPS 204 (ML-DSA), NIST SP 800-38D (GCM), RFC 5869 (HKDF)

---

## 1. Cryptographic Primitive Verification

| Primitive | Implementation / Crate | Standard | Security Level | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **ML-KEM-768** | `ml-kem` 0.3.4 (RustCrypto) | NIST FIPS 203 | Category 3 (AES-192 equivalent quantum hardness) | Ephemeral Session Key Encapsulation |
| **ML-DSA-65** | `ml-dsa` 0.1.2 (RustCrypto) | NIST FIPS 204 | Category 3 (128-bit quantum security against forgery) | Image Manifest Digital Signature |
| **AES-256-GCM** | `aes-gcm` 0.11.1 (RustCrypto) | NIST SP 800-38D | 256-bit classical symmetric security | Authenticated Encryption with Associated Data (AEAD) |
| **HKDF-SHA-256** | `hkdf` 0.13.0 + `sha2` 0.11.1 | RFC 5869 | 256-bit PRK expansion | Session Key Derivation with Context Binding |
| **SHA-256** | `sha2` 0.11.1 | FIPS 180-4 | 256-bit collision resistance | Merkle Tree & Fragment Integrity Hash |

> [!NOTE]
> **Implementation Assurance Disclaimer**: The RustCrypto implementations of ML-KEM and ML-DSA are actively maintained pure-Rust implementations of NIST FIPS 203 and FIPS 204. However, they are research-grade implementations and have not undergone independent FIPS 140-3 laboratory cryptographic module validation. All scientific claims must explicitly state this boundary.

---

## 2. Security Mechanisms & Audit Findings

### 2.1 AEAD Authentication Function (GHASH vs. Poly1305)
*   **Audit Finding**: A previous report mistakenly referred to Poly1305 in connection with AES-256-GCM.
*   **Correction**: AES-GCM strictly uses **GHASH** (universal hashing over the binary Galois field $\text{GF}(2^{128})$ with reduction polynomial $f(x) = x^{128} + x^7 + x^2 + x + 1$) to generate the 16-byte (128-bit) authentication tag. Poly1305 is an entirely different MAC used with ChaCha20 over the prime field $2^{130}-5$.
*   **Status**: **RESOLVED**. All code comments, crate docs, and reports now strictly specify GHASH.

### 2.2 AES-GCM Nonce Management and Lifetime Bound
*   **Standard Requirement (NIST SP 800-38D Section 8)**: Nonce reuse under the same AES key completely compromises GCM authentication integrity, allowing GHASH key recovery and message forgery. Under any single key, no 96-bit nonce may be invoked more than once.
*   **Q-SHIELD Implementation**:
    *   **Structure**: 96 bits ($12\text{ bytes}$) = `[4-byte Session Salt || 8-byte Atomic Monotonic Counter]`.
    *   **Ephemeral Key Life Cycle**: ML-KEM-768 establishes a fresh ephemeral session key per transfer session. An ephemeral key never encrypts more than $2^{32}$ fragments, remaining orders of magnitude below NIST SP 800-38D limit of $2^{64}$ invocations.
    *   **Restart Persistence**: `NonceManager::restore(prefix, counter)` allows saving and restoring monotonic counters across system restarts. `NonceManager::random_session()` initializes a cryptographically secure OS RNG salt for fresh sessions.
*   **Status**: **VERIFIED**. Nonce collisions within a key lifetime are prevented both deterministically and probabilistically.

### 2.3 Replay Protection Architecture
*   **Problem Statement**: While Authenticated Associated Data (AAD) binds headers (TransferId, TileId, FragIdx, Nonce), AAD alone only validates *authenticity*—it cannot prevent an active adversary from re-injecting an authentic fragment that was captured earlier in the session.
*   **Q-SHIELD Implementation**:
    *   Added `ReplayProtectionTracker` to `VerifiableReceiverSession`.
    *   The receiver records every incoming `(TransferId, FragmentId)` and `(TransferId, SequenceNumber)`.
    *   Any duplicate fragment arrival triggers an immediate `QShieldError::ReplayAttackDetected` error, logs the incident, and rejects the payload before tile buffering.
    *   Verified via automated integration test `test_replay_attack_detection`.
*   **Status**: **VERIFIED**.

### 2.4 Associated Data (AAD) Binding Coverage
*   **Composition**:
    $$\text{AAD} = \text{format!}(\text{"AAD:TID=\{\}:IMG=\{\}:FRAG=\{\}:TILE=\{\}:SEQ=\{\}:OFF=\{\}:LEN=\{\}:TOT\_IMG=\{\}:TOT\_TILE=\{\}:NONCE=\{\}:IMP=\{:.4\}"})$$
*   **Security Guarantee**: Any modification by an attacker to the transfer ID, image ID, tile ID, sequence offset, length, or tile importance causes an immediate GHASH authentication tag mismatch on the receiver, aborting decryption prior to memory allocation.
*   **Status**: **VERIFIED**.

### 2.5 Digital Signature & Merkle Canonicalization
*   **Tree Structure**:
    *   Leaf Node: $\text{SHA-256}(0\text{x}00 \parallel \text{TransferId} \parallel \text{ImageId} \parallel \text{TileCoordinate} \parallel \text{TileHash} \parallel \text{"RGB8"})$
    *   Internal Node: $\text{SHA-256}(0\text{x}01 \parallel \text{LeftChild} \parallel \text{RightChild})$
*   **Manifest Signing**:
    *   The canonical JSON manifest includes dimensions, coordinates, tile hashes, and the Merkle root.
    *   ML-DSA-65 digitally signs the SHA-256 digest of this canonical manifest.
    *   Tile verification: The receiver first validates the ML-DSA-65 signature on the root. Then, each received tile is independently authenticated against the root via an $O(\log_2 M)$ Merkle inclusion proof.
*   **Status**: **VERIFIED**.
