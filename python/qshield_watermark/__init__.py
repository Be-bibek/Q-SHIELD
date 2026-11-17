"""
Q-SHIELD Image Watermarking Package:
Robust DWT-DCT Provenance Embedding and Fragile Block-Level Tamper Localization.
"""

from .metrics import (
    compute_psnr,
    compute_ssim,
    compute_normalized_correlation,
    compute_ber,
    evaluate_tamper_localization,
)
from .robust_dwtdct import RobustDwtDctWatermarker, string_to_bits, bits_to_string
from .fragile_block import FragileBlockWatermarker
from .attacks import (
    apply_jpeg_compression,
    apply_cropping,
    apply_gaussian_noise,
    apply_salt_and_pepper_noise,
    apply_resizing,
    apply_rotation,
    apply_gaussian_blur,
    apply_sharpening,
    apply_contrast_adjustment,
    apply_format_conversion,
    generate_synthetic_tamper_insertion,
    generate_synthetic_tamper_copy_move,
)

__all__ = [
    "compute_psnr",
    "compute_ssim",
    "compute_normalized_correlation",
    "compute_ber",
    "evaluate_tamper_localization",
    "RobustDwtDctWatermarker",
    "FragileBlockWatermarker",
    "string_to_bits",
    "bits_to_string",
    "apply_jpeg_compression",
    "apply_cropping",
    "apply_gaussian_noise",
    "apply_salt_and_pepper_noise",
    "apply_resizing",
    "apply_rotation",
    "apply_gaussian_blur",
    "apply_sharpening",
    "apply_contrast_adjustment",
    "apply_format_conversion",
    "generate_synthetic_tamper_insertion",
    "generate_synthetic_tamper_copy_move",
]
