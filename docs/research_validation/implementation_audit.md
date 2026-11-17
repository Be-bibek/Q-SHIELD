# Q-SHIELD Research Implementation Audit

**Audit Date**: 2026-11-17  
**Auditor**: Antigravity Principal Research Software Engineer & Cryptographic Auditor  
**Workspace**: `D:\flutter_main\Bibek\Q-SHIELD`  
**Baseline Git Commit**: `d8610675326b34a6d08d7ea741c440711feebdc8` (Tag: `v1.0-baseline`)

---

## 1. Executive Summary

This audit assesses the scientific rigor, algorithmic correctness, implementation completeness, and experimental methodology of the Q-SHIELD codebase prior to academic publication. The findings are categorized into **PASS**, **PARTIAL**, **FAIL**, and **NOT VERIFIED**.

| Subsystem / Module | Audit Finding | Key Defect / Observation | Required Remediation |
| :--- | :--- | :--- | :--- |
| **`qshield-crypto`** | **PARTIAL** | AEAD correctly uses AES-256-GCM, but documentation referenced Poly1305 rather than GHASH. Nonce manager uses memory-only state; nonces across restarts need explicit binding/persisting. Replay protection is missing active rejection in the receiver. | Correct GHASH terminology. Add persistent replay tracking with duplicate rejection. Add session-bound nonce validation. |
| **`qshield-manifest`** | **PASS** | Canonical JSON determinism, SHA-256 manifest digest, ML-DSA-65 digital signature and verification are implemented and tested. | None for core logic. Add statistical latency benchmarks. |
| **`qshield-fragment`** | **PARTIAL** | AAD binding is cryptographically sound, but wire overhead was estimated as a flat 128 bytes rather than computing exact serialized byte fields ($O_{\text{header}} + O_{\text{nonce}} + O_{\text{tag}} + O_{\text{proof}}$). | Implement exact wire-format accounting and benchmark overhead across 4 KB, 16 KB, 64 KB, and 256 KB fragments. |
| **`qshield-merkle`** | **PASS** | NIST domain separation (`0x00` leaf, `0x01` parent) implemented. Correct binary tree computation and inclusion proof verification. | Document relationship to RFC 6962 leaf/node prefix semantics. |
| **`qshield-scheduler`** | **PARTIAL** | SAViCS algorithm in Rust implements multi-attribute weights, but the experimental Python script evaluated *only* a single metric ($\sum \text{importance}$), in which a greedy algorithm (`ImagePriority`) naturally achieved a higher single-metric sum. The synthetic simulation also lacked packet loss, deadlines were trivial, and tile dependencies were omitted in the Python loop. | Redesign multi-objective benchmark with 7 metrics: delivered importance, verified coverage, tile completion rate, deadline success, contact utilization, goodput, and retransmission cost across 30+ seeds. |
| **`qshield-receiver`** | **PARTIAL** | Receiver verifies AEAD and Merkle proofs and reassembles 2D tiles, but duplicate fragments are ignored rather than actively detected and rejected via a persistent replay cache. | Add `ReplayProtectionTracker` with windowed sequence numbers and duplicate fragment rejection. |
| **`qshield_watermark`** | **PARTIAL** | Robust DWT-DCT and fragile 8x8 block LSB parity are functional. However, the reported jump from $NC = 0.518$ at JPEG Q75 to $NC = 1.000$ at Q85 was caused by spatial JPEG 8x8 DCT grid misalignment with LL subband 16x16 downsampled blocks. PSNR discrepancy (41.2 dB vs 52.0 dB) was caused by testing a smooth synthetic wave vs real satellite imagery. | Expand attacks to Gaussian, S&P noise, blur, sharpen, contrast, crop, resize, rotate. Evaluate fragile watermarking against ground-truth masks with Precision/Recall/F1/IoU. |
| **`qshield_orbit` & `qshield_contacts`** | **PASS** | High-fidelity SGP4 and Skyfield propagators for Sentinel-2A and Landsat-9 with Arctic ground station contact windows. | Document exact TLE sources, epochs, and retrieval dates. |
| **`qshield_network`** | **PARTIAL** | FSPL and atmospheric loss modeled, but link budget was not formally documented into carrier frequency, antenna gain, receiver $G/T$, and Shannon/DVB-S2 spectral efficiency. | Add documented link budget formulation connecting RF link parameters to achievable contact data rates. |
| **Datasets** | **PARTIAL** | Real Sentinel-2A and tactical images exist in `datasets/`, but lacked formal provenance manifests (sensor, acquisition date, bounding coordinates, spectral bands, checksums). | Add `datasets/manifest.json` with full metadata and SHA-256 checksums. |

---

## 2. Detailed Subsystem Analysis

### 2.1 Cryptographic Subsystem (`qshield-crypto`, `qshield-manifest`, `qshield-fragment`)
*   **Primitives Used**:
    *   `ml-kem` crate (0.3.4): NIST FIPS 203 ML-KEM-768.
    *   `ml-dsa` crate (0.1.2): NIST FIPS 204 ML-DSA-65.
    *   `aes-gcm` crate (0.11.1): AES-256-GCM.
    *   `hkdf` crate (0.13.0) + `sha2` (0.11.1): HKDF-SHA-256.
