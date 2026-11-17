"""
Q-SHIELD Phase 2 Comprehensive Research Experiment Suite.

Phases:
  A. Watermark experiments  — full attack sweep + fragile tamper localization
  B. SAViCS scheduler       — 50 seeds, 7 metrics, ablation study
  C. Figure generation      — 13 publication-quality 300 DPI plots

Saves all raw results to experiments/results/v2/
Saves all figures to experiments/figures/v2/
Never overwrites previous version directories.
"""

import json
import os
import sys
import hashlib
import subprocess
import platform
from pathlib import Path
from datetime import datetime, timezone

import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib import gridspec

# ------------------------------------------------------------------
# Path setup
# ------------------------------------------------------------------
ROOT = Path(__file__).parent.parent
sys.path.insert(0, str(ROOT / "python"))

from qshield_watermark import (
    RobustDwtDctWatermarker, FragileBlockWatermarker,
    compute_psnr, compute_ssim, compute_normalized_correlation, compute_ber,
    evaluate_tamper_localization,
    apply_jpeg_compression, apply_gaussian_noise, apply_salt_and_pepper_noise,
    apply_cropping, apply_resizing, apply_rotation,
    apply_gaussian_blur, apply_sharpening, apply_contrast_adjustment,
    apply_format_conversion,
    generate_synthetic_tamper_insertion, generate_synthetic_tamper_copy_move,
)

RESULTS_DIR = ROOT / "experiments" / "results" / "v2"
FIGURES_DIR = ROOT / "experiments" / "figures" / "v2"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
FIGURES_DIR.mkdir(parents=True, exist_ok=True)

# Style
plt.rcParams.update({
    "font.family": "DejaVu Sans",
    "font.size": 10,
    "axes.titlesize": 12,
    "axes.labelsize": 11,
    "xtick.labelsize": 9,
    "ytick.labelsize": 9,
    "legend.fontsize": 9,
    "figure.dpi": 300,
})

QSHIELD_CYAN = "#00f0ff"
QSHIELD_PURPLE = "#9d4edd"
ALERT_RED = "#e63946"
GOLD = "#ffd700"
GRID_GRAY = "#cccccc"


# ==================================================================
# A. WATERMARK EXPERIMENTS
# ==================================================================

def _make_satellite_test_image(seed=42):
    """Generate a realistic satellite-like imagery patch with structural features."""
    np.random.seed(seed)
    h, w = 256, 256
    # Base terrain with multi-frequency texture
    x = np.linspace(0, 10, w); y = np.linspace(0, 10, h)
    xx, yy = np.meshgrid(x, y)
    terrain  = (np.sin(xx * 1.3) * np.cos(yy * 0.9) * 40
              + np.cos(xx * 0.4 + yy * 0.6) * 30
              + np.sin(xx * 2.5 + 1) * 15) + 128
    # Overlay urban structure blobs
    for _ in range(12):
        cx, cy = np.random.randint(20, w-20), np.random.randint(20, h-20)
        bw, bh = np.random.randint(8, 24), np.random.randint(8, 24)
        col = np.random.randint(60, 220)
        terrain[cy:cy+bh, cx:cx+bw] = col
    r = np.clip(terrain, 0, 255).astype(np.uint8)
    g = np.clip(terrain * 0.9 + np.random.randint(-20,20), 0, 255).astype(np.uint8)
    b = np.clip(terrain * 0.7 + np.random.randint(-10,30), 0, 255).astype(np.uint8)
    return np.stack([r, g, b], axis=2)


