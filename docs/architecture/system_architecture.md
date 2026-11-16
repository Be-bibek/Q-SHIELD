# Q-SHIELD System Architecture

## 1. System Objective
Q-SHIELD (Quantum-Secure High-integrity Image Encryption & LEO Downlink) provides an end-to-end framework for securing Earth observation imagery and tactical field images transmitted across intermittent satellite links.

## 2. Core Separation of Concerns
1. **Application-Layer Tile Security**:
   - Every image tile is independently authenticated, encrypted with AES-256-GCM, and bound into a domain-separated Merkle tree.
   - Individual tiles can be verified immediately upon receipt without waiting for the entire bundle.
2. **RFC 9171 / 9172 (BPSec) Boundary**:
   - RFC 9172 (Bundle Protocol Security) defines payload and block-level integrity/confidentiality for Delay-Tolerant Networking (DTN).
   - In accordance with RFC 9172 fragmentation rules, BPSec security blocks cannot be casually fragmented alongside payload blocks.
   - Q-SHIELD strictly isolates application-level image tile authentication from underlying DTN bundle security.

## 3. Operational Scenarios
- **Scenario A**: Earth Observation satellite (Sentinel-2 / Landsat) downlinking to polar ground station (Svalbard SG3).
- **Scenario B**: Ground edge reconnaissance device relaying via LEO satellite constellation to a secure defense gateway.
