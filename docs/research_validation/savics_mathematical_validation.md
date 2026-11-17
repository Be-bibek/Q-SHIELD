# SAViCS: Mathematical Formulation and Validation

**Document Version**: 2.0  
**Authors**: Antigravity Principal Research Software Engineer & Satellite Systems Researcher  
**Target Algorithm**: SAViCS (Security-Aware Verifiable Image Contact Scheduling)

---

## 1. Problem Formulation: Contact-Constrained Image Transmission

Let an acquired satellite or remote field image $\mathcal{I}$ be partitioned into a spatial grid of $M$ tiles:
$$\mathcal{T} = \{\tau_1, \tau_2, \dots, \tau_M\}$$
Each tile $\tau_k$ has an associated perceptual importance score $\mathcal{I}_k \in (0, 1]$, computed from local spatial frequency gradients and chromatic variance.

To satisfy physical MTU constraints and enable independent AEAD authentication, each tile $\tau_k$ is sliced into $K_k$ cryptographic fragments:
$$\mathcal{F}_k = \{f_{k, 1}, f_{k, 2}, \dots, f_{k, K_k}\}$$
The total set of candidate fragments across the image session is $\mathcal{F} = \bigcup_{k=1}^M \mathcal{F}_k$, where $N = |\mathcal{F}|$.

### 1.1 Fragment Parameters
For each fragment $i \in \mathcal{F}$:
*   $S_i \in \mathbb{N}$: Useful image payload size in bytes.
*   $O_i \in \mathbb{N}$: Cryptographic wire overhead (header, nonce, GHASH tag, Merkle proof).
*   $W_i = S_i + O_i$: Total over-the-air wire size in bytes.
*   $d_i \in \mathbb{R}^+$: Delivery deadline (seconds from epoch).
*   $\tau(i) \in \{1, \dots, M\}$: Parent tile index.
*   $\mathcal{R}_i \in \{0, 1\}$: Binary flag indicating whether fragment $i$ is a retransmission (NACK recovery).
*   $\mathcal{M}_i \in \{0, 1\}$: Binary flag indicating whether fragment $i$ completes parent tile $\tau(i)$ or image manifest.

### 1.2 Ephemeral Contact Windows
The satellite communication link is available exclusively during $W$ discrete line-of-sight contact opportunities:
$$\mathcal{W} = \{c_1, c_2, \dots, c_W\}$$
Each contact window $j \in \mathcal{W}$ is characterized by:
*   $[t_j^{\text{start}}, t_j^{\text{end}}]$: Temporal visibility window.
*   $R_j(t)$: Time-varying data transmission rate (bps), bounded by atmospheric attenuation and elevation angle $\theta(t)$.
*   $C_j = \int_{t_j^{\text{start}}}^{t_j^{\text{end}}} \frac{R_j(t)}{8} \, dt$: Maximum byte capacity of window $j$.

---

## 2. Formal Optimization Problem

### 2.1 Decision Variables
Let binary decision variable $x_{i, j} \in \{0, 1\}$ denote whether fragment $i$ is scheduled for transmission during contact window $j$:
$$x_{i, j} \in \{0, 1\}, \quad \forall i \in \mathcal{F}, \forall j \in \mathcal{W}$$

Let auxiliary binary variable $y_k \in \{0, 1\}$ denote whether tile $\tau_k$ is **fully completed and authenticated**:
$$y_k = \prod_{i \in \mathcal{F}_k} \left( \sum_{j \in \mathcal{W}} x_{i, j} \right) \in \{0, 1\}$$

### 2.2 System Constraints

1.  **Capacity Constraint**: The total scheduled wire bytes in any contact window $j$ cannot exceed the window's physical capacity $C_j$:
    $$\sum_{i \in \mathcal{F}} x_{i, j} \cdot W_i \le C_j, \quad \forall j \in \mathcal{W}$$

2.  **Uniqueness Constraint**: Each fragment is scheduled at most once per window (unless retransmission is triggered by channel feedback):
    $$\sum_{j \in \mathcal{W}} x_{i, j} \le 1, \quad \forall i \in \mathcal{F}$$

3.  **Deadline Constraint**: If fragment $i$ is scheduled in window $j$, its transmission completion time $t_{i, j}^{\text{finish}}$ must not exceed deadline $d_i$:
    $$x_{i, j} \cdot t_{i, j}^{\text{finish}} \le d_i, \quad \forall i \in \mathcal{F}, \forall j \in \mathcal{W}$$

