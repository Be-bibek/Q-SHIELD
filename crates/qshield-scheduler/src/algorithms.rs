//! Implementations of the six baseline and research scheduling algorithms.

use crate::types::{ContactWindowResource, SchedulableFragment, ScheduleResult};
use qshield_core::FragmentId;
use std::collections::{HashMap, HashSet};

pub trait Scheduler {
    fn name(&self) -> &'static str;
    fn schedule(
        &self,
        fragments: &[SchedulableFragment],
        windows: &[ContactWindowResource],
    ) -> ScheduleResult;
}

/// Generic helper to pack ordered candidates into contact windows
fn pack_ordered_into_windows(
    algo_name: &'static str,
    ordered_fragments: &[&SchedulableFragment],
    windows: &[ContactWindowResource],
    mut log: Vec<String>,
) -> ScheduleResult {
    let mut scheduled_ids = Vec::new();
    let mut window_allocations: HashMap<String, Vec<FragmentId>> = HashMap::new();
    let mut total_wire_bytes = 0u64;
    let mut total_useful_bytes = 0u64;
    let mut total_overhead_bytes = 0u64;
    let mut delivered_importance = 0.0f32;
    let mut deadline_met_count = 0usize;

    let mut remaining_capacities: HashMap<String, u64> = windows
        .iter()
        .map(|w| (w.window_id.clone(), w.capacity_bytes))
        .collect();

    let mut scheduled_set = HashSet::new();

    for &frag in ordered_fragments {
        if scheduled_set.contains(&frag.fragment_id) {
            continue;
        }

        // Check if dependencies have been scheduled
        let deps_met = frag
            .dependencies
            .iter()
            .all(|dep_id| scheduled_set.contains(dep_id));

        if !deps_met {
            log.push(format!(
                "[{}] Skip fragment {}: dependencies not yet scheduled",
                algo_name, frag.fragment_id
            ));
            continue;
        }

        // Find earliest window that can accommodate this fragment
        let needed_bytes = frag.total_wire_bytes as u64;
        let mut allocated = false;

        for w in windows {
            let rem = remaining_capacities.get_mut(&w.window_id).unwrap();
            if *rem >= needed_bytes {
                *rem -= needed_bytes;
                scheduled_set.insert(frag.fragment_id);
                scheduled_ids.push(frag.fragment_id);
                window_allocations
                    .entry(w.window_id.clone())
                    .or_default()
                    .push(frag.fragment_id);

                total_wire_bytes += needed_bytes;
                total_useful_bytes += frag.payload_size_bytes as u64;
                total_overhead_bytes += frag.cryptographic_overhead_bytes as u64;
                delivered_importance += frag.tile_importance;

                // Check deadline
                let tx_end_est = w.start_seconds + (needed_bytes as f64 * 8.0 / w.data_rate_bps);
                if tx_end_est <= frag.deadline_seconds {
                    deadline_met_count += 1;
                }

                log.push(format!(
                    "[{}] Allocated fragment {} ({} bytes) to window {}",
                    algo_name, frag.fragment_id, needed_bytes, w.window_id
                ));
                allocated = true;
                break;
            }
        }

        if !allocated {
            log.push(format!(
                "[{}] Dropped fragment {}: insufficient capacity across windows",
                algo_name, frag.fragment_id
            ));
        }
    }

    let unused_capacity: u64 = remaining_capacities.values().sum();
    let deadline_rate = if scheduled_ids.is_empty() {
        1.0
    } else {
        deadline_met_count as f32 / scheduled_ids.len() as f32
    };

    ScheduleResult {
        algorithm_name: algo_name.to_string(),
        scheduled_fragment_ids: scheduled_ids,
        window_allocations,
        total_scheduled_wire_bytes: total_wire_bytes,
        total_useful_payload_bytes: total_useful_bytes,
        total_cryptographic_overhead_bytes: total_overhead_bytes,
        delivered_importance_score: delivered_importance,
        unused_capacity_bytes: unused_capacity,
        deadline_success_rate: deadline_rate,
        decision_log: log,
    }
}

/// A. FIFO (First-In, First-Out) Baseline Scheduler
pub struct FifoScheduler;

impl Scheduler for FifoScheduler {
    fn name(&self) -> &'static str {
        "FIFO"
    }

    fn schedule(
        &self,
        fragments: &[SchedulableFragment],
        windows: &[ContactWindowResource],
    ) -> ScheduleResult {
        let refs: Vec<&SchedulableFragment> = fragments.iter().collect();
        pack_ordered_into_windows(self.name(), &refs, windows, vec!["Starting FIFO schedule".into()])
    }
}

/// B. EDF (Earliest Deadline First) Baseline Scheduler
pub struct EdfScheduler;

