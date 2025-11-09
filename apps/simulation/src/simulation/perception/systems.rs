//! Perception systems for spatial awareness
//!
//! Updates what each creature can detect in its surroundings.
//!
//! # Current Implementation: Naive O(n²)
//! Every creature checks distance to every other creature.
//! This is intentionally simple for baseline benchmarking.
//!
//! # Future Optimization (Post-Benchmark)
//! After measuring performance impact, we'll implement spatial hash
//! for O(n) queries with localized lookups.
//!
//! # Threading
//! This system ONLY reads `Position` (immutable), so it can run in parallel
//! with other read-only systems. Bevy will auto-parallelize.

use super::components::*;
use crate::simulation::core::components::{BodySize, Position};
use bevy_ecs::prelude::*;

/// Updates perception for all creatures (naive O(n²) implementation)
///
/// For each creature with `Perception` component:
/// 1. Clear cached neighbor list
/// 2. Check distance to all other creatures
/// 3. Add entities within perception range to neighbor list
///
/// # Performance
/// - **Complexity:** O(n²) where n = creature count
/// - **Expected:** ~0.1ms per 100 creatures (baseline - to be measured)
/// - **Threading:** Can run in parallel (only reads Position)
///
/// # System Ordering
/// Must run BEFORE behavior systems (seek, avoid, etc.) so they have
/// fresh perception data.
///
/// # Future Optimization
/// Replace with spatial hash for O(n) performance:
/// ```ignore
/// for (entity, pos, mut perception) in query.iter_mut() {
///     let candidates = spatial_hash.query_radius(pos, perception.range);
///     perception.nearby = candidates; // Much faster!
/// }
/// ```
///
/// TODO: Implement spatial hash after baseline benchmark (Sprint 6 Phase 8+)
pub fn update_perception_system(
    mut query: Query<(Entity, &Position, &BodySize, &mut Perception)>,
) {
    // Collect all positions and sizes first (avoid borrow checker issues)
    let creatures: Vec<(Entity, Position, BodySize)> = query
        .iter()
        .map(|(entity, pos, size, _)| (entity, *pos, *size))
        .collect();

    // For each creature, check edge-to-edge distance to all others
    for (entity, pos, size, mut perception) in query.iter_mut() {
        perception.clear();

        let range_sq = perception.range * perception.range;
        let self_radius = size.radius();

        // Check distance to all other creatures
        for (other_entity, other_pos, other_size) in &creatures {
            if entity == *other_entity {
                continue;
            }

            // Calculate center-to-center distance
            let dx = other_pos.x - pos.x;
            let dy = other_pos.y - pos.y;
            let center_dist_sq = dx * dx + dy * dy;

            // Edge-to-edge distance = center_distance - radius_self - radius_other
            // For performance, check if center distance is close enough before sqrt
            let other_radius = other_size.radius();
            let combined_radii = self_radius + other_radius;
            let combined_radii_sq = combined_radii * combined_radii;

            // Early rejection: if centers are way too far apart, skip sqrt
            if center_dist_sq > (perception.range + combined_radii).powi(2) {
                continue;
            }

            let center_dist = center_dist_sq.sqrt();
            let edge_dist = center_dist - combined_radii;

            // Add to neighbor list if edge-to-edge distance within range
            if edge_dist <= perception.range {
                perception.add_neighbor(*other_entity);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perception_detects_nearby_entities() {
        let mut world = World::new();

        // Create three creatures
        let crit1 = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Perception::new(10.0), // 10m range
            ))
            .id();

        let crit2 = world
            .spawn((
                Position { x: 5.0, y: 0.0 }, // 5m away - within range
                Perception::new(10.0),
            ))
            .id();

        let crit3 = world
            .spawn((
                Position { x: 20.0, y: 0.0 }, // 20m away from crit1, 15m from crit2 - out of range for both
                Perception::new(10.0),
            ))
            .id();

        // Run perception system manually (test context)
        let positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(&world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query::<(Entity, &Position, &mut Perception)>();
        for (entity, pos, mut perception) in query.iter_mut(&mut world) {
            perception.clear();
            let range_sq = perception.range * perception.range;

            for (other_entity, other_pos) in &positions {
                if entity == *other_entity {
                    continue;
                }
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= range_sq {
                    perception.add_neighbor(*other_entity);
                }
            }
        }

        // Check crit1 perception
        let perception1 = world.get::<Perception>(crit1).unwrap();
        assert_eq!(perception1.neighbor_count(), 1); // Should see crit2 only
        assert!(perception1.nearby.contains(&crit2));
        assert!(!perception1.nearby.contains(&crit3));

        // Check crit2 perception
        let perception2 = world.get::<Perception>(crit2).unwrap();
        assert_eq!(perception2.neighbor_count(), 1); // Should see crit1 only
        assert!(perception2.nearby.contains(&crit1));
        assert!(!perception2.nearby.contains(&crit3));

        // Check crit3 perception
        let perception3 = world.get::<Perception>(crit3).unwrap();
        assert_eq!(perception3.neighbor_count(), 0); // Too far from others
    }

    #[test]
    fn test_perception_does_not_detect_self() {
        let mut world = World::new();

        let crit = world
            .spawn((Position { x: 0.0, y: 0.0 }, Perception::new(10.0)))
            .id();

        // Run perception system manually (test context)
        let positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(&world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query::<(Entity, &Position, &mut Perception)>();
        for (entity, pos, mut perception) in query.iter_mut(&mut world) {
            perception.clear();
            let range_sq = perception.range * perception.range;

            for (other_entity, other_pos) in &positions {
                if entity == *other_entity {
                    continue;
                }
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= range_sq {
                    perception.add_neighbor(*other_entity);
                }
            }
        }

        // Should not detect self
        let perception = world.get::<Perception>(crit).unwrap();
        assert_eq!(perception.neighbor_count(), 0);
        assert!(!perception.nearby.contains(&crit));
    }

    #[test]
    fn test_perception_clears_previous_neighbors() {
        let mut world = World::new();

        let crit1 = world
            .spawn((Position { x: 0.0, y: 0.0 }, Perception::new(10.0)))
            .id();

        let crit2 = world
            .spawn((Position { x: 5.0, y: 0.0 }, Perception::new(10.0)))
            .id();

        // Run perception first time
        let positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(&world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query::<(Entity, &Position, &mut Perception)>();
        for (entity, pos, mut perception) in query.iter_mut(&mut world) {
            perception.clear();
            let range_sq = perception.range * perception.range;

            for (other_entity, other_pos) in &positions {
                if entity == *other_entity {
                    continue;
                }
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= range_sq {
                    perception.add_neighbor(*other_entity);
                }
            }
        }

        // Verify crit1 sees crit2
        let perception1 = world.get::<Perception>(crit1).unwrap();
        assert_eq!(perception1.neighbor_count(), 1);

        // Move crit2 far away
        if let Some(mut pos2) = world.get_mut::<Position>(crit2) {
            pos2.x = 50.0; // Now 50m away
        }

        // Run perception again
        let positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(&world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query::<(Entity, &Position, &mut Perception)>();
        for (entity, pos, mut perception) in query.iter_mut(&mut world) {
            perception.clear();
            let range_sq = perception.range * perception.range;

            for (other_entity, other_pos) in &positions {
                if entity == *other_entity {
                    continue;
                }
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= range_sq {
                    perception.add_neighbor(*other_entity);
                }
            }
        }

        // Verify crit1 no longer sees crit2 (cleared properly)
        let perception1 = world.get::<Perception>(crit1).unwrap();
        assert_eq!(perception1.neighbor_count(), 0);
    }

    #[test]
    fn test_perception_respects_range() {
        let mut world = World::new();

        let crit1 = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Perception::new(5.0), // Short range
            ))
            .id();

        let crit2 = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Perception::new(20.0), // Long range
            ))
            .id();

        let crit3 = world
            .spawn((
                Position { x: 10.0, y: 0.0 }, // 10m away
                Perception::new(10.0),
            ))
            .id();

        // Run perception system manually (test context)
        let positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(&world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query::<(Entity, &Position, &mut Perception)>();
        for (entity, pos, mut perception) in query.iter_mut(&mut world) {
            perception.clear();
            let range_sq = perception.range * perception.range;

            for (other_entity, other_pos) in &positions {
                if entity == *other_entity {
                    continue;
                }
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= range_sq {
                    perception.add_neighbor(*other_entity);
                }
            }
        }

        // crit1 (5m range) should NOT see crit3 (10m away)
        let perception1 = world.get::<Perception>(crit1).unwrap();
        assert!(!perception1.nearby.contains(&crit3));

        // crit2 (20m range) SHOULD see crit3 (10m away)
        let perception2 = world.get::<Perception>(crit2).unwrap();
        assert!(perception2.nearby.contains(&crit3));
    }

    #[test]
    fn test_perception_performance_baseline() {
        // This test creates a realistic scenario for benchmarking
        let mut world = World::new();

        // Spawn 100 creatures in a 100x100m area
        for i in 0..100 {
            let x = (i % 10) as f32 * 10.0;
            let y = (i / 10) as f32 * 10.0;

            world.spawn((Position { x, y }, Perception::new(10.0)));
        }

        // Measure perception update time (naive baseline)
        let start = std::time::Instant::now();

        let positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(&world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query::<(Entity, &Position, &mut Perception)>();
        for (entity, pos, mut perception) in query.iter_mut(&mut world) {
            perception.clear();
            let range_sq = perception.range * perception.range;

            for (other_entity, other_pos) in &positions {
                if entity == *other_entity {
                    continue;
                }
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= range_sq {
                    perception.add_neighbor(*other_entity);
                }
            }
        }

        let duration = start.elapsed();

        // This is our baseline - should be ~0.1-1ms for 100 creatures
        println!("Perception update (100 crits, naive O(n²)): {:?}", duration);

        // Verify perception is actually working
        let mut total_neighbors = 0;
        for perception in world.query::<&Perception>().iter(&world) {
            total_neighbors += perception.neighbor_count();
        }

        // Each creature should see ~8-12 neighbors (10m range, 10m grid spacing)
        // Total should be ~800-1200 (100 crits × ~10 neighbors each)
        println!("Total neighbor detections: {}", total_neighbors);
        assert!(total_neighbors > 0, "Perception should detect some neighbors");
    }
}