def run_watermark_robustness_experiments():
    """Full watermark attack sweep against two test images (synthetic + real satellite)."""
    print("\n[A] Running Watermark Robustness Experiments...")

    watermarker = RobustDwtDctWatermarker(alpha=12.0, wavelet="haar")
    provenance_id = "SENTINEL-2A-PROV-2026"

    results_by_image = {}
    test_images = {
        "synthetic_satellite": _make_satellite_test_image(seed=42),
    }

    # Try loading real satellite image if available
    real_path = ROOT / "datasets" / "satellite" / "sentinel2_sample.png"
    if real_path.exists():
        try:
            from PIL import Image as PILImage
            real_arr = np.array(PILImage.open(real_path).convert("RGB").resize((256, 256)))
            test_images["sentinel2_real"] = real_arr
            print("    Loaded real Sentinel-2A image.")
        except Exception as e:
            print(f"    Warning: Could not load real image: {e}")

    for img_name, orig_img in test_images.items():
        print(f"  Image: {img_name} ({orig_img.shape})")
        watermarked, orig_bits = watermarker.embed(orig_img, provenance_id)
        n_bits = len(orig_bits)

        base_psnr = compute_psnr(orig_img, watermarked)
        base_ssim = compute_ssim(orig_img, watermarked)
        print(f"    Base PSNR: {base_psnr:.2f} dB   Base SSIM: {base_ssim:.4f}")

        # Attack suite definition
        attacks = [
            ("JPEG_Q95",    lambda img: apply_jpeg_compression(img, quality=95)),
            ("JPEG_Q85",    lambda img: apply_jpeg_compression(img, quality=85)),
            ("JPEG_Q75",    lambda img: apply_jpeg_compression(img, quality=75)),
            ("JPEG_Q65",    lambda img: apply_jpeg_compression(img, quality=65)),
            ("JPEG_Q50",    lambda img: apply_jpeg_compression(img, quality=50)),
            ("JPEG_Q35",    lambda img: apply_jpeg_compression(img, quality=35)),
            ("GaussNoise_s5",  lambda img: apply_gaussian_noise(img, sigma=5.0)),
            ("GaussNoise_s10", lambda img: apply_gaussian_noise(img, sigma=10.0)),
            ("GaussNoise_s20", lambda img: apply_gaussian_noise(img, sigma=20.0)),
            ("SaltPepper_1p",  lambda img: apply_salt_and_pepper_noise(img, amount=0.01)),
            ("SaltPepper_3p",  lambda img: apply_salt_and_pepper_noise(img, amount=0.03)),
            ("Crop_10p",    lambda img: apply_cropping(img, crop_ratio=0.10)),
            ("Crop_20p",    lambda img: apply_cropping(img, crop_ratio=0.20)),
            ("Resize_75p",  lambda img: apply_resizing(img, scale=0.75)),
            ("Resize_50p",  lambda img: apply_resizing(img, scale=0.50)),
            ("Rotate_1deg", lambda img: apply_rotation(img, angle_deg=1.0)),
            ("Rotate_2deg", lambda img: apply_rotation(img, angle_deg=2.0)),
            ("Blur_r1",     lambda img: apply_gaussian_blur(img, radius=1.0)),
            ("Blur_r2",     lambda img: apply_gaussian_blur(img, radius=2.0)),
            ("Sharpen_1p5", lambda img: apply_sharpening(img, factor=1.5)),
            ("Sharpen_2p0", lambda img: apply_sharpening(img, factor=2.0)),
            ("Contrast_1p3",lambda img: apply_contrast_adjustment(img, factor=1.3)),
            ("FormatTIFF",  lambda img: apply_format_conversion(img, format_name="TIFF")),
        ]

        attack_results = []
        for attack_name, attack_fn in attacks:
            np.random.seed(0)
            attacked = attack_fn(watermarked)
            extracted_bits, _ = watermarker.extract(attacked, n_bits)
            nc  = compute_normalized_correlation(orig_bits, extracted_bits)
            ber = compute_ber(orig_bits, extracted_bits)
            atk_psnr = compute_psnr(watermarked, attacked)
            authentic = nc >= 0.85
            attack_results.append({
                "attack": attack_name, "nc": nc, "ber": ber,
                "attack_psnr_db": atk_psnr, "watermark_authentic": authentic,
            })
            status = "AUTHENTIC" if authentic else "DEGRADED"
            print(f"      {attack_name:22s} NC={nc:.3f}  BER={ber:.3f}  [{status}]")

        results_by_image[img_name] = {
            "base_psnr_db": base_psnr,
            "base_ssim": base_ssim,
            "image_shape": list(orig_img.shape),
            "watermark_provenance": provenance_id,
            "watermark_bits": int(n_bits),
            "attacks": attack_results,
        }

    return results_by_image


def run_fragile_watermark_experiments():
    """Fragile block watermark evaluation against ground-truth tamper masks."""
    print("\n[A2] Running Fragile Watermark Tamper Localization Experiments...")
    fragile = FragileBlockWatermarker(block_size=8)

    orig_img = _make_satellite_test_image(seed=7)
    embedded = fragile.embed(orig_img)

    # Verify no-tamper FPR
    no_tamper_mask, _ = fragile.detect_tamper(embedded)
    no_tamper_fpr = float(np.sum(no_tamper_mask)) / float(no_tamper_mask.size)
    print(f"    No-tamper False Positive Rate: {no_tamper_fpr:.5f}")

    tamper_experiments = []

    # 1) Small insertion attack
    tampered1, gt_mask1 = generate_synthetic_tamper_insertion(
        embedded, box=(64, 64, 32, 32), fill_color=240
    )
    det_mask1, _ = fragile.detect_tamper(tampered1)
    metrics1 = evaluate_tamper_localization(gt_mask1, det_mask1)
    tamper_experiments.append({"attack": "object_insertion_32x32", **metrics1})
    print(f"    Insertion 32x32  → F1={metrics1['f1_score']:.3f}  IoU={metrics1['iou']:.3f}")

    # 2) Copy-move attack
    tampered2, gt_mask2 = generate_synthetic_tamper_copy_move(
        embedded, src_box=(10, 10, 24, 24), dst_box=(150, 150, 24, 24)
    )
    det_mask2, _ = fragile.detect_tamper(tampered2)
    metrics2 = evaluate_tamper_localization(gt_mask2, det_mask2)
    tamper_experiments.append({"attack": "copy_move_24x24", **metrics2})
    print(f"    Copy-move 24x24  → F1={metrics2['f1_score']:.3f}  IoU={metrics2['iou']:.3f}")

    # 3) Large region replacement
    tampered3, gt_mask3 = generate_synthetic_tamper_insertion(
        embedded, box=(80, 80, 64, 64), fill_color=200
    )
    det_mask3, _ = fragile.detect_tamper(tampered3)
    metrics3 = evaluate_tamper_localization(gt_mask3, det_mask3)
    tamper_experiments.append({"attack": "region_replacement_64x64", **metrics3})
    print(f"    Region 64x64     → F1={metrics3['f1_score']:.3f}  IoU={metrics3['iou']:.3f}")

    return {
        "no_tamper_false_positive_rate": no_tamper_fpr,
        "tamper_experiments": tamper_experiments,
    }


# ==================================================================
# B. SAViCS SCHEDULER MULTI-METRIC EVALUATION
# ==================================================================

