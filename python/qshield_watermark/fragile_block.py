"""
Q-SHIELD Fragile Block-Level Tamper Detection Watermarking.
Embeds block-level authentication features into least significant bits (LSBs).
Enables precise spatial localization of pixel modification, tile relocation,
tile replacement, and cropping.
"""

import hashlib
import numpy as np


class FragileBlockWatermarker:
    """Block-level fragile watermark for spatial tamper localization."""

    def __init__(self, block_size: int = 8, secret_seed: str = "QSHIELD-FRAGILE-SALT"):
        self.block_size = block_size
        self.secret_seed = secret_seed.encode("utf-8")

    def _block_hash(self, block_msb: np.ndarray, row_idx: int, col_idx: int) -> int:
        """Compute 8-bit authentication token binding block MSB content and spatial coordinate."""
        h = hashlib.sha256()
        h.update(self.secret_seed)
        h.update(row_idx.to_bytes(4, "big"))
        h.update(col_idx.to_bytes(4, "big"))
        h.update(block_msb.tobytes())
        digest = h.digest()
        # Return first byte as authentication parity token
        return digest[0]

    def embed(self, image: np.ndarray) -> np.ndarray:
        """
        Embeds fragile authentication watermark into image.
        Uses 7 Most Significant Bits (MSBs) to calculate block hash,
        and embeds token into the LSB of pixels in the block.
        """
        img = image.copy().astype(np.uint8)
        h, w = img.shape[:2]
        bs = self.block_size

        is_color = img.ndim == 3

        for r in range(0, h - bs + 1, bs):
            for c in range(0, w - bs + 1, bs):
                if is_color:
                    block = img[r : r + bs, c : c + bs, 0]  # Work on primary band
                else:
                    block = img[r : r + bs, c : c + bs]

                # Extract 7 MSBs (clear LSB)
                block_msb = block & 0xFE
                token = self._block_hash(block_msb, r // bs, c // bs)

                # Embed 8 bits of token into the first 8 pixels' LSB
                flat_block = block.flatten()
                for bit_pos in range(8):
                    bit = (token >> (7 - bit_pos)) & 1
                    flat_block[bit_pos] = (flat_block[bit_pos] & 0xFE) | bit

                if is_color:
                    img[r : r + bs, c : c + bs, 0] = flat_block.reshape((bs, bs))
                else:
                    img[r : r + bs, c : c + bs] = flat_block.reshape((bs, bs))

        return img

    def detect_tamper(self, test_image: np.ndarray) -> tuple[np.ndarray, float]:
        """
        Inspects test image for tampering.
        Returns:
            tamper_mask (2D boolean ndarray of shape (H, W), True = tampered)
            tamper_ratio (float, fraction of tampered blocks)
        """
        img = test_image.astype(np.uint8)
        h, w = img.shape[:2]
        bs = self.block_size

        tamper_mask = np.zeros((h, w), dtype=bool)
        total_blocks = 0
        tampered_blocks = 0

        is_color = img.ndim == 3

        for r in range(0, h - bs + 1, bs):
            for c in range(0, w - bs + 1, bs):
                total_blocks += 1
                if is_color:
                    block = img[r : r + bs, c : c + bs, 0]
                else:
                    block = img[r : r + bs, c : c + bs]

                block_msb = block & 0xFE
                expected_token = self._block_hash(block_msb, r // bs, c // bs)

                flat_block = block.flatten()
                extracted_token = 0
                for bit_pos in range(8):
                    bit = flat_block[bit_pos] & 1
                    extracted_token = (extracted_token << 1) | bit

                if extracted_token != expected_token:
                    tamper_mask[r : r + bs, c : c + bs] = True
                    tampered_blocks += 1

        tamper_ratio = float(tampered_blocks / total_blocks) if total_blocks > 0 else 0.0
        return tamper_mask, tamper_ratio
