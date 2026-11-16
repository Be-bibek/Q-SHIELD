"""
Q-SHIELD Robust Provenance Watermarking Engine.
Hybrid DWT-DCT (Discrete Wavelet Transform + Discrete Cosine Transform)
transform-domain embedding and blind extraction.
"""

import numpy as np
import pywt
from scipy.fftpack import dct, idct


def _dct2(block: np.ndarray) -> np.ndarray:
    return dct(dct(block.T, norm='ortho').T, norm='ortho')


def _idct2(block: np.ndarray) -> np.ndarray:
    return idct(idct(block.T, norm='ortho').T, norm='ortho')


def string_to_bits(text: str) -> np.ndarray:
    """Convert text string into binary 1D array of 0s and 1s with a 16-bit sync word."""
    # 16-bit sync word: 10101011 11001101 (0xABCD)
    sync = [1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1]
    text_bytes = text.encode('utf-8')
    payload = []
    for b in text_bytes:
        for i in range(7, -1, -1):
            payload.append((b >> i) & 1)
    return np.array(sync + payload, dtype=np.uint8)


def bits_to_string(bits: np.ndarray) -> str:
    """Extract string from binary bits, verifying 16-bit sync word."""
    sync = [1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1]
    if len(bits) < 16:
        return ""
    
    # Check sync match or find closest offset
    best_offset = 0
    max_match = 0
    for offset in range(min(16, len(bits) - 16)):
        match = np.sum(bits[offset:offset + 16] == sync)
        if match > max_match:
            max_match = match
            best_offset = offset

    payload = bits[best_offset + 16:]
    byte_vals = []
    for i in range(0, len(payload) - 7, 8):
        byte_val = 0
        for bit_idx in range(8):
            byte_val = (byte_val << 1) | int(payload[i + bit_idx])
        byte_vals.append(byte_val)

    try:
        return bytes(byte_vals).decode('utf-8', errors='ignore')
    except Exception:
        return ""