4.  **Security Verification Dependency**: In Delay-Tolerant Networking and BPSec (RFC 9172), an image tile cannot be reconstructed until its Merkle leaf hash matches and its inclusion proof verifies against the manifest root:
    $$y_k = 1 \iff \forall i \in \mathcal{F}_k, \text{AEAD\_Verify}(i) = \text{true} \land \text{Merkle\_Verify}(\tau_k) = \text{true}$$

---

## 3. The SAViCS Heuristic Optimization Formulation

Because the multi-window, deadline-constrained tile completion problem reduces to the Multidimensional Knapsack Problem (which is strongly NP-hard), an exact solver incurs exponential complexity $\mathcal{O}(2^N)$ and is intractable for real-time onboard satellite scheduling.

SAViCS computes a composite multi-attribute utility score $U_i$ for each candidate fragment $i$:
$$U_i = w_{\text{imp}} \cdot \mathcal{I}_i + w_{\text{urgency}} \cdot \mathcal{U}_i + w_{\text{eff}} \cdot \mathcal{E}_i + w_{\text{retrans}} \cdot \mathcal{R}_i + w_{\text{mile}} \cdot \mathcal{M}_i$$

Where the normalized component functions are defined as:

1.  **Perceptual Importance**:
    $$\mathcal{I}_i = \text{Importance}(\tau(i)) \in (0, 1]$$
2.  **Deadline Urgency**:
    $$\mathcal{U}_i = \left( 1 - \frac{d_i}{T_{\max}} \right)^+ \in [0, 1]$$
3.  **Cryptographic Wire Efficiency**:
    $$\mathcal{E}_i = \frac{S_i}{W_i} = \frac{S_i}{S_i + O_i} \in (0, 1)$$
4.  **Retransmission Urgency Bonus**:
    $$\mathcal{R}_i = \begin{cases} 1.0, & \text{if fragment } i \text{ recovers an uncompleted tile previously sent} \\ 0.0, & \text{otherwise} \end{cases}$$
5.  **Tile Completion Milestone Reward**:
    $$\mathcal{M}_i = \begin{cases} 1.0, & \text{if fragment } i \text{ completes the remaining fragments for } \tau(i) \\ 0.0, & \text{otherwise} \end{cases}$$

### Calibrated Weight Vector
$$\mathbf{w} = [w_{\text{imp}}, w_{\text{urgency}}, w_{\text{eff}}, w_{\text{retrans}}, w_{\text{mile}}] = [0.35, 0.25, 0.15, 0.15, 0.10]$$
$$\sum_{m=1}^5 w_m = 1.00$$

### Algorithmic Complexity
Sorting $N$ fragments by $U_i$ takes $\mathcal{O}(N \log N)$. Packing into $W$ contact windows takes $\mathcal{O}(N \cdot W)$. The overall computational complexity is:
$$\mathcal{O}(N \log N + N \cdot W)$$
This enables real-time execution in $<1\text{ ms}$ on constrained onboard flight computers (e.g. ARM Cortex-R5 or RISC-V space processors).

---

## 4. Multi-Objective Research Evaluation Metrics

To thoroughly evaluate SAViCS against single-metric heuristics (FIFO, EDF, Image Priority, Contact-Aware, Security-Aware), 7 comprehensive metrics must be reported:

1.  **Delivered Importance (Complete Tiles)**: Sum of importance scores for tiles whose fragments were *completely* received and verified.
2.  **Authenticated Image Coverage (\%)**: Percentage of the 2D image raster that is fully verified and reconstructed:
    $$\text{Coverage} = \frac{\sum_{k=1}^M y_k \cdot \text{Area}(\tau_k)}{\text{Total Image Area}} \times 100\%$$
3.  **Tile Completion Rate (\%)**: Ratio of fully completed tiles to total image tiles:
    $$\text{TCR} = \frac{\sum_{k=1}^M y_k}{M} \times 100\%$$
4.  **Deadline Success Rate (\%)**: Fraction of delivered fragments that arrived before their deadline $d_i$.
5.  **Contact Utilization (\%)**: Fraction of contact window byte capacity effectively occupied by transmissions.
6.  **Payload Goodput (\%)**: Ratio of useful unencrypted image bytes delivered to total wire bytes transmitted:
    $$\text{Goodput} = \frac{\sum_{i \in \text{Delivered}} S_i}{\sum_{i \in \text{Delivered}} W_i} \times 100\%$$
7.  **Retransmission Waste Cost**: Bytes transmitted for fragments whose parent tiles were ultimately abandoned or incomplete.
