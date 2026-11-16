"""
Q-SHIELD Satellite Orbital Propagation Engine.
Uses sgp4 and skyfield for high-fidelity LEO orbit determination.
"""

from dataclasses import dataclass
from datetime import datetime, timezone
from typing import List, Tuple
import numpy as np
from skyfield.api import EarthSatellite, Topos, load, wgs84


# Realistic Two-Line Elements (TLEs) for benchmark Earth-Observation Satellites
SENTINEL_2A_TLE = (
    "SENTINEL-2A",
    "1 40697U 15028A   26075.50000000  .00000120  00000+0  34500-4 0  9993",
    "2 40697  98.5680 142.1200 0001200  92.4500 267.6800 14.30820000561234",
)

LANDSAT_9_TLE = (
    "LANDSAT-9",
    "1 49260U 21088A   26075.50000000  .00000085  00000+0  25400-4 0  9997",
    "2 49260  98.2040 140.4500 0001150  88.3200 271.8100 14.57110000234567",
)


@dataclass
class GroundStation:
    name: str
    latitude_deg: float
    longitude_deg: float
    elevation_m: float
    min_elevation_deg: float = 5.0

    def to_skyfield_topos(self):
        return wgs84.latlon(self.latitude_deg, self.longitude_deg, elevation_m=self.elevation_m)


# Standard benchmark polar and mid-latitude Earth Station ground sites
GROUND_STATIONS = {
    "SVALBARD": GroundStation("Svalbard (SG3)", 78.2297, 15.4077, 400.0, 5.0),
    "TROMSO": GroundStation("Tromsø Satellite Station", 69.6627, 18.9409, 130.0, 5.0),
    "FAIRBANKS": GroundStation("Fairbanks Command & Data", 64.8589, -147.8494, 180.0, 5.0),
    "MCMURDO": GroundStation("McMurdo Antarctic Station", -77.8463, 166.6682, 100.0, 5.0),
}


class LEOPropagator:
    """Propagator calculating satellite state vectors, ground tracks, and positions."""

    def __init__(self, name: str, line1: str, line2: str):
        self.ts = load.timescale()
        self.satellite = EarthSatellite(line1, line2, name, self.ts)
        self.name = name

    @classmethod
    def sentinel_2a(cls) -> "LEOPropagator":
        return cls(*SENTINEL_2A_TLE)

    @classmethod
    def landsat_9(cls) -> "LEOPropagator":
        return cls(*LANDSAT_9_TLE)

    def propagate_state(self, dt: datetime) -> dict:
        """Calculate satellite ephemeris, altitude, latitude, and longitude at a given UTC time."""
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        t = self.ts.from_datetime(dt)

        geocentric = self.satellite.at(t)
        subpoint = wgs84.subpoint(geocentric)

        pos_km = geocentric.position.km
        vel_km_s = geocentric.velocity.km_per_s

        speed_km_s = float(np.linalg.norm(vel_km_s))

        return {
            "satellite": self.name,
            "timestamp_utc": dt.isoformat(),
            "latitude_deg": float(subpoint.latitude.degrees),
            "longitude_deg": float(subpoint.longitude.degrees),
            "altitude_km": float(subpoint.elevation.km),
            "position_eci_km": [float(p) for p in pos_km],
            "velocity_eci_km_s": [float(v) for v in vel_km_s],
            "orbital_speed_km_s": speed_km_s,
        }