*   **Defect 1: AEAD Terminology**: The technical summary referenced "Poly1305" in connection with AES-256-GCM. AES-GCM strictly uses the Galois/Counter Mode universal hash function **GHASH** over GF($2^{128}$), whereas Poly1305 is used with ChaCha20. All documentation must strictly specify GHASH.
*   **Defect 2: Nonce Management & Process Restarts**: `NonceManager` generates a 96-bit nonce composed of a 4-byte session prefix and an 8-byte atomic monotonic counter. While thread-safe in-memory, if an identical key is reused after a process restart with the same prefix, counter collisions could occur. In Q-SHIELD, each transfer session uses an ephemeral key derived from ML-KEM-768, which bounds the risk, but the nonce manager must enforce session-specific salt binding or persistent state.
*   **Defect 3: Replay Protection**: While AAD binds `TransferId`, `TileId`, `FragIdx`, and `Nonce`, AAD alone does not prevent an adversary from replaying an authentic fragment within a session. The receiver lacked a duplicate fragment cache and sequence number sliding window.

### 2.2 SAViCS Scheduling Subsystem (`qshield-scheduler`)
*   **Root Cause of ImagePriority (5.62) vs. SAViCS (5.27)**:
    1.  In `scripts/run_benchmarks.py`, candidate fragments were evaluated in a knapsack of 120 KB capacity.
    2.  `ImagePriority` sorted fragments solely by descending `importance`.
    3.  `SAViCS` sorted fragments by a multi-attribute utility:
        $$U_i = 0.35 \cdot \text{imp}_i + 0.25 \cdot \text{urgency}_i + 0.20 \cdot \text{eff}_i + 0.20 \cdot \text{retrans}_i$$
    4.  However, the evaluation loop in `run_benchmarks.py` **only accumulated importance** ($\sum \text{imp}_i$).
    5.  Furthermore, the synthetic benchmark script did not induce packet loss, all deadlines were trivially met, and no incomplete tile penalties were applied!
    6.  As a mathematical necessity, sorting greedily by $X$ will always yield a higher or equal sum of $X$ than sorting by a weighted sum of $X, Y, Z$ when $Y$ and $Z$ are unconstrained.
*   **Remediation**:
    *   Model multi-objective trade-offs where SAViCS was designed to operate: link errors requiring retransmissions, hard contact deadlines, and tile completion milestones (partial tiles have zero reconstruction utility).
    *   Evaluate all 7 research metrics across at least 30 independent seeds.

### 2.3 Watermarking Subsystem (`qshield_watermark`)
*   **Root Cause of the JPEG Q75 -> Q85 Jump**:
    *   Embedding was performed on the $LL$ approximation subband of a 1-level 2D-DWT using 8x8 DCT blocks.
    *   An 8x8 block in the $LL$ subband corresponds to a $16 \times 16$ spatial region.
    *   Standard JPEG compression performs 8x8 block DCT directly in the spatial domain.
    *   At $Q \ge 85$, JPEG quantization steps are very small ($<8$), leaving the inter-coefficient differences preserved.
    *   At $Q \le 75$, JPEG quantization steps jump to $16\text{--}24$, causing aliasing across the misaligned $16 \times 16$ spatial footprint of the wavelet coefficients, which completely inverts the difference sign for majority-vote blocks, dropping correlation to the random chance baseline ($\approx 0.50$).
*   **PSNR Discrepancy (41.2 dB vs. 52.0 dB)**:
    *   $52.0\text{ dB}$ was measured on a synthetic smooth sine wave image where low-frequency variance was minimal.
    *   $41.2\text{ dB}$ was measured on an authentic Sentinel-2A multispectral scene with high texture and edge density.
    *   Both results are valid, but they must be explicitly contextualized by image source characteristics.

---

## 3. Action Plan for Phase 2–11

1.  **Phase 2**: Implement rigorous statistical cryptographic microbenchmarks with warm-up, $\ge 100$ iterations, full hardware telemetry, and AES-256-GCM payload sweeps (1 KB to 10 MB).
2.  **Phase 3**: Correct GHASH terminology, implement receiver `ReplayProtectionTracker`, and verify nonce uniqueness guarantees.
3.  **Phase 4**: Implement comprehensive multi-metric scheduler simulation with packet loss, retransmissions, deadlines, tile completion constraints, 30+ seeds, and ablation studies.
4.  **Phase 5**: Extend watermarking attack evaluation (JPEG, noise, blur, sharpen, contrast, crop, resize) and evaluate fragile watermarking against ground-truth masks.
5.  **Phase 6**: Document satellite link budget and orbital TLE metadata.
6.  **Phase 7**: Calculate exact wire-format overhead for 4 KB, 16 KB, 64 KB, 256 KB fragments.
7.  **Phase 8**: Document dataset provenance manifests with SHA-256 hashes.
8.  **Phase 9**: Generate reproducible, versioned experiment outputs (`experiments/results/v2/`).
9.  **Phase 10**: Render 13 publication-grade 300 DPI figures with error bars and clear labels.
10. **Phase 11**: Author the final research readiness report.
