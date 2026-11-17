# Satellite Communication Model Validation

**Document status:** RESEARCH PROTOTYPE — Not operationally validated  
**Author:** Q-SHIELD Research Team  
**Date:** 2026-11-17  
**Version:** 2.0

---

## 1. Orbital Model

### 1.1 SGP4 Implementation

Q-SHIELD uses the SGP4 (Simplified General Perturbations 4) model for orbit propagation, implemented in `python/qshield_orbit/orbit_propagator.py` using the `sgp4` Python library.

**SGP4 accuracy characteristics:**
- Position accuracy: ±1–3 km for TLE age < 2 days
- Velocity accuracy: ±1–2 m/s
- Valid propagation horizon: approximately 2 weeks from TLE epoch

**Reference satellites modelled:**

| Satellite | NORAD ID | Altitude (km) | Inclination (°) | Period (min) | Use in Q-SHIELD |
|---|---|---|---|---|---|
| Sentinel-2A | 40697 | 786 | 98.62 | 100.6 | Scenario A: Earth observation |
| Landsat-9 | 49260 | 705 | 98.20 | 98.9 | Scenario A: Alternative sensor |

**TLE provenance:** TLEs must be retrieved from `https://celestrak.org/SOCRATES/` or `https://space-track.org` at the time of experiment. The prototoype uses representative TLEs. Any paper using real contact-window predictions must document the TLE epoch date.

> [!WARNING]
> The contact windows in Figure 11 are **simulated representative values**, not live TLE-computed results. The SGP4 implementation in `python/qshield_orbit/orbit_propagator.py` is correct; the simulation uses the orbital period and typical pass geometry for Svalbard (78.2°N). Papers must run the actual propagator with a dated TLE to generate verifiable contact times.

### 1.2 Ground Station

| Parameter | Value |
|---|---|
| Station name | Svalbard Satellite Station (SvalSat) |
| Latitude | 78.23°N |
| Longitude | 15.39°E |
| Altitude | 458 m |
| Elevation mask | 5° (minimum viable contact) |
| Operator | Kongsberg Satellite Services |

SvalSat is used because its high latitude (78°N) gives extremely high contact frequency for sun-synchronous LEO satellites — up to 12 passes per day.

---

## 2. Link Budget Model

### 2.1 Formal Formulation

The link budget is implemented in `python/qshield_network/link_model.py`.

**Free-Space Path Loss (Friis equation):**

$$L_{FSPL}(d, f) = \left(\frac{4\pi d f}{c}\right)^2$$

In dB:
$$L_{FSPL,dB} = 20\log_{10}(d) + 20\log_{10}(f) + 20\log_{10}\left(\frac{4\pi}{c}\right)$$

**Received Power:**

$$P_r = P_t + G_t - L_{FSPL} - L_{atm} + G_r$$

(all terms in dBW or dBi)

**Link Margin:**

$$M = P_r - \left(N_0 + 10\log_{10}(B)\right) - E_b/N_0^{req}$$

**Shannon Capacity Upper Bound:**

$$C = B \log_2\left(1 + \frac{S}{N}\right)$$

### 2.2 Representative S-Band Parameters

| Parameter | Value | Notes |
|---|---|---|
| Carrier frequency | 2.025–2.110 GHz (S-band uplink) | CCSDS recommendation |
| Transmit power (satellite) | 2 W (3 dBW) | Typical small-sat |
| Transmit antenna gain | 3 dBi | Patch / monopole |
| Slant range at 5° elevation | ~2,750 km | Sentinel-2A altitude |
| FSPL at 5° elevation, 2 GHz | ~188 dB | Computed |
| Atmospheric attenuation | 0.5 dB | Clear sky, S-band |
| Ground receive gain (G/T) | 20 dB/K | VSAT-class dish |
| Required Eb/N0 (QPSK, BER=1e-6) | 10.5 dB | CCSDS standard |
| Effective data rate (simulation) | 50–150 Mbps | Function of elevation |

> [!NOTE]
> The 150 Mbps figure used in scheduling simulations represents the peak throughput at high elevation (zenith). The effective rate is modulated by `eff_rate = 150 × sin(elevation_deg × π/180)` in the simulation, which is a simplified approximation. A proper link budget should use the full Friis equation with measured G/T.

### 2.3 Contact Window Duration

For Sentinel-2A over Svalbard with 5° elevation mask:
- Minimum contact (grazing, 5°): ~4 min
- Typical contact (medium elevation): 7–10 min  
- Maximum contact (near-zenith): up to 13 min

Average contact duration: **~9.5 min** (570 s)

At 150 Mbps: **~1.07 GB** data transfer per zenith pass  
At 50 Mbps (5° elevation): **~22 MB** per marginal pass

---

## 3. Fragmentation and DTN Protocol

### 3.1 Fragment Size Selection

Fragment sizes {6 KB, 12 KB, 24 KB, 48 KB} in the simulation approximate realistic CCSDS transfer frame payloads:

| Fragment Size | Approx. Transfer Frames | Equivalent Image Tile |
|---|---|---|
| 6 KB | ~12 × CCSDS 512-byte frames | 64×64 pixel patch (8-bit RGB) |
| 12 KB | ~24 frames | 64×64 pixel (16-bit) |
| 24 KB | ~48 frames | 128×64 pixel (8-bit RGB) |
| 48 KB | ~96 frames | 128×128 pixel (8-bit RGB) |

### 3.2 BPSec RFC 9172 Boundary Statement

> [!IMPORTANT]
> Q-SHIELD implements **application-layer tile authentication**, not BPSec bundle-layer security blocks. The fragment security (AES-256-GCM encryption + GHASH tag + ML-DSA-65 Merkle signature) is applied at the Q-SHIELD transfer layer **above** the DTN Bundle Protocol. This is consistent with RFC 9172 §1.3 which explicitly permits application-defined security without BPSec. Fragments are not claimed to be BPSec-compliant security blocks.

---

## 4. Limitations and Future Work

| Limitation | Severity | Mitigation |
|---|---|---|
| Contact windows are simulated, not live-computed | Medium | Run `orbit_propagator.py` with dated TLE for real results |
| Link budget uses simplified elevation-rate formula | Low | Implement full Friis formula with antenna pattern |
| Multi-satellite constellation not modelled | Medium | Extend to Landsat-9 + Sentinel-2A joint coverage |
| Atmospheric scintillation not modelled | Low | Add Kolmogorov turbulence model for optical links |
| No Doppler compensation model | Low | Required for high-rate coherent modulations |

---

## 5. Validation Status

| Claim | Status | Evidence |
|---|---|---|
| SGP4 orbital model correct | ✅ VERIFIED | Standard sgp4 library, matches published ephemeris |
| Contact window duration realistic | ✅ SUPPORTED | 7–12 min matches literature (e.g., Wertz 2011) |
| 150 Mbps peak data rate plausible | ⚠️ SUPPORTED WITH LIMITATIONS | Achievable with high-gain ground antenna; cite specific hardware |
| Fragment sizes match real CCSDS frames | ✅ SUPPORTED | Consistent with CCSDS 133.0-B-2 |
| BPSec application-layer boundary correct | ✅ VERIFIED | Consistent with RFC 9172 §1.3 |
