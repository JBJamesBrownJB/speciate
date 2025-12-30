use crate::simulation::core::components::{BodySize, Position, Velocity};
use crate::simulation::perception::{L1Classification, L1Vision, NeighborCache};
use bevy_ecs::prelude::*;
use rayon::prelude::*;

use super::DriveContributions;

#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;

/// Vision Drive System: Generates drive contributions from L1 vision data.
///
/// For each creature, iterates L1Vision and:
/// - THREAT → push_flee (with velocity-based urgency)
/// - PREY → push_approach
/// - CROWDED → push_disperse (away from crowded areas)
/// - EMPTY → mild push_disperse (toward empty space)
pub fn vision_drive_system(
    mut query: Query<(
        &Position,
        &Velocity,
        &BodySize,
        &L1Vision,
        &NeighborCache,
        &mut DriveContributions,
    )>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "vision_drive");

    let mut entities: Vec<_> = query.iter_mut().collect();

    entities
        .par_iter_mut()
        .with_min_len(64)
        .for_each(|(pos, vel, size, l1_vision, neighbor_cache, contributions)| {
            process_vision_drives(pos, vel, size, l1_vision, neighbor_cache, contributions);
        });
}

/// Process vision-based drives for a single creature.
fn process_vision_drives(
    pos: &Position,
    _vel: &Velocity,
    size: &BodySize,
    l1_vision: &L1Vision,
    neighbor_cache: &NeighborCache,
    contributions: &mut DriveContributions,
) {
    let _self_mass = size.mass();

    for entry in l1_vision.iter() {
        let dir = (entry.direction_x, entry.direction_y);

        match entry.classification {
            L1Classification::Threat => {
                let urgency = calculate_threat_urgency(pos, dir, neighbor_cache);
                contributions.push_flee(negate(dir), urgency);
            }
            L1Classification::Prey => {
                contributions.push_approach(dir, 1.0);
            }
            L1Classification::Crowded => {
                contributions.push_disperse(negate(dir), 0.5);
            }
            L1Classification::Empty => {
                contributions.push_disperse(dir, 0.3);
            }
        }
    }
}

/// Calculate threat urgency based on nearby neighbors' velocity.
/// - Charging toward us (>2 m/s) → 1.0 (maximum urgency)
/// - Stationary → 0.5
/// - Retreating → 0.2
fn calculate_threat_urgency(
    my_pos: &Position,
    threat_dir: (f32, f32),
    neighbor_cache: &NeighborCache,
) -> f32 {
    let mut max_urgency: f32 = 0.5;

    for neighbor in neighbor_cache.iter_neighbors() {
        let to_neighbor = (neighbor.x - my_pos.x, neighbor.y - my_pos.y);
        let dist = (to_neighbor.0 * to_neighbor.0 + to_neighbor.1 * to_neighbor.1).sqrt();
        if dist < 0.001 {
            continue;
        }

        let neighbor_dir = (to_neighbor.0 / dist, to_neighbor.1 / dist);

        let dot = threat_dir.0 * neighbor_dir.0 + threat_dir.1 * neighbor_dir.1;
        if dot < 0.5 {
            continue;
        }

        let vx = neighbor.vx;
        let vy = neighbor.vy;
        let speed = (vx * vx + vy * vy).sqrt();

        if speed < 0.1 {
            max_urgency = max_urgency.max(0.5);
            continue;
        }

        let vel_dir = (vx / speed, vy / speed);
        let closing = -(vel_dir.0 * neighbor_dir.0 + vel_dir.1 * neighbor_dir.1);

        if closing > 0.5 && speed > 2.0 {
            max_urgency = 1.0;
            break;
        } else if closing > 0.0 {
            max_urgency = max_urgency.max(0.7);
        } else {
            max_urgency = max_urgency.max(0.2);
        }
    }

    max_urgency
}

