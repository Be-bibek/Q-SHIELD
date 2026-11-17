# Watermark Validation & Robustness Analysis

**Document version:** 2.0  
**Date:** 2026-11-17  
**Raw data location:** `experiments/results/v2/watermark_robustness.json`, `experiments/results/v2/fragile_tamper.json`  
**Figures:** `experiments/figures/v2/fig06_watermark_jpeg_synthetic_satellite.png`, `experiments/figures/v2/fig06_watermark_jpeg_sentinel2_real.png`, `experiments/figures/v2/fig07_watermark_full_attack_nc.png`, `experiments/figures/v2/fig10_fragile_tamper_localization.png`

---

## 1. Executive Summary

Q-SHIELD implements a dual-layer watermarking system:
1. **Robust Layer:** DWT-DCT spread-spectrum watermark embedded into the LL subband of the red channel ($\alpha = 12.0$, Haar wavelet) for post-transmission provenance attribution and ownership verification.
2. **Fragile Layer:** $8 \times 8$ block-level parity authentication watermark for pixel-level tamper localization.

This document details the complete experimental validation across 23 distinct attack conditions on two test images (synthetic satellite terrain and real Sentinel-2A Level-1C imagery).

---

## 2. Invisibility Metrics (No Attacks)

| Image | Resolution | Base PSNR (dB) | Base SSIM | Visual Artifacts |
|---|---|---:|---:|---|
| **Synthetic Terrain** | $256 \times 256 \times 3$ | 51.71 dB | 0.9945 | Imperceptible |
| **Sentinel-2A Real Scene** | $256 \times 256 \times 3$ | 52.05 dB | 0.9943 | Imperceptible |

Both images exceed the standard threshold for imperceptibility in satellite remote sensing ($> 40\text{ dB}$, $\text{SSIM} > 0.98$).

---

## 3. Robust Watermark: Full 23-Attack Sweep

The authentication threshold is set to **$\text{NC} \ge 0.85$** (Normalized Correlation), corresponding to a Bit Error Rate ($\text{BER}$) threshold $\le 0.12$.

| Category | Attack Specification | Synthetic Image NC | Synthetic Authentic? | Real Sentinel-2A NC | Real Authentic? |
|---|---|---:|:---:|---:|:---:|
| **Compression** | JPEG Q95 | 1.000 | ✅ | 1.000 | ✅ |
| | JPEG Q85 | 1.000 | ✅ | 1.000 | ✅ |
| | JPEG Q75 | 0.843 | ❌ | 0.517 | ❌ |
| | JPEG Q65 | 0.611 | ❌ | 0.497 | ❌ |
| | JPEG Q50 | 0.472 | ❌ | 0.449 | ❌ |
| | JPEG Q35 | 0.490 | ❌ | 0.494 | ❌ |
| **Additive Noise**| Gaussian $\sigma=5$ | 1.000 | ✅ | 0.994 | ✅ |
| | Gaussian $\sigma=10$ | 0.884 | ✅ | 0.894 | ✅ |
| | Gaussian $\sigma=20$ | 0.732 | ❌ | 0.747 | ❌ |
| | Salt & Pepper 1% | 0.747 | ❌ | 0.739 | ❌ |
| | Salt & Pepper 3% | 0.566 | ❌ | 0.603 | ❌ |
| **Geometric** | Cropping 10% | 1.000 | ✅ | 1.000 | ✅ |
| | Cropping 20% | 0.981 | ✅ | 0.988 | ✅ |
| | Resizing 75% | 1.000 | ✅ | 1.000 | ✅ |
| | Resizing 50% | 0.994 | ✅ | 1.000 | ✅ |
| | Rotation 1.0° | 0.988 | ✅ | 0.988 | ✅ |
| | Rotation 2.0° | 0.982 | ✅ | 0.975 | ✅ |
| **Filtering** | Gaussian Blur $r=1.0$ | 1.000 | ✅ | 1.000 | ✅ |
| | Gaussian Blur $r=2.0$ | 1.000 | ✅ | 1.000 | ✅ |
| | Sharpening $\times 1.5$ | 1.000 | ✅ | 1.000 | ✅ |
| | Sharpening $\times 2.0$ | 1.000 | ✅ | 1.000 | ✅ |
| **Radiometric** | Contrast Adjust $\times 1.3$| 1.000 | ✅ | 1.000 | ✅ |
| **Conversion** | Format Conversion (TIFF)| 1.000 | ✅ | 1.000 | ✅ |

