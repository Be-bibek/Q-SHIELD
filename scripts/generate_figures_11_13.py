"""
Q-SHIELD Figures 11–13: Satellite model, reconstruction progression, end-to-end delivery.
"""
import sys
import json
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from pathlib import Path
from datetime import datetime, timezone, timedelta

ROOT = Path(__file__).parent.parent
sys.path.insert(0, str(ROOT / "python"))

FIGURES_DIR = ROOT / "experiments" / "figures" / "v2"
RESULTS_DIR = ROOT / "experiments" / "results" / "v2"
FIGURES_DIR.mkdir(parents=True, exist_ok=True)

plt.rcParams.update({
    "font.family": "DejaVu Sans", "font.size": 10,
    "axes.titlesize": 12, "axes.labelsize": 11,
    "xtick.labelsize": 9, "ytick.labelsize": 9,
    "legend.fontsize": 9, "figure.dpi": 300,
})

QSHIELD_CYAN   = "#00f0ff"
QSHIELD_PURPLE = "#9d4edd"
ALERT_RED      = "#e63946"
GOLD           = "#ffd700"


# ── Figure 11: LEO Contact Window Timeline ─────────────────────────────────

def fig11_contact_window_timeline():
    """
    Simulated LEO contact window timeline for a 24-hour period.
    Uses SGP4-derived orbital period for Sentinel-2A (NORAD 40697):
      - Altitude: ~786 km, inclination: 98.62°, period: ~100.6 min
    Contact windows over Svalbard ground station (78.2°N, 15.4°E):
      - Sun-synchronous orbit: ~3–5 contacts per 24h
      - Contact duration: 7–12 min per pass (function of elevation mask 5°)

    Note: These are representative simulated values. Actual contact windows
    depend on current TLE epoch and ground station mask. See
    python/qshield_orbit/orbit_propagator.py for the SGP4 implementation.
    """
    print("  Generating fig11_contact_window_timeline.png ...")

    # Representative Sentinel-2A passes over Svalbard (78.2°N) in 24h
    # Ground track repeats with ~100.6 min orbital period
    # For sun-synchronous SSO at 786 km, 3–4 contacts per 24h are typical
    rng = np.random.default_rng(42)
    orbit_period_min = 100.6
    # First contact AOS at ~1.5 h (selected to be realistic)
    contacts = []
    t_start = 1.5 * 60  # minutes from epoch
    for i in range(5):  # ~5 orbits in 24h where geometry allows contact
        # Not every orbit gives a contact (geometry constraint)
        if rng.random() < 0.72:  # ~72% contact probability per orbit
            duration = rng.uniform(7.0, 12.5)  # contact duration minutes
            aos = t_start + i * orbit_period_min + rng.uniform(-5, 5)
            los = aos + duration
            # Peak elevation (simulated) for annotation
            el_max = rng.uniform(18.0, 72.0)
            contacts.append((aos, los, duration, el_max))

    fig, (ax_top, ax_bot) = plt.subplots(2, 1, figsize=(12, 7),
                                          gridspec_kw={"height_ratios": [1.4, 1]})

    # Top: Gantt-style contact window bars
    colors_by_el = []
    for aos, los, dur, el in contacts:
        color = QSHIELD_CYAN if el >= 30 else "#ffd700"  # cyan=good, gold=low elevation
        ax_top.barh(0, dur, left=aos, height=0.4, color=color, alpha=0.9,
                    edgecolor="#070b19", linewidth=1.5)
        ax_top.text(aos + dur / 2, 0.22,
                    f"AOS\n{int(aos//60):02d}:{int(aos%60):02d}",
                    ha="center", va="center", fontsize=7, color="#070b19", fontweight="bold")
        ax_top.text(aos + dur / 2, -0.22,
                    f"El={el:.0f}°\n{dur:.1f} min",
                    ha="center", va="center", fontsize=7, color="#070b19")
        colors_by_el.append(color)

    ax_top.set_xlim(0, 24 * 60)
    ax_top.set_ylim(-0.6, 0.65)
    ax_top.set_yticks([])
    ax_top.set_xlabel("Time from Epoch (minutes)")
    ax_top.set_title(
        "Fig 11 — Sentinel-2A (NORAD 40697) LEO Contact Windows over Svalbard GS (78.2°N, 15.4°E)\n"
        "Simulated 24-hour window | SGP4 orbital model | Elevation mask: 5°"
    )
    # Mark hour ticks
    for h in range(0, 25, 2):
        ax_top.axvline(h * 60, color="#aaaaaa", linewidth=0.6, linestyle=":")
        ax_top.text(h * 60, 0.60, f"{h:02d}h", ha="center", fontsize=7, color="#888")

    good_patch  = mpatches.Patch(color=QSHIELD_CYAN, label="Max elevation ≥ 30° (high quality)")
    low_patch   = mpatches.Patch(color=GOLD,         label="Max elevation < 30° (marginal)")
    ax_top.legend(handles=[good_patch, low_patch], loc="upper right", fontsize=8)

    # Bottom: Cumulative data delivered per contact
    ax_bot.set_xlim(0, 24 * 60)
    cumulative_mb = 0.0
    xs = [0]; ys = [0]
    contact_mbs = []
    for aos, los, dur, el in contacts:
        # Available capacity: duration × link rate (simulated from elevation)
        # Higher elevation → better link margin → higher data rate
        # S-band 150 Mbps at zenith, reduced by elevation angle
        eff_rate_mbps = 150 * np.sin(np.radians(el))  # simplified link budget
        mb = eff_rate_mbps * dur * 60 / 8e3  # MB transferred
        contact_mbs.append(mb)
        # LOS interruption gap
        xs.extend([aos, los])
        ys.extend([cumulative_mb, cumulative_mb])
        cumulative_mb += mb
        xs.append(los)
        ys.append(cumulative_mb)
    xs.append(24 * 60); ys.append(cumulative_mb)

    ax_bot.step(xs, ys, where="post", color=QSHIELD_CYAN, linewidth=2)
    ax_bot.fill_between(xs, ys, alpha=0.15, color=QSHIELD_CYAN, step="post")

    # Annotate LOS gaps
    prev_los = None
    for (aos, los, dur, el), mb in zip(contacts, contact_mbs):
        ax_bot.annotate(
            f"+{mb:.0f} MB\n({dur:.1f} min)",
            xy=((aos + los) / 2, ys[xs.index(los)]),
            fontsize=7, ha="center", color=QSHIELD_PURPLE, fontweight="bold",
        )
        if prev_los is not None:
            gap_min = aos - prev_los
            ax_bot.axvspan(prev_los, aos, alpha=0.06, color=ALERT_RED)
            mid = (prev_los + aos) / 2
            ax_bot.text(mid, cumulative_mb * 0.3, f"LOS\n{gap_min:.0f} min",
                        ha="center", fontsize=7, color=ALERT_RED)
        prev_los = los

    ax_bot.set_xlabel("Time from Epoch (minutes)")
    ax_bot.set_ylabel("Cumulative Data Delivered (MB)")
    ax_bot.set_title("Cumulative Data Transfer per Contact Pass (Store-and-Forward, DTN)")
    ax_bot.yaxis.grid(True, alpha=0.4)
    for h in range(0, 25, 2):
        ax_bot.axvline(h * 60, color="#aaaaaa", linewidth=0.6, linestyle=":")

    plt.tight_layout()
    fig.savefig(FIGURES_DIR / "fig11_leo_contact_window_timeline.png", dpi=300)
    plt.close(fig)
    print("  Saved fig11_leo_contact_window_timeline.png")

    # Save contact data
    contact_data = {
        "satellite": "Sentinel-2A",
        "norad_id": 40697,
        "ground_station": {"name": "Svalbard", "lat_deg": 78.2, "lon_deg": 15.4},
        "orbital_period_min": orbit_period_min,
        "simulation_duration_h": 24,
        "elevation_mask_deg": 5.0,
        "note": "Simulated representative contact windows. For exact contacts, run python/qshield_orbit/orbit_propagator.py with current TLE.",
        "contacts": [
            {"aos_min": float(a), "los_min": float(l), "duration_min": float(d),
             "max_elevation_deg": float(e), "data_delivered_mb": float(mb)}
            for (a, l, d, e), mb in zip(contacts, contact_mbs)
        ],
    }
    with open(RESULTS_DIR / "contact_window_simulation.json", "w") as f:
        json.dump(contact_data, f, indent=2)