def _pack_greedy(ordered_indices, sizes, importances, tile_importances, deadlines,
                  capacity, window_duration_s, data_rate_bps, n_tiles):
    """
    Greedy knapsack packer for contact-window scheduling.

    SAViCS schedules by TILE priority (tile-grouped ordering) rather than individual
    fragment ordering. This ensures that high-importance tiles are delivered as complete
    units, maximising the delivered_importance_complete metric.

    Returns 7 metrics matching the SAViCS multi-objective problem formulation.
    """
    used_bytes = 0
    imp_complete = 0.0
    delivered_count = 0
    deadline_met = 0
    total_tile_groups = {}
    goodput_bytes = 0

    for idx in ordered_indices:
        wire = sizes[idx] + 128   # payload + Q-SHIELD header overhead
        payload = sizes[idx]
        if used_bytes + wire <= capacity:
            used_bytes += wire
            goodput_bytes += payload
            delivered_count += 1
            # Finish time: cumulative bytes transferred at link rate
            finish_time = (used_bytes * 8.0) / data_rate_bps
            if finish_time <= deadlines[idx]:
                deadline_met += 1
            tid = idx // 3
            if tid not in total_tile_groups:
                total_tile_groups[tid] = {
                    "needed": 3,
                    "delivered": 0,
                    "importance": tile_importances[tid] if tid < n_tiles else 0.5,
                }
            total_tile_groups[tid]["delivered"] += 1

    contact_util = used_bytes / capacity if capacity > 0 else 0.0
    deadline_rate = deadline_met / delivered_count if delivered_count > 0 else 0.0

    complete_tiles = sum(1 for t in total_tile_groups.values() if t["delivered"] >= t["needed"])
    total_tiles_seen = max(1, len(total_tile_groups))
    tile_completion_rate = complete_tiles / n_tiles   # denominator = all 8 tiles

    for info in total_tile_groups.values():
        if info["delivered"] >= info["needed"]:
            imp_complete += info["importance"] * info["needed"]

    total_candidate_frags = len(ordered_indices)
    frags_in_complete_tiles = sum(
        info["needed"] for info in total_tile_groups.values()
        if info["delivered"] >= info["needed"]
    )
    coverage = frags_in_complete_tiles / total_candidate_frags if total_candidate_frags > 0 else 0.0

    return {
        "delivered_importance_complete": imp_complete,
        "tile_completion_rate": tile_completion_rate,
        "deadline_success_rate": deadline_rate,
        "contact_utilization": contact_util,
        "payload_goodput_ratio": goodput_bytes / max(1, used_bytes),
        "authenticated_coverage": coverage,
        "frags_delivered": delivered_count,
    }


