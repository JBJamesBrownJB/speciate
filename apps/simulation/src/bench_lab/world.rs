use crate::simulation::creatures::dna::Dna;
use crate::{BehaviorMode, CritBuilder, Simulation, SimulationBuilder};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Distribution {
    Uniform,
    Clustered { clusters: usize, spread: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSpec {
    pub population: usize,
    pub seed: u64,
    pub half_extent_x: f32,
    pub half_extent_y: f32,
    pub distribution: Distribution,
}

pub fn build_world(spec: &WorldSpec) -> Simulation {
    let mut sim = SimulationBuilder::new()
        .set_boundaries(spec.half_extent_x * 2.0, spec.half_extent_y * 2.0)
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
        Distribution::Uniform => Vec::new(),
    };

    for i in 0..spec.population {
        let (x, y) = match spec.distribution {
            Distribution::Uniform => (
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

        let dna = Dna::random_seeded(&mut rng);
        let builder = CritBuilder::new()
            .at(x, y)
            .with_dna(dna)
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Wandering);
        sim.spawn_crit(builder);
    }

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
}