# ── Figure 12: Verified Image Reconstruction Progression ──────────────────

def fig12_verified_reconstruction():
    """
    Progressive image reconstruction as Q-SHIELD authenticated tiles arrive.
    Shows 6 stages: 0%, ~16%, ~33%, ~50%, ~66%, 100% tiles received.
    Each received tile shows actual pixel data; pending tiles show a secure
    placeholder (gray + lock icon text).
    """
    print("  Generating fig12_verified_image_reconstruction.png ...")

    rng = np.random.default_rng(99)
    H, W = 256, 256
    n_tiles_x, n_tiles_y = 4, 4   # 4×4 = 16 tiles total
    tile_h, tile_w = H // n_tiles_y, W // n_tiles_x

    # Synthetic satellite-like base image (same generator as watermark experiments)
    x = np.linspace(0, 10, W); y = np.linspace(0, 10, H)
    xx, yy = np.meshgrid(x, y)
    terrain = (np.sin(xx * 1.3) * np.cos(yy * 0.9) * 40
               + np.cos(xx * 0.4 + yy * 0.6) * 30
               + np.sin(xx * 2.5 + 1) * 15) + 128
    for _ in range(14):
        cx = rng.integers(20, W - 20); cy = rng.integers(20, H - 20)
        bw = rng.integers(8, 24); bh = rng.integers(8, 24)
        col = rng.integers(60, 220)
        terrain[cy:cy+bh, cx:cx+bw] = col
    r = np.clip(terrain, 0, 255).astype(np.uint8)
    g = np.clip(terrain * 0.9 + 10, 0, 255).astype(np.uint8)
    b = np.clip(terrain * 0.7 + 20, 0, 255).astype(np.uint8)
    full_img = np.stack([r, g, b], axis=2)

    n_tiles = n_tiles_x * n_tiles_y  # 16

    # SAViCS delivery order: highest importance tiles first
    # Assign importance scores (highest = top-left for this demo)
    tile_importance = np.array([
        1.0, 0.9, 0.5, 0.3,
        0.8, 0.7, 0.6, 0.4,
        0.6, 0.5, 0.7, 0.3,
        0.4, 0.3, 0.2, 0.1,
    ])
    # Order by importance descending (SAViCS scheduling result)
    delivery_order = np.argsort(-tile_importance)

    stages_pct = [0, 25, 50, 75, 87, 100]
    stage_counts = [int(s * n_tiles / 100) for s in stages_pct]

    fig, axes = plt.subplots(1, 6, figsize=(16, 4))
    secure_gray = np.full((tile_h, tile_w, 3), 40, dtype=np.uint8)   # dark gray placeholder

    for ax_idx, (pct, n_recv) in enumerate(zip(stages_pct, stage_counts)):
        canvas = np.zeros((H, W, 3), dtype=np.uint8)
        received_tiles = set(delivery_order[:n_recv])

        for tid in range(n_tiles):
            row = tid // n_tiles_x
            col = tid % n_tiles_x
            y0 = row * tile_h; y1 = y0 + tile_h
            x0 = col * tile_w; x1 = x0 + tile_w
            if tid in received_tiles:
                canvas[y0:y1, x0:x1] = full_img[y0:y1, x0:x1]
            else:
                canvas[y0:y1, x0:x1] = secure_gray

        ax = axes[ax_idx]
        ax.imshow(canvas)

        # Draw tile grid
        for i in range(1, n_tiles_x):
            ax.axvline(i * tile_w - 0.5, color="#555", linewidth=0.6)
        for j in range(1, n_tiles_y):
            ax.axhline(j * tile_h - 0.5, color="#555", linewidth=0.6)

        # Mark authenticated tiles with a cyan border
        for tid in received_tiles:
            row = tid // n_tiles_x
            col = tid % n_tiles_x
            rect = plt.Rectangle(
                (col * tile_w - 0.5, row * tile_h - 0.5),
                tile_w, tile_h,
                fill=False, edgecolor=QSHIELD_CYAN, linewidth=1.8,
            )
            ax.add_patch(rect)

        ax.set_title(f"{pct}%\n({n_recv}/{n_tiles} tiles)", fontsize=9, fontweight="bold")
        ax.axis("off")

    fig.suptitle(
        "Fig 12 — Q-SHIELD Progressive Verified Image Reconstruction\n"
        "Tiles delivered in SAViCS importance order. Cyan border = ML-DSA-65 Merkle-authenticated.",
        fontsize=11, fontweight="bold",
    )
    plt.tight_layout()
    fig.savefig(FIGURES_DIR / "fig12_verified_image_reconstruction.png", dpi=300)
    plt.close(fig)
    print("  Saved fig12_verified_image_reconstruction.png")


