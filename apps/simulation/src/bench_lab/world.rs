use crate::simulation::creatures::dna::{Dna, SizeDistributionParams};
use crate::{BehaviorMode, CritBuilder, Simulation, SimulationBuilder};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Distribution {
    Uniform,
    Clustered { clusters: usize, spread: f32 },
    RealisticSize { median_meters: f32, sigma_log10: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSpec {
    pub population: usize,
    pub seed: u64,
    pub half_extent_x: f32, // full-world half-extent: creatures spawn across ±half_extent, bounds sit at the edges
    pub half_extent_y: f32, // full-world half-extent: creatures spawn across ±half_extent, bounds sit at the edges
    pub distribution: Distribution,
}

pub fn build_world(spec: &WorldSpec) -> Simulation {
    let mut sim = SimulationBuilder::new()
        .set_boundaries(spec.half_extent_x, spec.half_extent_y)
        .build();
    let mut rng = StdRng::seed_from_u64(spec.seed);

    let centers: Vec<(f32, f32)> = match spec.distribution {
        Distribution::Clustered { clusters, .. } => (0..clusters.max(1))
            .map(|_| {
                let cx = (rng.gen::<f32>() - 0.5) * (spec.half_extent_x * 2.0);
                let cy = (rng.gen::<f32>() - 0.5) * (spec.half_extent_y * 2.0);
                (cx, cy)
            })
            .collect(),
        Distribution::Uniform | Distribution::RealisticSize { .. } => Vec::new(),
    };

    let realistic_params = match spec.distribution {
        Distribution::RealisticSize { median_meters, sigma_log10 } => {
            Some(SizeDistributionParams::new(median_meters, sigma_log10))
        }
        _ => None,
    };

    for i in 0..spec.population {
        let (x, y) = match spec.distribution {
            Distribution::Uniform | Distribution::RealisticSize { .. } => (
                (rng.gen::<f32>() - 0.5) * (spec.half_extent_x * 2.0),
                (rng.gen::<f32>() - 0.5) * (spec.half_extent_y * 2.0),
            ),
            Distribution::Clustered { spread, .. } => {
                let (cx, cy) = centers[i % centers.len()];
                (
                    cx + (rng.gen::<f32>() - 0.5) * spread * 2.0,
                    cy + (rng.gen::<f32>() - 0.5) * spread * 2.0,
                )
            }
        };

        let dna = if let Some(ref params) = realistic_params {
            Dna::random_realistic_seeded(&mut rng, params)
        } else {
            Dna::random_seeded(&mut rng)
        };
        let builder = CritBuilder::new()
            .at(x, y)
            .with_dna(dna)
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Wandering);
        sim.spawn_crit(builder);
    }

    sim.set_system_frequency("perception", 8);
    sim.set_system_frequency("behavior", 8);

    sim
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(pop: usize, seed: u64) -> WorldSpec {
        WorldSpec {
            population: pop,
            seed,
            half_extent_x: 2500.0,
            half_extent_y: 2000.0,
            distribution: Distribution::Uniform,
        }
    }

    #[test]
    fn build_world_spawns_requested_population() {
        let sim = build_world(&spec(1000, 1));
        assert_eq!(sim.creature_count(), 1000);
    }

    #[test]
    fn build_world_is_deterministic() {
        let a = build_world(&spec(500, 99));
        let b = build_world(&spec(500, 99));
        assert_eq!(a.snapshot_creatures(), b.snapshot_creatures());
    }

    #[test]
    fn different_seeds_build_different_worlds() {
        let a = build_world(&spec(500, 1));
        let b = build_world(&spec(500, 2));
        assert_ne!(a.snapshot_creatures(), b.snapshot_creatures());
    }

    #[test]
    fn clustered_distribution_builds() {
        let s = WorldSpec {
            population: 800,
            seed: 3,
            half_extent_x: 2500.0,
            half_extent_y: 2000.0,
            distribution: Distribution::Clustered { clusters: 8, spread: 100.0 },
        };
        let sim = build_world(&s);
        assert_eq!(sim.creature_count(), 800);
    }

    #[test]
    fn clustered_distribution_positions_are_localized() {
        let spread = 50.0_f32;
        let s = WorldSpec {
            population: 120,
            seed: 42,
            half_extent_x: 500.0,
            half_extent_y: 500.0,
            distribution: Distribution::Clustered { clusters: 4, spread },
        };
        let sim = build_world(&s);
        let crits = sim.snapshot_creatures();
        assert_eq!(crits.len(), 120);

        let cluster_0: Vec<_> = crits.iter().step_by(4).collect();
        let cluster_1: Vec<_> = crits.iter().skip(1).step_by(4).collect();

        let max_spread_in_cluster = |members: &[&(u32, f32, f32, f32, f32)]| -> f32 {
            members.iter().flat_map(|a| {
                members.iter().map(move |b| {
                    ((a.1 - b.1).powi(2) + (a.2 - b.2).powi(2)).sqrt()
                })
            }).fold(0.0_f32, f32::max)
        };

        let spread_0 = max_spread_in_cluster(&cluster_0);
        let spread_1 = max_spread_in_cluster(&cluster_1);

        assert!(
            spread_0 <= spread * 2.0 * 2.0_f32.sqrt(),
            "cluster 0 diameter {spread_0} exceeds 2×spread diagonal bound"
        );
        assert!(
            spread_1 <= spread * 2.0 * 2.0_f32.sqrt(),
            "cluster 1 diameter {spread_1} exceeds 2×spread diagonal bound"
        );

        let all_x: Vec<f32> = crits.iter().map(|c| c.1).collect();
        let total_x_span = all_x.iter().cloned().fold(f32::MIN, f32::max)
            - all_x.iter().cloned().fold(f32::MAX, f32::min);
        assert!(
            total_x_span > spread * 2.0,
            "clusters must be spread apart (total x-span {total_x_span} should exceed 2×spread)"
        );
    }

    #[test]
    fn world_bounds_equal_spawn_extent_full_world() {
        let spec = WorldSpec {
            population: 1000,
            seed: 1,
            half_extent_x: 5000.0,
            half_extent_y: 5000.0,
            distribution: Distribution::Uniform,
        };
        let sim = build_world(&spec);
        assert_eq!(sim.get_boundaries(), (-5000.0, 5000.0, -5000.0, 5000.0));
    }

    #[test]
    fn build_world_throttles_perception_and_behavior_to_8() {
        let spec = WorldSpec {
            population: 100,
            seed: 1,
            half_extent_x: 5000.0,
            half_extent_y: 5000.0,
            distribution: Distribution::Uniform,
        };
        let sim = build_world(&spec);
        assert_eq!(sim.system_frequency("perception"), 8);
        assert_eq!(sim.system_frequency("behavior"), 8);
    }

    #[test]
    fn creatures_fill_the_whole_world() {
        let spec = WorldSpec {
            population: 5000,
            seed: 2,
            half_extent_x: 5000.0,
            half_extent_y: 5000.0,
            distribution: Distribution::Uniform,
        };
        let sim = build_world(&spec);
        let crits = sim.snapshot_creatures();
        let max_x = crits.iter().map(|c| c.1).fold(f32::MIN, f32::max);
        let min_x = crits.iter().map(|c| c.1).fold(f32::MAX, f32::min);
        let max_y = crits.iter().map(|c| c.2).fold(f32::MIN, f32::max);
        let min_y = crits.iter().map(|c| c.2).fold(f32::MAX, f32::min);
        assert!(max_x > 4000.0 && min_x < -4000.0, "x must span the full world");
        assert!(max_y > 4000.0 && min_y < -4000.0, "y must span the full world");
    }
}
