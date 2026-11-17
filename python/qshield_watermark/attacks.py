"""
Q-SHIELD Watermark Robustness Attacks Simulation.
Implements realistic channel and transmission distortions:
- JPEG compression
- Spatial cropping
- Gaussian and salt-and-pepper noise
- Geometric scaling / downsampling
- Rotation
- Gaussian blur and sharpening
- Contrast modification
- Format conversion
- Ground-truth synthetic tampering (box insertion, copy-move)
"""

import io
import numpy as np
from PIL import Image, ImageEnhance, ImageFilter


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


def apply_salt_and_pepper_noise(image: np.ndarray, amount: float = 0.02) -> np.ndarray:
    """Add impulsive salt-and-pepper noise."""
    out = image.copy()
    p = amount / 2.0
    rand = np.random.random(image.shape[:2])
    # Salt
    out[rand < p] = 255
    # Pepper
    out[(rand >= p) & (rand < amount)] = 0
    return out


def apply_resizing(image: np.ndarray, scale: float = 0.5) -> np.ndarray:
    """Downscale and upscale back to original dimensions."""
    pil_img = Image.fromarray(image)
    orig_size = pil_img.size
    down_size = (max(1, int(orig_size[0] * scale)), max(1, int(orig_size[1] * scale)))
    
    downscaled = pil_img.resize(down_size, Image.Resampling.BILINEAR)
    upscaled = downscaled.resize(orig_size, Image.Resampling.BILINEAR)
    return np.array(upscaled)


def apply_rotation(image: np.ndarray, angle_deg: float = 2.0) -> np.ndarray:
    """Rotate image by slight angle and rotate back to simulate gimbal alignment error."""
    pil_img = Image.fromarray(image)
    rot = pil_img.rotate(angle_deg, resample=Image.Resampling.BILINEAR, expand=False)
    rot_back = rot.rotate(-angle_deg, resample=Image.Resampling.BILINEAR, expand=False)
    return np.array(rot_back)


def apply_gaussian_blur(image: np.ndarray, radius: float = 1.0) -> np.ndarray:
    """Simulate optical defocus or atmospheric blur."""
    pil_img = Image.fromarray(image)
    blurred = pil_img.filter(ImageFilter.GaussianBlur(radius=radius))
    return np.array(blurred)


def apply_sharpening(image: np.ndarray, factor: float = 1.5) -> np.ndarray:
    """Apply high-frequency unsharp masking."""
    pil_img = Image.fromarray(image)
    enhancer = ImageEnhance.Sharpness(pil_img)
    sharpened = enhancer.enhance(factor)
    return np.array(sharpened)


def apply_contrast_adjustment(image: np.ndarray, factor: float = 1.2) -> np.ndarray:
    """Adjust image contrast."""
    pil_img = Image.fromarray(image)
    enhancer = ImageEnhance.Contrast(pil_img)
    adjusted = enhancer.enhance(factor)
    return np.array(adjusted)


def apply_format_conversion(image: np.ndarray, format_name: str = "TIFF") -> np.ndarray:
    """Simulate lossless format conversion (e.g. to TIFF or PNG)."""
    pil_img = Image.fromarray(image)
    buf = io.BytesIO()
    pil_img.save(buf, format=format_name)
    buf.seek(0)
    converted = Image.open(buf)
    return np.array(converted)


def generate_synthetic_tamper_insertion(
    image: np.ndarray, box: tuple[int, int, int, int], fill_color: int = 250
) -> tuple[np.ndarray, np.ndarray]:
    """
    Inserts synthetic region into image and returns (tampered_image, ground_truth_mask).
    box: (y0, x0, height, width)
    """
    out = image.copy()
    h, w = image.shape[:2]
    mask = np.zeros((h, w), dtype=bool)

    y0, x0, bh, bw = box
    y1 = min(h, y0 + bh)
    x1 = min(w, x0 + bw)

    out[y0:y1, x0:x1] = fill_color
    mask[y0:y1, x0:x1] = True

    return out, mask


def generate_synthetic_tamper_copy_move(
    image: np.ndarray, src_box: tuple[int, int, int, int], dst_box: tuple[int, int, int, int]
) -> tuple[np.ndarray, np.ndarray]:
    """
    Simulates malicious copy-move forgery and returns (tampered_image, ground_truth_mask).
    """
    out = image.copy()
    h, w = image.shape[:2]
    mask = np.zeros((h, w), dtype=bool)

    sy0, sx0, sh, sw = src_box
    dy0, dx0, _, _ = dst_box

    patch = image[sy0 : sy0 + sh, sx0 : sx0 + sw].copy()
    out[dy0 : dy0 + sh, dx0 : dx0 + sw] = patch
    mask[dy0 : dy0 + sh, dx0 : dx0 + sw] = True

    return out, mask