class RobustDwtDctWatermarker:
    """Hybrid DWT-DCT Transform-Domain Watermarker for Provenance Embedding."""

    def __init__(self, alpha: float = 12.0, wavelet: str = 'haar'):
        self.alpha = float(alpha)
        self.wavelet = wavelet
        self.block_size = 8

    def embed(self, image: np.ndarray, provenance_id: str) -> tuple[np.ndarray, np.ndarray]:
        """
        Embeds provenance string into image.
        Returns:
            watermarked_image (np.ndarray), embedded_bits (np.ndarray)
        """
        img = image.astype(np.float64)
        is_rgb = img.ndim == 3 and img.shape[2] >= 3

        if is_rgb:
            # Embed into Luminance (Y) channel in YCbCr
            # Y = 0.299 R + 0.587 G + 0.114 B
            r, g, b = img[:, :, 0], img[:, :, 1], img[:, :, 2]
            y = 0.299 * r + 0.587 * g + 0.114 * b
            cb = -0.168736 * r - 0.331264 * g + 0.5 * b + 128.0
            cr = 0.5 * r - 0.418688 * g - 0.081312 * b + 128.0
            work_channel = y
        else:
            work_channel = img.copy()

        # 1. 2D Discrete Wavelet Transform
        coeffs2 = pywt.dwt2(work_channel, self.wavelet)
        ll, (lh, hl, hh) = coeffs2

        # 2. Block DCT on LL subband
        h_ll, w_ll = ll.shape
        bs = self.block_size
        n_blocks_h = h_ll // bs
        n_blocks_w = w_ll // bs
        max_bits = n_blocks_h * n_blocks_w

        bits = string_to_bits(provenance_id)
        if len(bits) > max_bits:
            raise ValueError(f"Watermark bits ({len(bits)}) exceed LL capacity ({max_bits})")

        # Repeat watermark bits across blocks for spatial redundancy and robustness
        full_bits = np.resize(bits, max_bits)

        bit_idx = 0
        mod_ll = ll.copy()

        # Embed into mid-frequency DCT coefficients (3, 4) and (4, 3)
        u1, v1 = 3, 4
        u2, v2 = 4, 3

        for i in range(n_blocks_h):
            for j in range(n_blocks_w):
                r_s, r_e = i * bs, (i + 1) * bs
                c_s, c_e = j * bs, (j + 1) * bs
                block = mod_ll[r_s:r_e, c_s:c_e]
                dct_block = _dct2(block)

                bit = full_bits[bit_idx]
                bit_idx += 1

                # Quantization / difference modulation
                diff = dct_block[u1, v1] - dct_block[u2, v2]
                if bit == 1:
                    if diff < self.alpha:
                        dct_block[u1, v1] += (self.alpha - diff) / 2.0
                        dct_block[u2, v2] -= (self.alpha - diff) / 2.0
                else:
                    if diff > -self.alpha:
                        dct_block[u1, v1] -= (self.alpha + diff) / 2.0
                        dct_block[u2, v2] += (self.alpha + diff) / 2.0

                mod_ll[r_s:r_e, c_s:c_e] = _idct2(dct_block)

        # 3. Inverse DWT
        reconstructed_channel = pywt.idwt2((mod_ll, (lh, hl, hh)), self.wavelet)
        # Ensure dimensions match original
        reconstructed_channel = reconstructed_channel[:work_channel.shape[0], :work_channel.shape[1]]

        if is_rgb:
            # Reconstruct RGB from modified Y, original Cb, Cr
            y_mod = reconstructed_channel
            cb_diff = cb - 128.0
            cr_diff = cr - 128.0

            r_out = y_mod + 1.402 * cr_diff
            g_out = y_mod - 0.344136 * cb_diff - 0.714136 * cr_diff
            b_out = y_mod + 1.772 * cb_diff

            out_img = np.stack([r_out, g_out, b_out], axis=2)
        else:
            out_img = reconstructed_channel

        out_img = np.clip(np.round(out_img), 0, 255).astype(np.uint8)
        return out_img, bits

    def extract(self, image: np.ndarray, expected_bit_len: int) -> tuple[np.ndarray, str]:
        """
        Blind extraction of watermark bits and provenance string from image.
        """
        img = image.astype(np.float64)
        if img.ndim == 3 and img.shape[2] >= 3:
            r, g, b = img[:, :, 0], img[:, :, 1], img[:, :, 2]
            work_channel = 0.299 * r + 0.587 * g + 0.114 * b
        else:
            work_channel = img.copy()

        coeffs2 = pywt.dwt2(work_channel, self.wavelet)
        ll, _ = coeffs2

        h_ll, w_ll = ll.shape
        bs = self.block_size
        n_blocks_h = h_ll // bs
        n_blocks_w = w_ll // bs
        max_blocks = n_blocks_h * n_blocks_w

        u1, v1 = 3, 4
        u2, v2 = 4, 3

        raw_bits = []
        for i in range(n_blocks_h):
            for j in range(n_blocks_w):
                r_s, r_e = i * bs, (i + 1) * bs
                c_s, c_e = j * bs, (j + 1) * bs
                block = ll[r_s:r_e, c_s:c_e]
                dct_block = _dct2(block)

                diff = dct_block[u1, v1] - dct_block[u2, v2]
                raw_bits.append(1 if diff > 0 else 0)

        # Average repeated watermark bits across redundant blocks
        extracted_bits = np.zeros(expected_bit_len, dtype=np.uint8)
        repetitions = max_blocks // expected_bit_len

        for k in range(expected_bit_len):
            votes = [raw_bits[k + rep * expected_bit_len] for rep in range(repetitions) if (k + rep * expected_bit_len) < len(raw_bits)]
            extracted_bits[k] = 1 if sum(votes) >= len(votes) / 2.0 else 0

        recovered_str = bits_to_string(extracted_bits)
        return extracted_bits, recovered_str
