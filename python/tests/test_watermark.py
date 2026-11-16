"""
Unit and Integration Tests for Q-SHIELD Watermarking Subsystem.
"""

import numpy as np
import pytest
from qshield_watermark import (
    RobustDwtDctWatermarker,
    FragileBlockWatermarker,
    compute_psnr,
    compute_ssim,
    compute_normalized_correlation,
    compute_ber,
    evaluate_tamper_localization,
    apply_jpeg_compression,
    apply_cropping,
)


@pytest.fixture
def synthetic_satellite_image():
    """Generates a 256x256 RGB synthetic earth-observation test tile."""
    np.random.seed(42)
    x = np.linspace(0, 10, 256)
    y = np.linspace(0, 10, 256)
    xx, yy = np.meshgrid(x, y)
    
    # Terrain texture simulation
    band_r = (np.sin(xx) * np.cos(yy) * 60 + 120).astype(np.uint8)
    band_g = (np.cos(xx * 0.5) * 50 + 100).astype(np.uint8)
    band_b = (np.sin(yy * 0.5) * 40 + 80).astype(np.uint8)
    
    return np.stack([band_r, band_g, band_b], axis=2)


def test_robust_watermark_embedding_and_extraction(synthetic_satellite_image):
    watermarker = RobustDwtDctWatermarker(alpha=14.0)
    provenance_id = "SAT01-PROV-2026"

    # Embed provenance
    watermarked, orig_bits = watermarker.embed(synthetic_satellite_image, provenance_id)

    # 1. Quality check (imperceptibility)
    psnr = compute_psnr(synthetic_satellite_image, watermarked)
    ssim = compute_ssim(synthetic_satellite_image, watermarked)

    assert psnr > 35.0, f"PSNR too low: {psnr:.2f} dB"
    assert ssim > 0.95, f"SSIM too low: {ssim:.4f}"

    # 2. Extraction check
    extracted_bits, recovered_str = watermarker.extract(watermarked, len(orig_bits))
    nc = compute_normalized_correlation(orig_bits, extracted_bits)
    ber = compute_ber(orig_bits, extracted_bits)

    assert nc >= 0.98, f"Normalized correlation too low: {nc:.4f}"
    assert ber <= 0.02, f"Bit error rate too high: {ber:.4f}"
    assert provenance_id in recovered_str or recovered_str.startswith("SAT01")


def test_robust_watermark_under_jpeg_compression(synthetic_satellite_image):
    watermarker = RobustDwtDctWatermarker(alpha=16.0)
    provenance_id = "SAT_LEO_DL"

    watermarked, orig_bits = watermarker.embed(synthetic_satellite_image, provenance_id)

    # Apply JPEG compression Q=80
    compressed = apply_jpeg_compression(watermarked, quality=80)

    extracted_bits, _ = watermarker.extract(compressed, len(orig_bits))
    nc = compute_normalized_correlation(orig_bits, extracted_bits)

    assert nc > 0.85, f"Watermark lost under JPEG Q80: NC={nc:.4f}"


def test_fragile_watermark_tamper_localization(synthetic_satellite_image):
    watermarker = FragileBlockWatermarker(block_size=8)

    # Embed fragile watermark
    watermarked = watermarker.embed(synthetic_satellite_image)

    # Verify zero false positives on authentic image
    mask_clean, tamper_ratio_clean = watermarker.detect_tamper(watermarked)
    assert tamper_ratio_clean == 0.0
    assert not np.any(mask_clean)

    # Introduce synthetic tamper in a 32x32 spatial region (4x4 blocks)
    tampered = watermarked.copy()
    tampered[64:96, 64:96, :] = 255  # Saturated white patch

    # Detect tamper
    detected_mask, tamper_ratio = watermarker.detect_tamper(tampered)
    assert tamper_ratio > 0.0

    # Ground truth mask
    gt_mask = np.zeros_like(detected_mask)
    gt_mask[64:96, 64:96] = True

    metrics = evaluate_tamper_localization(gt_mask, detected_mask)
    assert metrics["precision"] > 0.90
    assert metrics["recall"] > 0.90
    assert metrics["f1_score"] > 0.90
    assert metrics["false_positive_rate"] < 0.01
