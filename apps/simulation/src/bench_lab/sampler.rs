use crate::bench_lab::stats::{summarize, TickStats};
use crate::Simulation;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PhaseSamples {
    pub wall_total: TickStats,
    pub total_tick: TickStats,
    pub perception: TickStats,
    pub steering: TickStats,
    pub movement: TickStats,
    pub spatial_grid_rebuild: TickStats,
    pub l1_aggregation: TickStats,
    pub behavior_transition: TickStats,
    pub export_positions: TickStats,
    pub cells_queried: TickStats,
}

pub fn sample_ticks(sim: &mut Simulation, warmup: usize, samples: usize, dt: f32) -> PhaseSamples {
    let samples = samples.max(1);

    for _ in 0..warmup {
        sim.update(dt);
    }

    let mut wall = Vec::with_capacity(samples);
    let mut total = Vec::with_capacity(samples);
    let mut perception = Vec::with_capacity(samples);
    let mut steering = Vec::with_capacity(samples);
    let mut movement = Vec::with_capacity(samples);
    let mut grid = Vec::with_capacity(samples);
    let mut l1 = Vec::with_capacity(samples);
    let mut behavior = Vec::with_capacity(samples);
    let mut export = Vec::with_capacity(samples);
    let mut cells = Vec::with_capacity(samples);

    for _ in 0..samples {
        let start = Instant::now();
        sim.update(dt);
        wall.push(start.elapsed().as_micros() as u64);

        let t = sim.get_system_timings();
        total.push(t.total_tick_us);
        perception.push(t.perception_us);
        steering.push(t.steering_us);
        movement.push(t.movement_us);
        grid.push(t.spatial_grid_rebuild_us);
        l1.push(t.l1_aggregation_us);
        behavior.push(t.behavior_transition_us);
        export.push(t.export_positions_us);
        cells.push(t.cells_queried_total);
    }

    PhaseSamples {
        wall_total: summarize(&wall),
        total_tick: summarize(&total),
        perception: summarize(&perception),
        steering: summarize(&steering),
        movement: summarize(&movement),
        spatial_grid_rebuild: summarize(&grid),
        l1_aggregation: summarize(&l1),
        behavior_transition: summarize(&behavior),
        export_positions: summarize(&export),
        cells_queried: summarize(&cells),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_lab::world::{build_world, Distribution, WorldSpec};

    fn small_world() -> Simulation {
        build_world(&WorldSpec {
            population: 1000,
            seed: 5,
            half_extent_x: 500.0,
            half_extent_y: 500.0,
            distribution: Distribution::Uniform,
        })
    }

    #[test]
    fn sample_ticks_clamps_zero_samples_to_one() {
        let mut sim = small_world();
        let s = sample_ticks(&mut sim, 0, 0, 0.05);
        assert_eq!(s.wall_total.count, 1, "zero samples must clamp to 1 to prevent false budget pass");
    }

    #[test]
    fn sampler_collects_requested_sample_count() {
        let mut sim = small_world();
        let s = sample_ticks(&mut sim, 3, 10, 0.05);
        assert_eq!(s.wall_total.count, 10);
    }

    #[test]
    fn sampler_measures_nonzero_wall_time() {
        let mut sim = small_world();
        let s = sample_ticks(&mut sim, 3, 10, 0.05);
        assert!(s.wall_total.mean > 0.0, "wall clock must register real time");
    }

    #[test]
    #[cfg(feature = "dev-tools")]
    fn sampler_captures_per_phase_under_dev_tools() {
        let mut sim = small_world();
        let s = sample_ticks(&mut sim, 3, 10, 0.05);
        assert!(s.perception.mean > 0.0, "per-phase timings populated with dev-tools");
    }

    #[test]
    #[cfg(feature = "dev-tools")]
    fn sampler_captures_cells_queried_under_dev_tools() {
        let mut sim = small_world();
        let s = sample_ticks(&mut sim, 3, 10, 0.05);
        assert!(s.cells_queried.mean > 0.0, "perception must query L0 cells; signal for range-trim experiments");
    }
}