**Summary:** 16 out of 23 attacks passed the strict authentication threshold ($\text{NC} \ge 0.85$).

---

## 4. Algorithmic Limitation: The JPEG Compression Threshold

A critical finding in the experimental evaluation is the steep drop in Normalized Correlation between JPEG Quality 85 and Quality 75:
- **At $Q \ge 85$:** Watermark bit recovery is perfect ($\text{NC} = 1.000$, $\text{BER} = 0.000$).
- **At $Q = 75$:** $\text{NC}$ drops to $0.843$ (synthetic) and $0.517$ (real image), falling below the $0.85$ threshold.
- **At $Q \le 65$:** Watermark information is completely destroyed ($\text{NC} \approx 0.50$, equivalent to random guessing).

### Root Cause Analysis:
1. **DCT Quantization Step Size:** Standard JPEG luminance quantization tables scale inversely with quality factor:

$$q(u, v) = \text{round}\left(Q_{\text{base}}(u, v) \cdot \frac{50}{Q}\right) \quad (\text{for } Q < 50)$$

$$q(u, v) = \text{round}\left(Q_{\text{base}}(u, v) \cdot \frac{100 - Q}{50}\right) \quad (\text{for } Q \ge 50)$$

At $Q=75$, the multiplier is $0.5$, but for mid-to-high frequency DCT coefficients, the quantization step size exceeds the modulation amplitude $\alpha = 12.0$.
2. **Spatial Grid Misalignment:** The DWT LL subband has a $16 \times 16$ spatial footprint, whereas JPEG operates on local $8 \times 8$ pixel blocks. Block boundaries create boundary artifacts that disrupt the multi-scale DWT wavelets during inversion.
3. **Manuscript Transparency:** In the IEEE paper, this behavior must be documented explicitly as a **known boundary condition**, rather than attempting to conceal it. Q-SHIELD's primary integrity layer is cryptographic (ML-DSA-65 Merkle proofs); watermarking acts as a secondary persistent provenance layer when images are stored or viewed outside the cryptographic wrapper.

---

## 5. Fragile Watermark: Tamper Localization Results

The $8 \times 8$ block fragile watermark was evaluated against synthetic ground-truth tamper masks:

| Tamper Attack | Tampered Area | False Positive Rate | Precision | Recall | F1-Score | Intersection over Union (IoU) |
|---|---|---:|---:|---:|---:|---:|
| **No Tamper (Baseline)** | 0 pixels | **0.00000** | 1.000 | 1.000 | 1.000 | 1.000 |
| **Object Insertion** | $32 \times 32$ box | 0.00000 | 1.000 | 1.000 | **1.000** | **1.000** |
| **Copy-Move Forgery** | $24 \times 24$ box | 0.00000 | 0.632 | 0.837 | **0.720** | **0.562** |
| **Region Replacement** | $64 \times 64$ box | 0.00000 | 1.000 | 1.000 | **1.000** | **1.000** |

### Insights:
- **Zero False-Positive Rate:** When no tampering has occurred, the detector generates 0 false detections across all 1,024 blocks.
- **Perfect Geometric Localization:** Both insertion ($32 \times 32$) and replacement ($64 \times 64$) achieve perfect $\text{F1} = 1.000$ and $\text{IoU} = 1.000$, showing exact block boundary identification.
- **Copy-Move Limitation:** Copy-move achieves $\text{F1} = 0.720$ and $\text{IoU} = 0.562$. Because copy-move transfers existing image blocks from one part of the image to another, some target blocks happen to have matching parity bits by chance ($\sim 50\%$ coincidence rate per bit), requiring neighborhood morphological filtering to clean up isolated misses.
