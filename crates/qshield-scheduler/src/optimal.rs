//! Reference Optimal Branch-and-Bound Solver for Small Benchmark Instances.

use crate::types::{ContactWindowResource, SchedulableFragment, ScheduleResult};
use qshield_core::FragmentId;
use std::collections::{HashMap, HashSet};

/// Exact reference solver evaluating all feasible dependency-valid allocations
pub struct OptimalReferenceSolver;

impl OptimalReferenceSolver {
    pub fn solve(
        fragments: &[SchedulableFragment],
        windows: &[ContactWindowResource],
    ) -> ScheduleResult {
        let total_capacity: u64 = windows.iter().map(|w| w.capacity_bytes).sum();
        let n = fragments.len();

        if n > 18 {
            panic!("OptimalReferenceSolver is exponential O(2^N); use heuristics for N > 18");
        }

        let mut best_importance = 0.0f32;
        let mut best_subset: Vec<usize> = Vec::new();

        // Enumerate subsets respecting dependency topology
        for mask in 0..(1usize << n) {
            let mut current_bytes = 0u64;
            let mut current_importance = 0.0f32;
            let mut selected_indices = Vec::new();
            let mut selected_ids = HashSet::new();

            for i in 0..n {
                if (mask & (1 << i)) != 0 {
                    selected_indices.push(i);
                    selected_ids.insert(fragments[i].fragment_id);
                    current_bytes += fragments[i].total_wire_bytes as u64;
                    current_importance += fragments[i].tile_importance;
                }
            }

            if current_bytes > total_capacity {
                continue;
            }

            // Check if all dependencies are satisfied
            let mut valid = true;
            for &idx in &selected_indices {
                for dep in &fragments[idx].dependencies {
                    if !selected_ids.contains(dep) {
                        valid = false;
                        break;
                    }
                }
                if !valid {
                    break;
                }
            }

            if valid && current_importance > best_importance {
                best_importance = current_importance;
                best_subset = selected_indices;
            }
        }

        // Pack best subset into windows
        let mut scheduled_ids = Vec::new();
        let mut window_allocations: HashMap<String, Vec<FragmentId>> = HashMap::new();
        let mut remaining_cap: HashMap<String, u64> =
            windows.iter().map(|w| (w.window_id.clone(), w.capacity_bytes)).collect();

        let mut total_wire = 0u64;
        let mut total_payload = 0u64;
        let mut total_overhead = 0u64;

        for idx in best_subset {
            let frag = &fragments[idx];
            let needed = frag.total_wire_bytes as u64;

            for w in windows {
                let rem = remaining_cap.get_mut(&w.window_id).unwrap();
                if *rem >= needed {
                    *rem -= needed;
                    scheduled_ids.push(frag.fragment_id);
                    window_allocations.entry(w.window_id.clone()).or_default().push(frag.fragment_id);
                    total_wire += needed;
                    total_payload += frag.payload_size_bytes as u64;
                    total_overhead += frag.cryptographic_overhead_bytes as u64;
                    break;
                }
            }
        }

        let unused: u64 = remaining_cap.values().sum();

        ScheduleResult {
            algorithm_name: "OPTIMAL_REFERENCE".to_string(),
            scheduled_fragment_ids: scheduled_ids,
            window_allocations,
            total_scheduled_wire_bytes: total_wire,
            total_useful_payload_bytes: total_payload,
            total_cryptographic_overhead_bytes: total_overhead,
            delivered_importance_score: best_importance,
            unused_capacity_bytes: unused,
            deadline_success_rate: 1.0,
            decision_log: vec!["Solved via exhaustive Branch-and-Bound".into()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::{FifoScheduler, SavicsScheduler, Scheduler};
    use qshield_core::TileId;

    #[test]
    fn test_all_schedulers_and_optimal_solver_comparison() {
        let windows = vec![ContactWindowResource {
            window_id: "CW_SVALBARD_01".into(),
            start_seconds: 0.0,
            end_seconds: 600.0,
            duration_seconds: 600.0,
            capacity_bytes: 50_000, // Constrained capacity
            data_rate_bps: 150_000_000.0,
        }];

        let mut fragments = Vec::new();
        for i in 0..8 {
            fragments.push(SchedulableFragment {
                fragment_id: FragmentId(i),
                tile_id: TileId(i % 4),
                payload_size_bytes: 10_000,
                cryptographic_overhead_bytes: 128,
                total_wire_bytes: 10_128,
                tile_importance: if i % 2 == 0 { 0.95 } else { 0.20 }, // Mixed importance
                deadline_seconds: (i as f64 + 1.0) * 100.0,
                is_milestone_or_final: i == 7,
                dependencies: Vec::new(),
                is_retransmission: i == 0,
            });
        }

        let fifo = FifoScheduler;
        let savics = SavicsScheduler::default();

        let res_fifo = fifo.schedule(&fragments, &windows);
        let res_savics = savics.schedule(&fragments, &windows);
        let res_opt = OptimalReferenceSolver::solve(&fragments, &windows);

        assert!(res_savics.total_scheduled_wire_bytes <= 50_000);
        assert!(res_fifo.total_scheduled_wire_bytes <= 50_000);
        assert!(res_opt.total_scheduled_wire_bytes <= 50_000);

        // SAViCS prioritizes high importance fragments over pure FIFO
        assert!(res_savics.delivered_importance_score >= res_fifo.delivered_importance_score);
        // Optimal is upper bound
        assert!(res_opt.delivered_importance_score >= res_savics.delivered_importance_score);
    }
}
