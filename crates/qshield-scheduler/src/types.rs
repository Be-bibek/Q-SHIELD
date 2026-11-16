//! Domain models for contact scheduling, resources, and decisions.

use qshield_core::{FragmentId, TileId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Candidate fragment evaluated by scheduler
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchedulableFragment {
    pub fragment_id: FragmentId,
    pub tile_id: TileId,
    pub payload_size_bytes: usize,
    pub cryptographic_overhead_bytes: usize,
    pub total_wire_bytes: usize,
    pub tile_importance: f32, // [0.0, 1.0]
    pub deadline_seconds: f64,
    pub is_milestone_or_final: bool,
    pub dependencies: Vec<FragmentId>,
    pub is_retransmission: bool,
}

impl SchedulableFragment {
    pub fn efficiency_ratio(&self) -> f32 {
        if self.total_wire_bytes == 0 {
            0.0
        } else {
            self.payload_size_bytes as f32 / self.total_wire_bytes as f32
        }
    }
}

/// Simulated contact window resource budget
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactWindowResource {
    pub window_id: String,
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub duration_seconds: f64,
    pub capacity_bytes: u64,
    pub data_rate_bps: f64,
}

/// Outcome of a scheduling execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleResult {
    pub algorithm_name: String,
    pub scheduled_fragment_ids: Vec<FragmentId>,
    pub window_allocations: HashMap<String, Vec<FragmentId>>,
    pub total_scheduled_wire_bytes: u64,
    pub total_useful_payload_bytes: u64,
    pub total_cryptographic_overhead_bytes: u64,
    pub delivered_importance_score: f32,
    pub unused_capacity_bytes: u64,
    pub deadline_success_rate: f32,
    pub decision_log: Vec<String>,
}