/// Negate a direction vector.
#[inline]
fn negate(dir: (f32, f32)) -> (f32, f32) {
    (-dir.0, -dir.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::perception::{L1Classification, L1VisionEntry};

    #[test]
    fn test_threat_generates_flee_contribution() {
        let mut contributions = DriveContributions::default();
        let pos = Position { x: 0.0, y: 0.0 };
        let vel = Velocity { vx: 0.0, vy: 0.0 };
        let size = BodySize::new(1.0);
        let neighbor_cache = NeighborCache::new();

        let mut l1_vision = L1Vision::new();
        l1_vision.push(L1VisionEntry {
            cell_idx: 0,
            classification: L1Classification::Threat,
            _pad: [0; 3],
            direction_x: 1.0,
            direction_y: 0.0,
        });

        process_vision_drives(&pos, &vel, &size, &l1_vision, &neighbor_cache, &mut contributions);

        assert!(contributions.has_flee(), "Threat should generate flee contribution");
        assert_eq!(contributions.flee_count, 1);

        let flee = contributions.iter_flee().next().unwrap();
        assert!(flee.direction.0 < 0.0, "Flee should be AWAY from threat (negative x)");
    }

    #[test]
    fn test_prey_generates_approach_contribution() {
        let mut contributions = DriveContributions::default();
        let pos = Position { x: 0.0, y: 0.0 };
        let vel = Velocity { vx: 0.0, vy: 0.0 };
        let size = BodySize::new(1.0);
        let neighbor_cache = NeighborCache::new();

        let mut l1_vision = L1Vision::new();
        l1_vision.push(L1VisionEntry {
            cell_idx: 0,
            classification: L1Classification::Prey,
            _pad: [0; 3],
            direction_x: 1.0,
            direction_y: 0.0,
        });

        process_vision_drives(&pos, &vel, &size, &l1_vision, &neighbor_cache, &mut contributions);

        assert!(contributions.has_approach(), "Prey should generate approach contribution");
        assert_eq!(contributions.approach_count, 1);

        let approach = contributions.iter_approach().next().unwrap();
        assert!(approach.direction.0 > 0.0, "Approach should be TOWARD prey");
    }

    #[test]
    fn test_crowded_generates_disperse_away() {
        let mut contributions = DriveContributions::default();
        let pos = Position { x: 0.0, y: 0.0 };
        let vel = Velocity { vx: 0.0, vy: 0.0 };
        let size = BodySize::new(1.0);
        let neighbor_cache = NeighborCache::new();

        let mut l1_vision = L1Vision::new();
        l1_vision.push(L1VisionEntry {
            cell_idx: 0,
            classification: L1Classification::Crowded,
            _pad: [0; 3],
            direction_x: 1.0,
            direction_y: 0.0,
        });

        process_vision_drives(&pos, &vel, &size, &l1_vision, &neighbor_cache, &mut contributions);

        assert!(contributions.has_disperse(), "Crowded should generate disperse");
        let disperse = contributions.iter_disperse().next().unwrap();
        assert!(disperse.direction.0 < 0.0, "Disperse should be AWAY from crowded");
    }

    #[test]
    fn test_empty_generates_disperse_toward() {
        let mut contributions = DriveContributions::default();
        let pos = Position { x: 0.0, y: 0.0 };
        let vel = Velocity { vx: 0.0, vy: 0.0 };
        let size = BodySize::new(1.0);
        let neighbor_cache = NeighborCache::new();

        let mut l1_vision = L1Vision::new();
        l1_vision.push(L1VisionEntry {
            cell_idx: 0,
            classification: L1Classification::Empty,
            _pad: [0; 3],
            direction_x: 1.0,
            direction_y: 0.0,
        });

        process_vision_drives(&pos, &vel, &size, &l1_vision, &neighbor_cache, &mut contributions);

        assert!(contributions.has_disperse(), "Empty should generate disperse");
        let disperse = contributions.iter_disperse().next().unwrap();
        assert!(disperse.direction.0 > 0.0, "Disperse should be TOWARD empty space");
    }

    #[test]
    fn test_multiple_perceptions_accumulate() {
        let mut contributions = DriveContributions::default();
        let pos = Position { x: 0.0, y: 0.0 };
        let vel = Velocity { vx: 0.0, vy: 0.0 };
        let size = BodySize::new(1.0);
        let neighbor_cache = NeighborCache::new();

        let mut l1_vision = L1Vision::new();
        l1_vision.push(L1VisionEntry {
            cell_idx: 0,
            classification: L1Classification::Threat,
            _pad: [0; 3],
            direction_x: 1.0,
            direction_y: 0.0,
        });
        l1_vision.push(L1VisionEntry {
            cell_idx: 1,
            classification: L1Classification::Prey,
            _pad: [0; 3],
            direction_x: -1.0,
            direction_y: 0.0,
        });
        l1_vision.push(L1VisionEntry {
            cell_idx: 2,
            classification: L1Classification::Crowded,
            _pad: [0; 3],
            direction_x: 0.0,
            direction_y: 1.0,
        });

        process_vision_drives(&pos, &vel, &size, &l1_vision, &neighbor_cache, &mut contributions);

        assert_eq!(contributions.flee_count, 1);
        assert_eq!(contributions.approach_count, 1);
        assert_eq!(contributions.disperse_count, 1);
    }
}