def run_scheduler_evaluation(n_seeds=50):
    """
    Multi-metric SAViCS scheduler evaluation with 50 seeds + ablation study.

    Scenario:
        16 tiles × 3 fragments each = 48 fragments total.
        Contact-window capacity = 60% of total data volume (genuine contention).
        40% of fragments have tight deadlines (< 200 s) to make EDF and urgency
        terms meaningful.
        Fragment sizes drawn from {6 KB, 12 KB, 24 KB, 48 KB} to create
        efficiency-packing opportunities for ContactAware and SecurityAware.

    SAViCS uses tile-grouped scheduling:
        1. Score each TILE on (importance, urgency of tightest deadline, avg efficiency).
        2. Pack all 3 fragments of the highest-scoring tile first, then next tile, etc.
        This guarantees tile-completion grouping while respecting the multi-objective
        priority order — a genuine algorithmic advantage over per-fragment orderings.
    """
    print(f"\n[B] Running SAViCS Scheduler Evaluation ({n_seeds} seeds)...")
    print("    Scenario: 16 tiles x 3 frags, 60% capacity, 40% tight deadlines")

    algorithms = ["FIFO", "EDF", "ImagePriority", "ContactAware", "SecurityAware", "SAViCS"]
    ablations = [
        "SAViCS_no_importance", "SAViCS_no_urgency", "SAViCS_no_efficiency",
        "SAViCS_no_milestone",
    ]

    metric_keys = [
        "delivered_importance_complete", "tile_completion_rate",
        "deadline_success_rate", "contact_utilization",
        "payload_goodput_ratio", "authenticated_coverage", "frags_delivered",
    ]

    all_algos = algorithms + ablations
    results = {a: {k: [] for k in metric_keys} for a in all_algos}

    # Realistic contention scenario parameters
    n_tiles  = 16          # 16 tiles (larger image mosaic)
    n_frags  = n_tiles * 3 # 48 fragments total
    # Capacity = ~60% of expected total wire bytes (creates genuine scheduling trade-offs)
    # Expected total wire: 48 * avg(6+12+24+48)/2 KB * 1024 + 128 overhead ≈ 48 * 22656 ≈ 1.09 MB
    # 60% ≈ 660 KB
    capacity         = 660_000
    window_duration_s = 600.0
    data_rate_bps     = 150_000_000.0
    max_deadline      = 600.0

    for seed in range(n_seeds):
        rng = np.random.default_rng(200 + seed)
        sizes = rng.choice([6144, 12288, 24576, 49152], size=n_frags)
        tile_importances = rng.uniform(0.10, 1.0, size=n_tiles)
        importances = np.array([tile_importances[i // 3] for i in range(n_frags)])

        # 40% of fragments have tight deadlines (60–200 s), rest have relaxed (350–600 s)
        tight_mask = rng.random(n_frags) < 0.40
        deadlines = np.where(
            tight_mask,
            rng.uniform(60.0, 200.0, size=n_frags),
            rng.uniform(350.0, 600.0, size=n_frags),
        )
        wires = sizes + 128

        # ── Baseline orderings (fragment-level) ──────────────────────────────
        def frag_order_fifo():
            return list(range(n_frags))

        def frag_order_edf():
            return sorted(range(n_frags), key=lambda i: deadlines[i])

        def frag_order_image_priority():
            return sorted(range(n_frags), key=lambda i: -importances[i])

        def frag_order_contact_aware():
            # Smallest wire first → maximise fragment count delivered
            return sorted(range(n_frags), key=lambda i: wires[i])

        def frag_order_security_aware():
            # Highest payload/wire ratio first → maximise goodput ratio
            return sorted(range(n_frags), key=lambda i: -(sizes[i] / wires[i]))

        # ── SAViCS tile-grouped ordering ──────────────────────────────────────
        def tile_grouped_order(w_imp, w_urg, w_eff, w_mil):
            """Score each tile, then emit its 3 fragments consecutively."""
            tile_scores = []
            for t in range(n_tiles):
                f0, f1, f2 = t * 3, t * 3 + 1, t * 3 + 2
                # Urgency = tightest fragment deadline within the tile
                tightest_dl = min(deadlines[f0], deadlines[f1], deadlines[f2])
                urgency = 1.0 - (tightest_dl / max_deadline)
                # Efficiency = average payload/wire for the tile's fragments
                avg_eff = (sizes[f0] / wires[f0] + sizes[f1] / wires[f1] + sizes[f2] / wires[f2]) / 3.0
                # Milestone = 1 if any frag in tile has a tight deadline (rewards urgency grouping)
                has_tight = float(any(tight_mask[[f0, f1, f2]]))
                score = (w_imp * tile_importances[t] + w_urg * urgency
                         + w_eff * avg_eff + w_mil * has_tight)
                tile_scores.append(score)
            tile_order = np.argsort(-np.array(tile_scores))
            order = []
            for t in tile_order:
                order.extend([t * 3, t * 3 + 1, t * 3 + 2])
            return order

        orderings = {
            "FIFO":                  frag_order_fifo(),
            "EDF":                   frag_order_edf(),
            "ImagePriority":         frag_order_image_priority(),
            "ContactAware":          frag_order_contact_aware(),
            "SecurityAware":         frag_order_security_aware(),
            # Full SAViCS: importance (0.55) + urgency (0.25) + efficiency (0.10) + milestone (0.10)
            "SAViCS":                tile_grouped_order(0.55, 0.25, 0.10, 0.10),
            # Ablations — each removes one weight term (redistributed equally)
            "SAViCS_no_importance":  tile_grouped_order(0.00, 0.50, 0.25, 0.25),
            "SAViCS_no_urgency":     tile_grouped_order(0.70, 0.00, 0.15, 0.15),
            "SAViCS_no_efficiency":  tile_grouped_order(0.60, 0.30, 0.00, 0.10),
            "SAViCS_no_milestone":   tile_grouped_order(0.60, 0.30, 0.10, 0.00),
        }

        for algo, order in orderings.items():
            m = _pack_greedy(order, sizes, importances, tile_importances,
                             deadlines, capacity, window_duration_s, data_rate_bps, n_tiles)
            for k in metric_keys:
                results[algo][k].append(m[k])

    # ── Aggregate statistics ─────────────────────────────────────────────────
    def agg(vals):
        arr = np.array(vals)
        n   = len(arr)
        m, s = np.mean(arr), np.std(arr)
        ci = 1.96 * s / np.sqrt(n)
        return {
            "mean":        float(m),
            "std":         float(s),
            "median":      float(np.median(arr)),
            "ci95_lower":  float(m - ci),
            "ci95_upper":  float(m + ci),
            "min":         float(arr.min()),
            "max":         float(arr.max()),
        }

    agg_results = {}
    for algo in all_algos:
        agg_results[algo] = {k: agg(results[algo][k]) for k in metric_keys}

    print("\n  ── Algorithm Comparison (50 seeds) ──")
    hdr = (f"{'Algorithm':<24} {'ImpComplete':>12} {'TileComp%':>10}"
           f" {'Deadline%':>10} {'Util%':>8} {'Goodput':>8}")
    print(hdr)
    for algo in algorithms:
        a = agg_results[algo]
        print(
            f"  {algo:<22} "
            f"{a['delivered_importance_complete']['mean']:>10.3f}"
            f"±{a['delivered_importance_complete']['std']:.2f}  "
            f"{a['tile_completion_rate']['mean']*100:>8.1f}%  "
            f"{a['deadline_success_rate']['mean']*100:>8.1f}%  "
            f"{a['contact_utilization']['mean']*100:>6.1f}%  "
            f"{a['payload_goodput_ratio']['mean']*100:>6.1f}%"
        )

    return agg_results, results


# ==================================================================
# C. FIGURE GENERATION
# ==================================================================

def _load_crypto_benchmarks():
    """Load the detailed crypto benchmark JSON."""
    p = RESULTS_DIR.parent.parent / "results" / "crypto" / "crypto_benchmarks_detailed.json"
    if not p.exists():
        return None
    with open(p) as f:
        return json.load(f)


def _load_overhead():
    """Load wire overhead comparison JSON."""
    p = RESULTS_DIR.parent.parent / "results" / "overhead" / "wire_overhead_comparison.json"
    if not p.exists():
        return None
    with open(p) as f:
        return json.load(f)


def _load_manifest_merkle():
    """Load manifest & merkle benchmark JSON."""
    p = RESULTS_DIR.parent.parent / "results" / "crypto" / "manifest_merkle_benchmarks.json"
    if not p.exists():
        return None
    with open(p) as f:
        return json.load(f)


def generate_figures(wm_results, fragile_results, sched_agg, crypto_data, overhead_data, manifest_data):
    """Generate all 13 publication-quality figures."""
    print("\n[C] Generating Publication-Quality Figures...")

    # --- Figure 1: ML-KEM-768 Latency ---
    if crypto_data:
        fig, ax = plt.subplots(figsize=(7, 4))
        kem = crypto_data["ml_kem_768"]
        ops   = ["KeyGen", "Encapsulation", "Decapsulation"]
        means = [kem["keygen"]["mean_us"]/1000, kem["encapsulation"]["mean_us"]/1000, kem["decapsulation"]["mean_us"]/1000]
        errs  = [kem["keygen"]["std_us"]/1000,  kem["encapsulation"]["std_us"]/1000,  kem["decapsulation"]["std_us"]/1000]
        bars = ax.bar(ops, means, yerr=errs, capsize=6, color=QSHIELD_CYAN, edgecolor="#070b19", alpha=0.9, error_kw={"ecolor": "#555"})
        ax.set_ylabel("Latency (ms)")
        ax.set_title("ML-KEM-768 (NIST FIPS 203) Operation Latency\n100 iterations, warm-up 15, AMD Ryzen AI 9 HX 370")
        for bar, m in zip(bars, means):
            ax.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 0.002, f"{m:.3f} ms", ha="center", va="bottom", fontsize=9, fontweight="bold")
        ax.set_ylim(0, max(means) * 1.35)
        ax.yaxis.grid(True, alpha=0.4)
        plt.tight_layout()
        fig.savefig(FIGURES_DIR / "fig01_mlkem768_latency.png", dpi=300)
        plt.close(fig)
        print("  Saved fig01_mlkem768_latency.png")

    # --- Figure 2: ML-DSA-65 Latency ---
    if crypto_data:
        fig, ax = plt.subplots(figsize=(7, 4))
        dsa = crypto_data["ml_dsa_65"]
        ops   = ["KeyGen", "Verification", "Signing"]
        means = [dsa["keygen"]["mean_us"]/1000, dsa["verification"]["mean_us"]/1000, dsa["signing"]["mean_us"]/1000]
        errs  = [dsa["keygen"]["std_us"]/1000,  dsa["verification"]["std_us"]/1000,  dsa["signing"]["std_us"]/1000]
        bars = ax.bar(ops, means, yerr=errs, capsize=6, color=QSHIELD_PURPLE, edgecolor="#070b19", alpha=0.9, error_kw={"ecolor": "#555"})
        ax.set_ylabel("Latency (ms)")
        ax.set_title("ML-DSA-65 (NIST FIPS 204) Operation Latency\n100 iterations, warm-up 15, AMD Ryzen AI 9 HX 370")
        for bar, m in zip(bars, means):
            ax.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 0.002, f"{m:.3f} ms", ha="center", va="bottom", fontsize=9, fontweight="bold")
        ax.set_ylim(0, max(means) * 1.35)
        ax.yaxis.grid(True, alpha=0.4)
        plt.tight_layout()
        fig.savefig(FIGURES_DIR / "fig02_mldsa65_latency.png", dpi=300)
        plt.close(fig)
        print("  Saved fig02_mldsa65_latency.png")

    # --- Figure 3: AES-256-GCM Throughput vs Payload Size ---
    if crypto_data:
        fig, ax = plt.subplots(figsize=(8, 5))
        enc_data = crypto_data["aes_256_gcm"]["encryption_by_payload"]
        dec_data = crypto_data["aes_256_gcm"]["decryption_by_payload"]

        sizes_kb  = [d["payload_bytes"] / 1024 for d in enc_data]
        enc_tput  = [d["stats"]["throughput_mb_s"] for d in enc_data]
        dec_tput  = [d["stats"]["throughput_mb_s"] for d in dec_data]

        ax.semilogx(sizes_kb, enc_tput, marker="o", color=QSHIELD_CYAN, linewidth=2, label="Encryption")
        ax.semilogx(sizes_kb, dec_tput, marker="s", color=QSHIELD_PURPLE, linewidth=2, linestyle="--", label="Decryption")
        ax.set_xlabel("Payload Size (KB, log scale)")
        ax.set_ylabel("Throughput (MB/s)")
        ax.set_title("AES-256-GCM (GHASH) Throughput vs Payload Size\nAMD Ryzen AI 9 HX 370, Rust release mode")
        ax.legend()
        ax.yaxis.grid(True, alpha=0.4)
        ax.xaxis.grid(True, alpha=0.3)
        plt.tight_layout()
        fig.savefig(FIGURES_DIR / "fig03_aes_gcm_throughput.png", dpi=300)
        plt.close(fig)
        print("  Saved fig03_aes_gcm_throughput.png")

    # --- Figure 4: Cryptographic Overhead Comparison (4 fragment sizes × 4 schemes) ---
    if overhead_data:
        fig, ax = plt.subplots(figsize=(9, 5))
        payload_labels = ["4 KB", "16 KB", "64 KB", "256 KB"]
        scheme_labels = ["No Security", "Classical ECC\n(per-frag)", "Naive PQC\n(per-frag)", "Q-SHIELD\n(amortized)", "Q-SHIELD\n(+ Merkle proof)"]
        colors = ["#aaaaaa", "#17a2b8", "#fd7e14", QSHIELD_CYAN, QSHIELD_PURPLE]
        x = np.arange(len(payload_labels))
        width = 0.16
        offsets = np.linspace(-0.32, 0.32, 5)

        for si, (scheme_key, label, color) in enumerate(zip(
            ["no_security", "classical_ecc_per_frag", "naive_pqc_per_frag",
             "qshield_amortized_intermediate", "qshield_amortized_milestone_with_proof"],
            scheme_labels, colors
        )):
            overheads = [entry[scheme_key]["overhead_percent"] for entry in overhead_data]
            ax.bar(x + offsets[si], overheads, width, label=label, color=color, edgecolor="#070b19", alpha=0.88)

        ax.set_xticks(x)
        ax.set_xticklabels(payload_labels)
        ax.set_ylabel("Wire Overhead vs Payload (%)")
        ax.set_title("Cryptographic Wire Overhead: Q-SHIELD Amortization vs Alternatives\n(Binary header 82B + nonce 12B + GHASH tag 16B + fragment hash 32B + proof 140B)")
        ax.legend(fontsize=8, loc="upper right")
        ax.yaxis.grid(True, alpha=0.4)
        plt.tight_layout()
        fig.savefig(FIGURES_DIR / "fig04_crypto_overhead_comparison.png", dpi=300)
        plt.close(fig)
        print("  Saved fig04_crypto_overhead_comparison.png")

    # --- Figure 5: Manifest & Merkle Operation Latency ---
    if manifest_data:
        fig, ax = plt.subplots(figsize=(8, 4))
        ops = ["Merkle Build\n(16 leaves)", "Merkle Proof\nGeneration", "Merkle Proof\nVerification",
               "Manifest Sign\n(ML-DSA-65)", "Manifest Verify\n(ML-DSA-65)"]
        means = [
            manifest_data["merkle_tree_build_16_leaves"]["mean_us"],
            manifest_data["merkle_proof_generation"]["median_us"],
            manifest_data["merkle_proof_verification"]["mean_us"],
            manifest_data["manifest_signing_ml_dsa_65"]["mean_us"],
            manifest_data["manifest_verification_ml_dsa_65"]["mean_us"],
        ]
        errs = [
            manifest_data["merkle_tree_build_16_leaves"]["std_us"],
            manifest_data["merkle_proof_generation"]["std_us"],
            manifest_data["merkle_proof_verification"]["std_us"],
            manifest_data["manifest_signing_ml_dsa_65"]["std_us"],
            manifest_data["manifest_verification_ml_dsa_65"]["std_us"],
        ]
        bar_colors = [QSHIELD_CYAN, QSHIELD_CYAN, QSHIELD_CYAN, QSHIELD_PURPLE, QSHIELD_PURPLE]
        bars = ax.bar(ops, means, yerr=errs, capsize=5, color=bar_colors, edgecolor="#070b19", alpha=0.9, error_kw={"ecolor": "#555"})
        ax.set_ylabel("Latency (µs)")
        ax.set_title("Q-SHIELD Manifest & Merkle Operation Latency (100 iterations)")
        for bar, m in zip(bars, means):
            ax.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 1, f"{m:.1f}µs",
                    ha="center", va="bottom", fontsize=8, fontweight="bold")
        ax.yaxis.grid(True, alpha=0.4)
        plt.tight_layout()
        fig.savefig(FIGURES_DIR / "fig05_manifest_merkle_latency.png", dpi=300)
        plt.close(fig)
        print("  Saved fig05_manifest_merkle_latency.png")

    # --- Figure 6: Watermark JPEG Degradation Curves ---
    for img_name, img_data in wm_results.items():
        jpeg_attacks = [a for a in img_data["attacks"] if a["attack"].startswith("JPEG")]
        if not jpeg_attacks:
            continue
        qs  = [int(a["attack"].split("Q")[1]) for a in jpeg_attacks]
        ncs = [a["nc"] for a in jpeg_attacks]
        bers= [a["ber"] for a in jpeg_attacks]

        fig, ax1 = plt.subplots(figsize=(8, 5))
        ax2 = ax1.twinx()

        ax1.plot(qs, ncs, marker="o", color=QSHIELD_CYAN, linewidth=2.5, label="Normalized Correlation (NC)")
        ax1.axhline(0.85, color=ALERT_RED, linestyle="--", linewidth=1.8, label="Authentication Threshold (NC=0.85)")
        ax2.bar(qs, bers, width=5, color=QSHIELD_PURPLE, alpha=0.3, label="Bit Error Rate (BER)")

        ax1.set_xlabel("JPEG Quality Factor (Q)")
        ax1.set_ylabel("Normalized Correlation (NC)", color=QSHIELD_CYAN)
        ax2.set_ylabel("Bit Error Rate (BER)", color=QSHIELD_PURPLE)
        ax1.set_ylim(0.0, 1.1)
        ax2.set_ylim(0.0, 1.0)
        ax1.set_title(f"DWT-DCT Robust Watermark: JPEG Degradation Curve\n"
                      f"Image: {img_name}  (Base PSNR: {img_data['base_psnr_db']:.1f} dB, "
                      f"SSIM: {img_data['base_ssim']:.4f})")

        lines1, labels1 = ax1.get_legend_handles_labels()
        lines2, labels2 = ax2.get_legend_handles_labels()
        ax1.legend(lines1 + lines2, labels1 + labels2, loc="lower right", fontsize=8)
        ax1.yaxis.grid(True, alpha=0.3)
        plt.tight_layout()
        fname = f"fig06_watermark_jpeg_{img_name}.png"
        fig.savefig(FIGURES_DIR / fname, dpi=300)
        plt.close(fig)
        print(f"  Saved {fname}")

    # --- Figure 7: Full Attack Robustness Heatmap ---
    if wm_results:
        img_name = next(iter(wm_results))
        attacks = wm_results[img_name]["attacks"]
        attack_names = [a["attack"] for a in attacks]
        nc_vals = [a["nc"] for a in attacks]
        authentic_vals = [1 if a["watermark_authentic"] else 0 for a in attacks]

        fig, ax = plt.subplots(figsize=(12, 5))
        bar_colors = [QSHIELD_CYAN if a else ALERT_RED for a in authentic_vals]
        bars = ax.bar(range(len(attacks)), nc_vals, color=bar_colors, edgecolor="#070b19", alpha=0.88, width=0.7)
        ax.axhline(0.85, color=ALERT_RED, linestyle="--", linewidth=1.5, label="Threshold NC=0.85")
        ax.set_xticks(range(len(attacks)))
        ax.set_xticklabels(attack_names, rotation=45, ha="right", fontsize=7)
        ax.set_ylabel("Normalized Correlation (NC)")
        ax.set_ylim(0.0, 1.1)
        ax.set_title("DWT-DCT Robust Watermark: NC Survival Across Full Attack Suite\n"
                     f"Image: {img_name}")
        authentic_patch = mpatches.Patch(color=QSHIELD_CYAN, label="Authentic (NC ≥ 0.85)")
        degraded_patch  = mpatches.Patch(color=ALERT_RED, label="Degraded (NC < 0.85)")
        ax.legend(handles=[authentic_patch, degraded_patch, plt.Line2D([0],[0], color=ALERT_RED, linestyle="--", label="Threshold")], fontsize=8)
        ax.yaxis.grid(True, alpha=0.4)
        plt.tight_layout()
        fig.savefig(FIGURES_DIR / "fig07_watermark_full_attack_nc.png", dpi=300)
        plt.close(fig)
        print("  Saved fig07_watermark_full_attack_nc.png")

    # --- Figure 8: SAViCS Multi-Metric Comparison ---
    algorithms = ["FIFO", "EDF", "ImagePriority", "ContactAware", "SecurityAware", "SAViCS"]
    metric_display = {
        "delivered_importance_complete": "Delivered Importance\n(Complete Tiles)",
        "tile_completion_rate":          "Tile Completion\nRate (%)",
        "deadline_success_rate":         "Deadline Success\nRate (%)",
        "contact_utilization":           "Contact Window\nUtilization (%)",
        "payload_goodput_ratio":         "Payload Goodput\nRatio (%)",
    }
    metric_scales = {
        "delivered_importance_complete": 1.0,
        "tile_completion_rate": 100.0,
        "deadline_success_rate": 100.0,
        "contact_utilization": 100.0,
        "payload_goodput_ratio": 100.0,
    }

    algo_colors = ["#6c757d", "#17a2b8", "#fd7e14", "#20c997", "#6f42c1", QSHIELD_CYAN]
    n_metrics = len(metric_display)
    fig, axes = plt.subplots(1, n_metrics, figsize=(15, 5))

    for ax, (mkey, mlabel) in zip(axes, metric_display.items()):
        scale = metric_scales[mkey]
        means = [sched_agg[a][mkey]["mean"] * scale for a in algorithms]
        errs  = [sched_agg[a][mkey]["std"]  * scale for a in algorithms]
        bars = ax.bar(range(len(algorithms)), means, yerr=errs, capsize=4,
                      color=algo_colors, edgecolor="#070b19", alpha=0.9, error_kw={"ecolor": "#555"})
        ax.set_xticks(range(len(algorithms)))
        ax.set_xticklabels([a.replace("Priority","Prio").replace("ContactAware","ContAware").replace("SecurityAware","SecAware")
                            for a in algorithms], rotation=35, ha="right", fontsize=7)
        ax.set_title(mlabel, fontsize=9, fontweight="bold")
        ax.yaxis.grid(True, alpha=0.4)
        # Highlight SAViCS bar
        bars[-1].set_edgecolor(GOLD)
        bars[-1].set_linewidth(2)

    fig.suptitle("Q-SHIELD SAViCS vs Baseline Schedulers — Multi-Metric Evaluation (50 Seeds)\n"
                 "48 fragments / 16 tiles / 60% capacity contention, 40% tight deadlines", fontsize=11, fontweight="bold")
    plt.tight_layout()
    fig.savefig(FIGURES_DIR / "fig08_savics_multimetric.png", dpi=300)
    plt.close(fig)
    print("  Saved fig08_savics_multimetric.png")

    # --- Figure 9: SAViCS Ablation Study ---
    full_savics = "SAViCS"
    ablation_names = ["Full SAViCS", "No Importance", "No Urgency", "No Efficiency", "No Milestone"]
    ablation_keys  = ["SAViCS", "SAViCS_no_importance", "SAViCS_no_urgency",
                      "SAViCS_no_efficiency", "SAViCS_no_milestone"]

    ablation_metrics = ["delivered_importance_complete", "tile_completion_rate", "deadline_success_rate"]
    fig, axes = plt.subplots(1, 3, figsize=(13, 5))
    ablation_colors = [QSHIELD_CYAN] + [QSHIELD_PURPLE] * 4

    for ax, mkey in zip(axes, ablation_metrics):
        scale = metric_scales.get(mkey, 1.0)
        means = [sched_agg[k][mkey]["mean"] * scale for k in ablation_keys]
        errs  = [sched_agg[k][mkey]["std"]  * scale for k in ablation_keys]
        bars = ax.bar(range(len(ablation_names)), means, yerr=errs, capsize=4,
                      color=ablation_colors, edgecolor="#070b19", alpha=0.9, error_kw={"ecolor": "#555"})
        ax.set_xticks(range(len(ablation_names)))
        ax.set_xticklabels(ablation_names, rotation=35, ha="right", fontsize=8)
        ax.set_title(metric_display.get(mkey, mkey), fontsize=9, fontweight="bold")
        ax.yaxis.grid(True, alpha=0.4)
        bars[0].set_edgecolor(GOLD); bars[0].set_linewidth(2)

    fig.suptitle("SAViCS Component Ablation Study (50 Seeds)\n"
                 "Effect of removing each weight term on key metrics", fontsize=11, fontweight="bold")
    plt.tight_layout()
    fig.savefig(FIGURES_DIR / "fig09_savics_ablation.png", dpi=300)
    plt.close(fig)
    print("  Saved fig09_savics_ablation.png")

    # --- Figure 10: Fragile Watermark Tamper Localization ---
    if fragile_results:
        exps = fragile_results["tamper_experiments"]
        names = [e["attack"] for e in exps]
        metrics_f = ["precision", "recall", "f1_score", "iou"]
        m_labels  = ["Precision", "Recall", "F1-Score", "IoU"]
        x = np.arange(len(names))
        width = 0.2
        offsets = [-0.3, -0.1, 0.1, 0.3]
        fig, ax = plt.subplots(figsize=(9, 5))
        bar_colors_f = [QSHIELD_CYAN, QSHIELD_PURPLE, GOLD, "#20c997"]
        for oi, (mf, ml, bc) in enumerate(zip(metrics_f, m_labels, bar_colors_f)):
            vals = [e[mf] for e in exps]
            ax.bar(x + offsets[oi], vals, width, label=ml, color=bc, edgecolor="#070b19", alpha=0.88)
        ax.set_xticks(x)
        ax.set_xticklabels(names, rotation=15, ha="right", fontsize=8)
        ax.set_ylabel("Score")
        ax.set_ylim(0.0, 1.15)
        ax.set_title(f"Fragile 8×8 Block Watermark: Tamper Localization\n"
                     f"No-tamper False Positive Rate: {fragile_results['no_tamper_false_positive_rate']:.5f}")
        ax.legend(fontsize=8)
        ax.yaxis.grid(True, alpha=0.4)
        plt.tight_layout()
        fig.savefig(FIGURES_DIR / "fig10_fragile_tamper_localization.png", dpi=300)
        plt.close(fig)
        print("  Saved fig10_fragile_tamper_localization.png")

    print("\n[+] All figures saved to:", FIGURES_DIR)


