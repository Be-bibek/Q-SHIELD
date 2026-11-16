# Security-Aware Verifiable Image Contact Scheduling (SAViCS)

## 1. Problem Formulation
In low Earth orbit (LEO) satellite communications, contact windows between spacecraft and ground stations are brief (typically 8–12 minutes) and intermittent. Classical schedulers (FIFO, Earliest Deadline First) operate under pure throughput or deadline metrics without accounting for:
1. Spatial perceptual variance and intelligence value of individual tiles.
2. Cryptographic packet expansion and authentication metadata overhead.
3. Verification milestone dependencies (e.g. Merkle leaf inclusion and manifest signatures).
4. Unrecoverable truncation of half-transmitted fragments when Loss of Signal (LOS) occurs.

## 2. SAViCS Formulation
SAViCS assigns each candidate fragment $f_i$ a composite utility score:
$$U_i = w_{\text{imp}} \cdot \text{importance}_i + w_{\text{urgency}} \cdot \left(1 - \frac{d_i}{T_{\max}}\right) + w_{\text{eff}} \cdot \frac{\text{size}_i}{\text{wire\_bytes}_i} + w_{\text{retrans}} \cdot \mathbb{I}_{\text{retrans}} + w_{\text{mile}} \cdot \mathbb{I}_{\text{milestone}}$$

### Heuristic Packing:
Fragments are sequenced by descending $U_i$ and scheduled into the earliest contact window where remaining capacity $C_w \ge \text{wire\_bytes}_i$. Fragments whose transmission would cross the LOS boundary with high interruption probability are queued for the subsequent pass.
