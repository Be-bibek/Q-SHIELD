"""
Q-SHIELD Watermark Robustness Attacks Simulation.
Implements realistic channel and transmission distortions:
- JPEG compression
- Spatial cropping
- Gaussian and salt-and-pepper noise
- Geometric scaling / downsampling
"""

import io
import numpy as np
from PIL import Image


def apply_jpeg_compression(image: np.ndarray, quality: int = 75) -> np.ndarray:
    """Simulate lossy JPEG compression at specified quality level."""
    pil_img = Image.fromarray(image)
    buf = io.BytesIO()
    pil_img.save(buf, format="JPEG", quality=quality)
    buf.seek(0)
    compressed = Image.open(buf)
    return np.array(compressed)


def apply_cropping(image: np.ndarray, crop_ratio: float = 0.15) -> np.ndarray:
    """Simulate spatial data loss / cropping by zeroing out an area."""
    out = image.copy()
    h, w = out.shape[:2]
    ch = int(h * crop_ratio)
    cw = int(w * crop_ratio)
    y0 = (h - ch) // 2
    x0 = (w - cw) // 2
    out[y0 : y0 + ch, x0 : x0 + cw] = 0
    return out


def apply_gaussian_noise(image: np.ndarray, mean: float = 0.0, sigma: float = 5.0) -> np.ndarray:
    """Add zero-mean Gaussian noise to simulate sensor / channel noise."""
    noise = np.random.normal(mean, sigma, image.shape)
    noisy = image.astype(np.float64) + noise
    return np.clip(np.round(noisy), 0, 255).astype(np.uint8)


def apply_resizing(image: np.ndarray, scale: float = 0.5) -> np.ndarray:
    """Downscale and upscale back to original dimensions."""
    pil_img = Image.fromarray(image)
    orig_size = pil_img.size
    down_size = (max(1, int(orig_size[0] * scale)), max(1, int(orig_size[1] * scale)))
    
    downscaled = pil_img.resize(down_size, Image.Resampling.BILINEAR)
    upscaled = downscaled.resize(orig_size, Image.Resampling.BILINEAR)
    return np.array(upscaled)
