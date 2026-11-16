"""
Q-SHIELD Dynamic Satellite Link Simulation Model.
Simulates LEO space-to-ground and relay communications:
- Data rate dynamics
- Slant-range propagation delays
- Atmospheric and rain fade attenuation
- Intermittent connection dropouts and interruption recovery
"""

import math
import random
from dataclasses import dataclass


@dataclass
class LinkConfig:
    nominal_downlink_mbps: float = 150.0
    nominal_uplink_mbps: float = 10.0
    packet_loss_rate: float = 0.015
    interruption_prob_per_pass: float = 0.05
    interruption_duration_mean_sec: float = 15.0
    buffer_capacity_mb: float = 512.0


class SatelliteRFLink:
    """Simulates physical and link-layer dynamics of an intermittent LEO contact."""

    def __init__(self, config: LinkConfig, seed: int = 42):
        self.config = config
        self.rng = random.Random(seed)
        self.is_connected = False
        self.interrupted = False

    def connect(self):
        self.is_connected = True
        self.interrupted = False

    def disconnect(self):
        self.is_connected = False
        self.interrupted = False

    def calculate_free_space_path_loss_db(self, frequency_ghz: float, slant_range_km: float) -> float:
        """FSPL = 20 log10(d) + 20 log10(f) + 92.45 dB."""
        if slant_range_km <= 0 or frequency_ghz <= 0:
            return 0.0
        return 20.0 * math.log10(slant_range_km) + 20.0 * math.log10(frequency_ghz) + 92.45

    def simulate_fragment_transmission(
        self,
        fragment_bytes: int,
        elevation_deg: float,
        slant_range_km: float,
    ) -> dict:
        """
        Simulates transmission of an encrypted fragment over the LEO link.
        Returns transmission metrics including delivered status, latency, and whether an interruption occurred.
        """
        if not self.is_connected:
            return {
                "delivered": False,
                "reason": "LINK_DOWN",
                "latency_ms": 0.0,
                "interrupted": False,
            }

        # Check for sudden contact interruption (e.g. tracking glitch, antenna handover)
        if not self.interrupted and self.rng.random() < (self.config.interruption_prob_per_pass * 0.01):
            self.interrupted = True
            return {
                "delivered": False,
                "reason": "CONTACT_INTERRUPTED",
                "latency_ms": 0.0,
                "interrupted": True,
            }

        if self.interrupted:
            return {
                "delivered": False,
                "reason": "LINK_INTERRUPTED",
                "latency_ms": 0.0,
                "interrupted": True,
            }

        # Lower elevation angles suffer greater atmospheric loss and higher packet loss
        elevation_factor = max(0.2, min(1.0, math.sin(math.radians(max(5.0, elevation_deg)))))
        effective_rate_mbps = self.config.nominal_downlink_mbps * elevation_factor

        # Random packet loss check
        loss_prob = self.config.packet_loss_rate * (1.5 - 0.5 * elevation_factor)
        if self.rng.random() < loss_prob:
            return {
                "delivered": False,
                "reason": "PACKET_LOSS",
                "latency_ms": 0.0,
                "interrupted": False,
            }

        # Compute delay: transmission time + propagation time
        tx_time_ms = (fragment_bytes * 8.0) / (effective_rate_mbps * 1e6) * 1000.0
        prop_time_ms = (slant_range_km / 299792.458) * 1000.0
        total_latency_ms = tx_time_ms + prop_time_ms

        return {
            "delivered": True,
            "reason": "DELIVERED",
            "latency_ms": total_latency_ms,
            "effective_rate_mbps": effective_rate_mbps,
            "interrupted": False,
        }