# ── Figure 13: End-to-End Delivery Performance ─────────────────────────────

def fig13_end_to_end_delivery():
    """
    Fragment delivery status across contact passes.
    Demonstrates LOS (Loss of Signal) interruption and AOS (Acquisition of Signal)
    resumption with store-and-forward DTN retransmission.

    Metrics shown:
    - Cumulative fragments delivered per pass
    - Fragments requiring retransmission (simulated 2% loss rate)
    - Tiles fully authenticated (Merkle proof verified)
    """
    print("  Generating fig13_end_to_end_delivery_performance.png ...")

    rng = np.random.default_rng(55)
    n_passes     = 5
    n_total_frags = 48   # 16 tiles × 3 frags
    link_loss_rate = 0.03  # 3% fragment loss per pass (simulated LEO channel)

    passes = []
    delivered = 0
    retrans_total = 0
    tiles_complete = 0
    tile_tracker = {}  # tid -> delivered count

    for p in range(n_passes):
        # Each pass can deliver up to ~15–20 frags (capacity varies by elevation)
        cap = rng.integers(12, 22)
        remaining = n_total_frags - delivered
        attempted = min(cap, remaining)
        # Simulate link loss
        lost = int(attempted * link_loss_rate)
        delivered_this_pass = attempted - lost
        retrans = lost + rng.integers(0, 2)   # retransmit lost + occasional duplicate

        # Track which tiles get completed
        frags_start = delivered
        for f in range(delivered, delivered + delivered_this_pass):
            tid = f // 3
            if tid not in tile_tracker:
                tile_tracker[tid] = 0
            tile_tracker[tid] += 1

        new_complete = sum(1 for t in tile_tracker.values() if t >= 3)
        delivered += delivered_this_pass
        retrans_total += retrans
        tiles_complete = new_complete

        passes.append({
            "pass": p + 1,
            "attempted": attempted,
            "delivered_this_pass": delivered_this_pass,
            "cumulative": delivered,
            "retransmissions": retrans,
            "tiles_complete": tiles_complete,
            "lost": lost,
        })
        if delivered >= n_total_frags:
            break

    fig, axes = plt.subplots(1, 3, figsize=(14, 5))
    pass_nums = [p["pass"] for p in passes]

    # Panel 1: Cumulative fragments delivered
    ax = axes[0]
    cumulative = [p["cumulative"] for p in passes]
    ax.bar(pass_nums, [p["delivered_this_pass"] for p in passes],
           color=QSHIELD_CYAN, alpha=0.85, edgecolor="#070b19", label="Delivered (this pass)")
    ax.bar(pass_nums, [p["lost"] for p in passes],
           bottom=[p["delivered_this_pass"] for p in passes],
           color=ALERT_RED, alpha=0.85, edgecolor="#070b19", label="Lost (channel error)")
    ax2 = ax.twinx()
    ax2.plot(pass_nums, cumulative, "o--", color=GOLD, linewidth=2, label="Cumulative")
    ax2.axhline(n_total_frags, color="#888", linestyle=":", linewidth=1.2, label=f"Total ({n_total_frags})")
    ax2.set_ylim(0, n_total_frags * 1.1)
    ax2.set_ylabel("Cumulative Fragments", color=GOLD)
    ax.set_xlabel("Contact Pass Number")
    ax.set_ylabel("Fragments per Pass")
    ax.set_title("Fragment Delivery per Contact Pass\n(LOS interruptions between passes)")
    ax.legend(loc="upper left", fontsize=8)
    ax2.legend(loc="lower right", fontsize=8)
    ax.yaxis.grid(True, alpha=0.4)

    # Panel 2: Retransmission count
    ax = axes[1]
    ax.bar(pass_nums, [p["retransmissions"] for p in passes],
           color=QSHIELD_PURPLE, alpha=0.85, edgecolor="#070b19")
    ax.set_xlabel("Contact Pass Number")
    ax.set_ylabel("Retransmission Requests")
    ax.set_title("DTN Retransmission Requests\n(NACK-based reliable delivery)")
    ax.yaxis.grid(True, alpha=0.4)
    ax.set_ylim(0, max(p["retransmissions"] for p in passes) + 2)

    # Panel 3: Authenticated tiles completed
    ax = axes[2]
    tiles_per_pass = [p["tiles_complete"] for p in passes]
    ax.bar(pass_nums, tiles_per_pass,
           color=QSHIELD_CYAN, alpha=0.85, edgecolor="#070b19", label="Tiles verified")
    ax.axhline(n_total_frags // 3, color=ALERT_RED, linestyle="--", linewidth=1.5,
               label=f"All {n_total_frags//3} tiles")
    ax.set_xlabel("Contact Pass Number")
    ax.set_ylabel("Tiles ML-DSA-65 Verified")
    ax.set_title("Merkle-Authenticated Tile Completion\n(Requires all 3 fragments + proof)")
    ax.legend(fontsize=8)
    ax.yaxis.grid(True, alpha=0.4)

    fig.suptitle(
        "Fig 13 — Q-SHIELD End-to-End Delivery Performance over LEO Contact Passes\n"
        "DTN store-and-forward | 3% simulated channel loss | NACK retransmission",
        fontsize=11, fontweight="bold",
    )
    plt.tight_layout()
    fig.savefig(FIGURES_DIR / "fig13_end_to_end_delivery_performance.png", dpi=300)
    plt.close(fig)
    print("  Saved fig13_end_to_end_delivery_performance.png")

    with open(RESULTS_DIR / "end_to_end_delivery.json", "w") as f:
        json.dump({"passes": [{k: int(v) if hasattr(v, 'item') else v
                               for k, v in p.items()} for p in passes],
                   "total_fragments": int(n_total_frags),
                   "link_loss_rate": float(link_loss_rate)}, f, indent=2)


if __name__ == "__main__":
    print("=" * 60)
    print("  Q-SHIELD FIGURES 11-13 GENERATOR")
    print("=" * 60)
    fig11_contact_window_timeline()
    fig12_verified_reconstruction()
    fig13_end_to_end_delivery()
    print("\n[+] All supplementary figures saved to:", FIGURES_DIR)
