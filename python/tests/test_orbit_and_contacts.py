"""
Unit tests for Q-SHIELD Orbital Propagation, Contact Planning, and Link Dynamics.
"""

from datetime import datetime, timedelta, timezone
from qshield_orbit.orbit_propagator import GROUND_STATIONS, LEOPropagator
from qshield_contacts.contact_plan import ContactPlanner
from qshield_network.link_model import LinkConfig, SatelliteRFLink


def test_leo_orbital_propagation():
    propagator = LEOPropagator.sentinel_2a()
    now = datetime(2026, 3, 16, 12, 0, 0, tzinfo=timezone.utc)

    state = propagator.propagate_state(now)

    assert state["satellite"] == "SENTINEL-2A"
    assert 700.0 <= state["altitude_km"] <= 850.0  # Sentinel-2 nominal LEO altitude ~786 km
    assert 7.0 <= state["orbital_speed_km_s"] <= 8.0  # LEO speed ~7.4 km/s
    assert -90.0 <= state["latitude_deg"] <= 90.0
    assert -180.0 <= state["longitude_deg"] <= 180.0


def test_contact_window_generation():
    propagator = LEOPropagator.sentinel_2a()
    station = GROUND_STATIONS["SVALBARD"]  # High polar station, guaranteed frequent passes
    planner = ContactPlanner(propagator, station)

    start = datetime(2026, 3, 16, 0, 0, 0, tzinfo=timezone.utc)
    end = start + timedelta(hours=6)  # 6-hour window

    windows = planner.find_contact_windows(start, end, step_seconds=15)

    assert len(windows) > 0, "Expected at least 1 contact window over 6 hours at Svalbard"
    first_window = windows[0]

    assert first_window.duration_seconds >= 60.0
    assert first_window.max_elevation_deg >= 5.0
    assert first_window.min_slant_range_km < first_window.max_slant_range_km
    assert first_window.avg_propagation_delay_ms > 0.0


def test_satellite_link_simulation():
    config = LinkConfig(nominal_downlink_mbps=150.0, packet_loss_rate=0.0)
    link = SatelliteRFLink(config, seed=123)

    # When disconnected, transmission should fail
    res_down = link.simulate_fragment_transmission(65536, 45.0, 1000.0)
    assert not res_down["delivered"]

    # When connected, transmission should succeed
    link.connect()
    res_up = link.simulate_fragment_transmission(65536, 45.0, 1000.0)
    assert res_up["delivered"]
    assert res_up["latency_ms"] > 0.0
