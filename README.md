# Q-SHIELD: Post-Quantum Secure Satellite Image Transmission and Authentication System

[![Rust CI](https://img.shields.io/badge/Rust-1.94%2B-orange.svg)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/Python-3.11%2B-blue.svg)](https://www.python.org)
[![NIST PQC](https://img.shields.io/badge/NIST%20PQC-ML--KEM--768%20%7C%20ML--DSA--65-green.svg)](https://csrc.nist.gov/pubs/fips/203/final)
[![License](https://img.shields.io/badge/License-Apache%202.0%20%2F%20MIT-blue.svg)](LICENSE)

## System Overview

Q-SHIELD is a research-grade prototype architecture and software implementation designed for post-quantum secure satellite image transmission, fragmented authentication, and contact-aware scheduling over intermittent low Earth orbit (LEO) space-to-ground and satellite relay communications.

### Key Capabilities
- **Post-Quantum Cryptography (PQC)**:
  - NIST FIPS 203 **ML-KEM-768** (Module-Lattice Key Encapsulation Mechanism) for session shared secret establishment.
  - NIST FIPS 204 **ML-DSA-65** (Module-Lattice Digital Signature Standard) for canonical manifest signing and provenance verification.
  - **AES-256-GCM** authenticated symmetric encryption with guaranteed nonce uniqueness and Authenticated Associated Data (AAD) binding.
  - **HKDF-SHA-256** for context-bound key derivation (protocol identifier, version, sender/receiver identities, transfer ID, suite ID).
- **Domain-Separated Merkle Tree Integrity**:
  - Independent tile-level authentication enabling verifiable progressive reconstruction without requiring the entire image bundle.
  - Leaves bind image ID, tile coordinates, dimensions, encoding, and transfer ID.
- **Image Security & Watermarking**:
  - Dual-layer image protection: DWT-DCT transform-domain robust watermarking for provenance attribution paired with block-level fragile watermarking for tamper localization.
  - Authoritative cryptographic integrity verification (watermarks complement but never replace cryptographic guarantees).
- **Intermittent Contact & Orbital Dynamics**:
  - SGP4 / Skyfield orbit propagation calculating slant range, elevation angles, and contact windows between LEO satellites and ground stations.
  - Disruption-tolerant fragmented transmission engine supporting link interruptions and resumable state-machine transfers (`CREATED`, `ENCRYPTED`, `QUEUED`, `TRANSMITTED`, `RECEIVED`, `AUTHENTICATED`, `VERIFIED`, `RECONSTRUCTED`).
- **SAViCS Scheduling Engine**:
  - **Security-Aware Verifiable Image Contact Scheduling (SAViCS)** heuristic dynamically optimizing verified image coverage under tight contact window budgets, cryptographic overhead, and delivery deadlines against FIFO, EDF, and Priority baselines.
- **RFC 9171 / 9172 Security Separation**:
  - Explicit architectural boundary distinguishing application-level independently verifiable image tiles from DTN Bundle Protocol security blocks (BPSec).

## Architecture

```
                                  +---------------------------------------+
                                  |         Source Image Ingestion        |
                                  |      (GeoTIFF / TIFF / PNG / JPEG)    |
                                  +-------------------+-------------------+
                                                      |
                                                      v
                                  +---------------------------------------+
                                  | Image Preprocessing & Adaptive Tiling |
                                  +-------------------+-------------------+
                                                      |
                         +----------------------------+---------------------------+
                         |                                                        |
                         v                                                        v
           +-----------------------------+                          +-----------------------------+
           | Robust Provenance Watermark |                          |   Canonical Image Manifest  |
           |      (DWT-DCT Hybrid)       |                          |    (Dimensions, Tile Hashes)|
           +-----------------------------+                          +--------------+--------------+
                         |                                                        |
                         +----------------------------+---------------------------+
                                                      |
                                                      v
                                  +---------------------------------------+
                                  |    Merkle Tree Root & Leaf Inclusion  |
                                  |      (Domain-Separated Hashing)       |
                                  +-------------------+-------------------+
                                                      |
                                                      v
                                  +---------------------------------------+
                                  | ML-DSA-65 Manifest Digital Signature  |
                                  +-------------------+-------------------+
                                                      |
                                                      v
                                  +---------------------------------------+
                                  |   ML-KEM-768 Post-Quantum Key Est.   |
                                  |       HKDF-SHA-256 Session Key        |
                                  +-------------------+-------------------+
                                                      |
                                                      v
                                  +---------------------------------------+
                                  | Tile AEAD AES-256-GCM Fragment Engine |
                                  |     (Authenticated Associated Data)   |
                                  +-------------------+-------------------+
                                                      |
                                                      v
                                  +---------------------------------------+
                                  | SAViCS / Contact Window Scheduler     |
                                  |     (LEO Orbital Contact Plan)        |
                                  +-------------------+-------------------+
                                                      |
                                             [Intermittent LEO Link]
                                                      |
                                                      v
                                  +---------------------------------------+
                                  | Receiver Verification & Reconstruct   |
                                  | (Sig Verify, Proof Check, Tamper Map) |
                                  +---------------------------------------+
```

## Repository Structure

```
q-shield/
├── Cargo.toml                  # Rust workspace definition
├── pyproject.toml              # Python workspace & packaging
├── crates/
│   ├── qshield-core/           # Common domain types, errors, state tracking
│   ├── qshield-crypto/         # ML-KEM-768, ML-DSA-65, AES-256-GCM, HKDF, SHA-256
│   ├── qshield-image/          # Image parsing, tiling, coordinate grid, reassembly
│   ├── qshield-watermark/      # Watermark verification interfaces
│   ├── qshield-manifest/       # Canonical manifest generation and signature
│   ├── qshield-fragment/       # Image-aware AEAD fragmentation engine
│   ├── qshield-merkle/         # Domain-separated Merkle tree & inclusion proofs
│   ├── qshield-transfer/       # Disruption-tolerant transfer protocol & resumption
│   ├── qshield-scheduler/      # SAViCS & baseline scheduling algorithms
│   ├── qshield-receiver/       # Verifiable partial/complete image reconstructor
│   └── qshield-api/            # REST API service (Axum)
├── python/
│   ├── qshield_orbit/          # SGP4 LEO orbital propagator
│   ├── qshield_contacts/       # Slant range, elevation, contact planning
│   ├── qshield_imaging/        # Multispectral / satellite image preprocessing
│   ├── qshield_watermark/      # DWT-DCT & block-fragile watermarking
│   ├── qshield_network/        # Simulated RF channel dynamics & loss models
│   ├── qshield_experiments/    # Benchmark runners & statistical evaluators
│   └── tests/                  # Python test suite
├── apps/
│   └── dashboard/              # Next.js / TypeScript visual operations dashboard
├── configs/                    # Scenario A & Scenario B configuration profiles
├── datasets/                   # Sample imagery and manifest storage
├── experiments/                # Experimental scripts, baselines, and raw results
└── scripts/                    # Automation and reproduction scripts
```

## Security Notice

This codebase uses pure-Rust implementations of NIST FIPS 203 (ML-KEM) and NIST FIPS 204 (ML-DSA) from the RustCrypto ecosystem. These implementations are designed for scientific research, cybersecurity benchmarking, and prototyping; they are not independently formally audited or FIPS-140-3 validated.