impl Scheduler for EdfScheduler {
    fn name(&self) -> &'static str {
        "EDF"
    }

    fn schedule(
        &self,
        fragments: &[SchedulableFragment],
        windows: &[ContactWindowResource],
    ) -> ScheduleResult {
        let mut refs: Vec<&SchedulableFragment> = fragments.iter().collect();
        refs.sort_by(|a, b| {
            a.deadline_seconds
                .partial_cmp(&b.deadline_seconds)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        pack_ordered_into_windows(self.name(), &refs, windows, vec!["Starting EDF schedule".into()])
    }
}

/// C. Image Priority Scheduler (by perceptual tile importance descending)
pub struct ImagePriorityScheduler;

impl Scheduler for ImagePriorityScheduler {
    fn name(&self) -> &'static str {
        "IMAGE_PRIORITY"
    }

    fn schedule(
        &self,
        fragments: &[SchedulableFragment],
        windows: &[ContactWindowResource],
    ) -> ScheduleResult {
        let mut refs: Vec<&SchedulableFragment> = fragments.iter().collect();
        refs.sort_by(|a, b| {
            b.tile_importance
                .partial_cmp(&a.tile_importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        pack_ordered_into_windows(
            self.name(),
            &refs,
            windows,
            vec!["Starting Image Priority schedule".into()],
        )
    }
}

/// D. Contact-Aware Scheduler (prioritizes fragments that fit snugly into windows without splitting)
pub struct ContactAwareScheduler;

impl Scheduler for ContactAwareScheduler {
    fn name(&self) -> &'static str {
        "CONTACT_AWARE"
    }

    fn schedule(
        &self,
        fragments: &[SchedulableFragment],
        windows: &[ContactWindowResource],
    ) -> ScheduleResult {
        let mut refs: Vec<&SchedulableFragment> = fragments.iter().collect();
        // Sort by smaller size first to maximize fragment count within contact boundaries
        refs.sort_by_key(|f| f.total_wire_bytes);
        pack_ordered_into_windows(
            self.name(),
            &refs,
            windows,
            vec!["Starting Contact-Aware schedule".into()],
        )
    }
}

/// E. Security-Overhead-Aware Scheduler (maximizes useful payload per cryptographic byte)
pub struct SecurityOverheadAwareScheduler;

impl Scheduler for SecurityOverheadAwareScheduler {
    fn name(&self) -> &'static str {
        "SECURITY_OVERHEAD_AWARE"
    }

    fn schedule(
        &self,
        fragments: &[SchedulableFragment],
        windows: &[ContactWindowResource],
    ) -> ScheduleResult {
        let mut refs: Vec<&SchedulableFragment> = fragments.iter().collect();
        // Sort by efficiency ratio (payload / wire) descending
        refs.sort_by(|a, b| {
            b.efficiency_ratio()
                .partial_cmp(&a.efficiency_ratio())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        pack_ordered_into_windows(
            self.name(),
            &refs,
            windows,
            vec!["Starting Security-Overhead-Aware schedule".into()],
        )
    }
}

/// F. SAViCS (Security-Aware Verifiable Image Contact Scheduling)
/// Jointly weighs tile importance, urgency/deadline, cryptographic efficiency,
/// retransmission urgency, and verification milestone dependencies.
pub struct SavicsScheduler {
    pub w_importance: f32,
    pub w_urgency: f32,
    pub w_efficiency: f32,
    pub w_retrans: f32,
    pub w_milestone: f32,
}

impl Default for SavicsScheduler {
    fn default() -> Self {
        Self {
            w_importance: 0.35,
            w_urgency: 0.25,
            w_efficiency: 0.15,
            w_retrans: 0.15,
            w_milestone: 0.10,
        }
    }
}

impl Scheduler for SavicsScheduler {
    fn name(&self) -> &'static str {
        "SAViCS"
    }

    fn schedule(
        &self,
        fragments: &[SchedulableFragment],
        windows: &[ContactWindowResource],
    ) -> ScheduleResult {
        let refs: Vec<&SchedulableFragment> = fragments.iter().collect();

        // Find reference max deadline for normalization
        let max_deadline = fragments
            .iter()
            .map(|f| f.deadline_seconds)
            .fold(1.0f64, f64::max);

        // Compute multi-attribute SAViCS utility score for each candidate
        let mut scored_fragments: Vec<(&SchedulableFragment, f32)> = refs
            .into_iter()
            .map(|frag| {
                let urgency = (1.0 - (frag.deadline_seconds / max_deadline).clamp(0.0, 1.0)) as f32;
                let efficiency = frag.efficiency_ratio();
                let retrans_bonus = if frag.is_retransmission { 1.0f32 } else { 0.0f32 };
                let milestone_bonus = if frag.is_milestone_or_final { 1.0f32 } else { 0.0f32 };

                let score = self.w_importance * frag.tile_importance
                    + self.w_urgency * urgency
                    + self.w_efficiency * efficiency
                    + self.w_retrans * retrans_bonus
                    + self.w_milestone * milestone_bonus;

                (frag, score)
            })
            .collect();

        // Sort descending by SAViCS score
        scored_fragments.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let ordered: Vec<&SchedulableFragment> = scored_fragments.into_iter().map(|(f, _)| f).collect();
        pack_ordered_into_windows(
            self.name(),
            &ordered,
            windows,
            vec!["Starting SAViCS multi-attribute optimization schedule".into()],
        )
    }
}
