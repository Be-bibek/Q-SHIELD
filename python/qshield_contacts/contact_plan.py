"""
Q-SHIELD Satellite Contact Window Planning and Slant Range Engine.
Computes line-of-sight visibility, azimuth, elevation, slant range,
and propagation delay for ground stations.
"""

from dataclasses import asdict, dataclass
from datetime import datetime, timedelta, timezone
from typing import List, Optional
import numpy as np
from skyfield.api import load
from qshield_orbit.orbit_propagator import GroundStation, LEOPropagator, GROUND_STATIONS

SPEED_OF_LIGHT_KM_S = 299792.458


@dataclass
class ContactWindow:
    contact_id: str
    satellite: str
    ground_station: str
    start_time_utc: str
    end_time_utc: str
    duration_seconds: float
    max_elevation_deg: float
    min_slant_range_km: float
    max_slant_range_km: float
    avg_propagation_delay_ms: float
    downlink_capacity_mbps: float
    uplink_capacity_mbps: float
    packet_loss_rate: float
    interruption_prob: float

    def to_dict(self) -> dict:
        return asdict(self)


class ContactPlanner:
    """Computes geometric visibility passes and generates actionable contact plans."""

    def __init__(self, propagator: LEOPropagator, ground_station: GroundStation):
        self.propagator = propagator
        self.ground_station = ground_station
        self.ts = load.timescale()
        self.topos = ground_station.to_skyfield_topos()

    def find_contact_windows(
        self,
        start_time: datetime,
        end_time: datetime,
        step_seconds: int = 10,
        downlink_mbps: float = 150.0,
        uplink_mbps: float = 10.0,
        packet_loss_rate: float = 0.015,
        interruption_prob: float = 0.05,
    ) -> List[ContactWindow]:
        """
        Scan time horizon for passes where elevation angle >= ground_station.min_elevation_deg.
        """
        if start_time.tzinfo is None:
            start_time = start_time.replace(tzinfo=timezone.utc)
        if end_time.tzinfo is None:
            end_time = end_time.replace(tzinfo=timezone.utc)

        windows: List[ContactWindow] = []
        current_time = start_time
        min_elev = self.ground_station.min_elevation_deg

        in_contact = False
        contact_start = None
        elevations = []
        ranges = []

        window_idx = 0

        while current_time <= end_time:
            t = self.ts.from_datetime(current_time)
            difference = self.propagator.satellite - self.topos
            topocentric = difference.at(t)
            alt, az, distance = topocentric.altaz()

            alt_deg = float(alt.degrees)
            range_km = float(distance.km)

            if alt_deg >= min_elev:
                if not in_contact:
                    in_contact = True
                    contact_start = current_time
                    elevations = [alt_deg]
                    ranges = [range_km]
                else:
                    elevations.append(alt_deg)
                    ranges.append(range_km)
            else:
                if in_contact:
                    in_contact = False
                    contact_end = current_time
                    duration = (contact_end - contact_start).total_seconds()

                    if duration >= 60.0:  # Ignore trivial sub-minute horizon grazes
                        max_elev = float(np.max(elevations))
                        min_range = float(np.min(ranges))
                        max_range = float(np.max(ranges))
                        avg_delay_ms = float((np.mean(ranges) / SPEED_OF_LIGHT_KM_S) * 1000.0)

                        window_idx += 1
                        contact_id = f"CW_{self.propagator.name}_{self.ground_station.name[:4].upper()}_{window_idx:03d}"

                        windows.append(
                            ContactWindow(
                                contact_id=contact_id,
                                satellite=self.propagator.name,
                                ground_station=self.ground_station.name,
                                start_time_utc=contact_start.isoformat(),
                                end_time_utc=contact_end.isoformat(),
                                duration_seconds=duration,
                                max_elevation_deg=max_elev,
                                min_slant_range_km=min_range,
                                max_slant_range_km=max_range,
                                avg_propagation_delay_ms=avg_delay_ms,
                                downlink_capacity_mbps=downlink_mbps,
                                uplink_capacity_mbps=uplink_mbps,
                                packet_loss_rate=packet_loss_rate,
                                interruption_prob=interruption_prob,
                            )
                        )

            current_time += timedelta(seconds=step_seconds)

        return windows
