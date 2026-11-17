# SAViCS Experimental Results & Statistical Analysis

**Document version:** 2.0  
**Date:** 2026-11-17  
**Evaluation:** 50 independent Monte Carlo seeds  
**Scenario:** 16 tiles × 3 fragments = 48 fragments total, 660 KB contact capacity (~60% contention), 40% tight deadlines (< 200 s).  
**Raw data location:** `experiments/results/v2/scheduler_evaluation.json`  
**Figures:** `experiments/figures/v2/fig08_savics_multimetric.png`, `experiments/figures/v2/fig09_savics_ablation.png`

---

## 1. Executive Summary

This document presents the empirical evaluation of the **SAViCS (Security-Aware Visibility Contact Scheduling)** algorithm compared against five standard baseline scheduling strategies:
1. **FIFO** (First-In, First-Out)
2. **EDF** (Earliest Deadline First)
3. **ImagePriority** (Greedy Importance-First)
4. **ContactAware** (Smallest Wire-Size First)
5. **SecurityAware** (Highest Payload-to-Wire Ratio First)

Evaluations were performed across 50 independent Monte Carlo simulation seeds under realistic LEO satellite contact contention.

---

## 2. Quantitative Baseline Comparison (50 Seeds)

All metrics are reported as **Mean ± Standard Deviation [95% Confidence Interval]**:

| Algorithm | Delivered Importance (Complete Tiles) | Tile Completion Rate (%) | Contact Utilization (%) | Payload Goodput Ratio (%) | Fragments Delivered |
|---|---:|---:|---:|---:|---:|
| **FIFO** | 14.69 ± 2.98 [13.87, 15.52] | 55.5 ± 8.4% | 99.26 ± 0.12% | 99.43 ± 0.07% | 28.94 ± 3.82 |
| **EDF** | 6.91 ± 2.83 [6.13, 7.70] | 26.8 ± 11.1% | 99.23 ± 0.19% | 99.40 ± 0.07% | 30.62 ± 3.84 |
| **ImagePriority** | **19.65 ± 2.52 [18.95, 20.34]** | 56.5 ± 7.9% | 99.25 ± 0.07% | 99.43 ± 0.07% | 29.36 ± 3.68 |
| **ContactAware** | 13.93 ± 3.17 [13.05, 14.81] | 52.5 ± 10.4% | 95.94 ± 2.20% | 99.23 ± 0.04% | 38.22 ± 2.14 |
| **SecurityAware** | 2.31 ± 1.92 [1.78, 2.84] | 8.4 ± 6.2% | 99.91 ± 0.12% | 99.68 ± 0.03% | 16.48 ± 1.45 |
| **SAViCS** | **18.93 ± 2.23 [18.31, 19.54]** | **56.8 ± 6.8%** | 99.25 ± 0.06% | 99.43 ± 0.06% | 29.38 ± 3.28 |

---

## 3. Analysis & Key Research Insights

### 3.1 SAViCS vs. ImagePriority: The Multi-Objective Trade-Off

- **Importance delivery:** ImagePriority achieves a marginally higher mean delivered importance (19.65 vs. 18.93, a 3.7% difference). However, the 95% confidence intervals overlap ([18.95, 20.34] vs. [18.31, 19.54]), indicating that this difference is **statistically non-significant**.
- **Tile completion stability:** SAViCS demonstrates a slightly higher mean tile completion rate (56.8% vs. 56.5%) and a lower standard deviation (6.8% vs. 7.9%), showing greater scheduling stability across heterogeneous contact passes.
- **Architectural distinction:** ImagePriority is a **single-metric heuristic**. If emergency latency constraints, retransmission queues, or milestone cryptographic proofs are introduced, ImagePriority is blind to them. In contrast, SAViCS uses a formal parameterized weight vector:

$$S(t) = w_{\text{imp}} \cdot I(t) + w_{\text{urg}} \cdot U(t) + w_{\text{eff}} \cdot E(t) + w_{\text{mil}} \cdot M(t)$$

allowing ground station operators to adaptively prioritize urgent tactical reconnaissance ($w_{\text{urg}} \uparrow$) versus standard environmental monitoring ($w_{\text{imp}} \uparrow$).

### 3.2 Failure Modes of Specialized Baselines

- **Earliest Deadline First (EDF):** EDF performs poorly on image reconstruction (6.91 importance, 26.8% tile completion). Because EDF only prioritizes individual fragment deadlines without regard to tile grouping or image importance, it delivers isolated fragments scattered across multiple tiles, leaving most tiles un-reconstructible.
- **SecurityAware:** Performs worst (2.31 importance, 8.4% completion). SecurityAware prioritizes the highest payload-to-wire ratio (largest fragments), which drastically consumes the contact window capacity with very few fragments (16.48 delivered), failing to assemble full tiles.
- **ContactAware:** Delivers the highest raw fragment count (38.22 frags) by packing the smallest wire sizes, but only achieves 52.5% tile completion because small fragments belong to lower-resolution or lower-priority tiles.

---

## 4. SAViCS Component Ablation Study (50 Seeds)

To verify the necessity and sensitivity of each term in the SAViCS scoring formulation, an ablation study was conducted where each component weight was set to zero and the remaining weights redistributed:

| Configuration | Removed Component | Delivered Importance | Tile Completion (%) | CI95 (Importance) |
|---|---|---:|---:|---|
| **Full SAViCS** | None (Baseline) | **18.93 ± 2.23** | **56.8 ± 6.8%** | [18.31, 19.54] |
| `SAViCS_no_importance` | $w_{\text{imp}} = 0$ | 15.17 ± 2.93 | 57.2 ± 6.3% | [14.36, 15.98] |
| `SAViCS_no_urgency` | $w_{\text{urg}} = 0$ | 19.41 ± 2.27 | 56.4 ± 6.8% | [18.78, 20.04] |
| `SAViCS_no_efficiency` | $w_{\text{eff}} = 0$ | 18.82 ± 2.22 | 56.5 ± 6.8% | [18.21, 19.44] |
| `SAViCS_no_milestone` | $w_{\text{mil}} = 0$ | 19.32 ± 2.24 | 56.5 ± 6.8% | [18.70, 19.94] |

### Ablation Findings:
1. **Importance term is critical:** Removing $w_{\text{imp}}$ reduces delivered importance by **19.9%** (from 18.93 to 15.17), with a non-overlapping confidence interval.
2. **Urgency trade-off:** Removing $w_{\text{urg}}$ increases raw importance delivery to 19.41 because the scheduler is freed from prioritizing urgent tiles with lower visual importance. However, this sacrifices time-critical mission objectives.
3. **Efficiency & Milestone stabilization:** Setting $w_{\text{eff}}$ or $w_{\text{mil}}$ to 0 shows small marginal effects in this synthetic contact scenario, but provides structural guarantees against fragment bloat and incomplete Merkle proof chains.

---

## 5. Conclusion & Recommendations for the Paper

When presenting SAViCS in the research manuscript:
1. **Avoid claiming "superiority" over ImagePriority on raw importance.** Frame SAViCS honestly as achieving **statistically equivalent importance delivery** while uniquely providing a **multi-objective framework** that accommodates deadlines, security overhead, and atomic tile grouping.
2. **Highlight tile-grouped scheduling:** Explain that atomic tile grouping is what allows high-importance tiles to be immediately verified via Merkle inclusion proofs upon receipt, enabling real-time preview during the contact pass.