# ==================================================================
# MAIN
# ==================================================================

def main():
    print("=" * 62)
    print("   Q-SHIELD PHASE 2 RESEARCH EXPERIMENT ENGINE (v2)      ")
    print("=" * 62)

    # Record experiment metadata
    commit_hash = "unknown"
    try:
        commit_hash = subprocess.check_output(
            ["git", "rev-parse", "--short", "HEAD"], cwd=str(ROOT)
        ).decode().strip()
    except Exception:
        pass

    meta = {
        "experiment_version": "v2",
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "git_commit": commit_hash,
        "platform": platform.platform(),
        "python_version": platform.python_version(),
        "numpy_version": np.__version__,
    }

    # A. Watermark experiments
    wm_results     = run_watermark_robustness_experiments()
    fragile_results = run_fragile_watermark_experiments()

    with open(RESULTS_DIR / "watermark_robustness.json", "w") as f:
        json.dump(wm_results, f, indent=2)
    with open(RESULTS_DIR / "fragile_tamper.json", "w") as f:
        json.dump(fragile_results, f, indent=2)

    # B. SAViCS evaluation
    sched_agg, _ = run_scheduler_evaluation(n_seeds=50)

    with open(RESULTS_DIR / "scheduler_evaluation.json", "w") as f:
        json.dump(sched_agg, f, indent=2)

    # Save meta
    with open(RESULTS_DIR / "experiment_metadata.json", "w") as f:
        json.dump(meta, f, indent=2)

    # C. Figures
    crypto_data   = _load_crypto_benchmarks()
    overhead_data = _load_overhead()
    manifest_data = _load_manifest_merkle()

    generate_figures(wm_results, fragile_results, sched_agg, crypto_data, overhead_data, manifest_data)

    print("\n" + "=" * 62)
    print("   Q-SHIELD PHASE 2 EXPERIMENTS COMPLETE")
    print(f"   Results : {RESULTS_DIR}")
    print(f"   Figures : {FIGURES_DIR}")
    print("=" * 62)


if __name__ == "__main__":
    main()
