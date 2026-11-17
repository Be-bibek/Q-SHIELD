"""
Q-SHIELD Image Watermark and Distortion Evaluation Metrics.
Computes PSNR, SSIM, Normalized Correlation (NC), Bit Error Rate (BER),
and Tamper Localization Accuracy.
"""

import numpy as np
from skimage.metrics import peak_signal_noise_ratio as skimage_psnr
from skimage.metrics import structural_similarity as skimage_ssim


def compute_psnr(image_true: np.ndarray, image_test: np.ndarray, data_range: float = 255.0) -> float:
    """Calculate Peak Signal-to-Noise Ratio (PSNR) in dB."""
    return float(skimage_psnr(image_true, image_test, data_range=data_range))


def compute_ssim(image_true: np.ndarray, image_test: np.ndarray) -> float:
    """Calculate Structural Similarity Index (SSIM)."""
    if image_true.ndim == 3 and image_true.shape[2] == 3:
        return float(skimage_ssim(image_true, image_test, channel_axis=2, data_range=255.0))
    return float(skimage_ssim(image_true, image_test, data_range=255.0))


def compute_normalized_correlation(orig_bits: np.ndarray, extracted_bits: np.ndarray) -> float:
    """
    Calculate Normalized Correlation (NC) between two binary watermark sequences.
    Value is in [-1.0, 1.0], where 1.0 indicates perfect extraction.
    """
    orig = np.asarray(orig_bits, dtype=np.float64)
    ext = np.asarray(extracted_bits, dtype=np.float64)

    min_len = min(len(orig), len(ext))
    if min_len == 0:
        return 0.0

    orig = orig[:min_len]
    ext = ext[:min_len]

    numerator = np.sum(orig * ext)
    denominator = np.sqrt(np.sum(orig ** 2) * np.sum(ext ** 2))

    if denominator == 0:
        return 0.0
    return float(numerator / denominator)


def compute_ber(orig_bits: np.ndarray, extracted_bits: np.ndarray) -> float:
    """Calculate Bit Error Rate (BER) between original and extracted bit arrays."""
    orig = np.asarray(orig_bits, dtype=np.int32)
    ext = np.asarray(extracted_bits, dtype=np.int32)

    min_len = min(len(orig), len(ext))
    if min_len == 0:
        return 1.0

    orig = orig[:min_len]
    ext = ext[:min_len]

    errors = np.sum(orig != ext)
    return float(errors / min_len)


def evaluate_tamper_localization(ground_truth_mask: np.ndarray, detected_mask: np.ndarray) -> dict:
    """
    Evaluate tamper localization performance:
    Precision, Recall, F1-Score, and False Positive Rate (FPR).
    """
    gt = ground_truth_mask.astype(bool).flatten()
    det = detected_mask.astype(bool).flatten()

    tp = np.sum(gt & det)
    fp = np.sum(~gt & det)
    fn = np.sum(gt & ~det)
    tn = np.sum(~gt & ~det)

    precision = float(tp / (tp + fp)) if (tp + fp) > 0 else 1.0
    recall = float(tp / (tp + fn)) if (tp + fn) > 0 else 1.0
    f1 = float(2 * precision * recall / (precision + recall)) if (precision + recall) > 0 else 0.0
    fpr = float(fp / (fp + tn)) if (fp + tn) > 0 else 0.0
    iou = float(tp / (tp + fp + fn)) if (tp + fp + fn) > 0 else 1.0
    accuracy = float((tp + tn) / (tp + tn + fp + fn)) if (tp + tn + fp + fn) > 0 else 1.0

    return {
        "true_positives": int(tp),
        "false_positives": int(fp),
        "true_negatives": int(tn),
        "false_negatives": int(fn),
        "precision": precision,
        "recall": recall,
        "f1_score": f1,
        "iou": iou,
        "accuracy": accuracy,
        "false_positive_rate": fpr,
    }
