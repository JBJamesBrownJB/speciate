//! Spec test harness - utilities for behavioral specification tests
//!
//! This module provides a framework for testing high-level simulation behaviors
//! against trial configurations. The same trials can be run visually for manual
//! verification.

use speciate::{BodySize, CritId, Position, Velocity, SimulationBuilder};
use speciate::trials::loader::load_trial;

pub mod avoidance_behavior;
pub mod crowd_navigation;

/// Result of running a spec test trial
pub struct SpecResult {
    pub tick_count: usize,
    pub creatures: Vec<CreatureSnapshot>,
}

/// Snapshot of creature state for assertions
#[derive(Debug, Clone)]
pub struct CreatureSnapshot {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
}

impl CreatureSnapshot {
    pub fn speed(&self) -> f32 {
        (self.vx * self.vx + self.vy * self.vy).sqrt()
    }
}

/// Load a trial and run for N ticks, returning final state
pub fn run_trial(trial_name: &str, ticks: usize, delta_time: f32) -> SpecResult {
    run_trial_with_callback(trial_name, ticks, delta_time, |_, _| {})
}

/// Load a trial and run for N ticks, calling callback each tick with current state
pub fn run_trial_with_callback<F>(
    trial_name: &str,
    ticks: usize,
    delta_time: f32,
    mut per_tick: F,
) -> SpecResult
where
    F: FnMut(usize, &[CreatureSnapshot]),
{
    let mut sim = SimulationBuilder::new()
        .set_boundaries(500.0, 500.0) // Large default boundary
        .build();

    // Load trial into simulation world
    {
        let world = sim.world_mut();
        load_trial(world, trial_name)
            .unwrap_or_else(|e| panic!("Failed to load trial '{}': {}", trial_name, e));
    }

    // Run simulation for specified ticks
    for tick in 0..ticks {
        sim.update(delta_time);

        // Capture state for callback
        let world = sim.world_mut();
        let mut query = world.query::<(&CritId, &Position, &Velocity, &BodySize)>();
        let creatures: Vec<CreatureSnapshot> = query
            .iter(world)
            .map(|(id, pos, vel, size)| CreatureSnapshot {
                id: id.0,
                x: pos.x,
                y: pos.y,
                vx: vel.vx,
                vy: vel.vy,
                radius: size.radius(),
            })
            .collect();

        per_tick(tick, &creatures);
    }

    // Capture final state
    let world = sim.world_mut();
    let mut query = world.query::<(&CritId, &Position, &Velocity, &BodySize)>();

    let creatures: Vec<CreatureSnapshot> = query
        .iter(world)
        .map(|(id, pos, vel, size)| CreatureSnapshot {
            id: id.0,
            x: pos.x,
            y: pos.y,
            vx: vel.vx,
            vy: vel.vy,
            radius: size.radius(),
        })
        .collect();

    SpecResult {
        tick_count: ticks,
        creatures,
    }
}

/// Check if any two creatures overlap (radii touching)
/// Returns list of (id_a, id_b, overlap_amount)
pub fn find_overlaps(creatures: &[CreatureSnapshot]) -> Vec<(u32, u32, f32)> {
    let mut overlaps = Vec::new();

    for i in 0..creatures.len() {
        for j in (i + 1)..creatures.len() {
            let a = &creatures[i];
            let b = &creatures[j];

            let dx = a.x - b.x;
            let dy = a.y - b.y;
            let distance = (dx * dx + dy * dy).sqrt();
            let min_distance = a.radius + b.radius;

            if distance < min_distance {
                let overlap_amount = min_distance - distance;
                overlaps.push((a.id, b.id, overlap_amount));
            }
        }
    }

    overlaps
}

/// Assert no creatures overlap (radii touching)
pub fn assert_no_overlaps(result: &SpecResult) {
    let overlaps = find_overlaps(&result.creatures);

    if !overlaps.is_empty() {
        let mut msg = format!(
            "Found {} overlapping creature pairs after {} ticks:\n",
            overlaps.len(),
            result.tick_count
        );
        for (id_a, id_b, amount) in overlaps.iter().take(10) {
            msg.push_str(&format!(
                "  Crit {} and {} overlap by {:.3} units\n",
                id_a, id_b, amount
            ));
        }
        if overlaps.len() > 10 {
            msg.push_str(&format!("  ... and {} more\n", overlaps.len() - 10));
        }
        panic!("{}", msg);
    }
}

/// Assert creature reached target area (within tolerance)
pub fn assert_reached_target(
    creature: &CreatureSnapshot,
    target_x: f32,
    target_y: f32,
    tolerance: f32,
) {
    let dx = creature.x - target_x;
    let dy = creature.y - target_y;
    let distance = (dx * dx + dy * dy).sqrt();

    assert!(
        distance <= tolerance,
        "Crit {} should reach target ({}, {}) within {} units, but is at ({:.1}, {:.1}) - distance {:.1}",
        creature.id,
        target_x,
        target_y,
        tolerance,
        creature.x,
        creature.y,
        distance
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_overlaps_no_overlap() {
        let creatures = vec![
            CreatureSnapshot { id: 1, x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, radius: 0.5 },
            CreatureSnapshot { id: 2, x: 5.0, y: 0.0, vx: 0.0, vy: 0.0, radius: 0.5 },
        ];
        let overlaps = find_overlaps(&creatures);
        assert!(overlaps.is_empty());
    }

    #[test]
    fn test_find_overlaps_with_overlap() {
        let creatures = vec![
            CreatureSnapshot { id: 1, x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, radius: 0.5 },
            CreatureSnapshot { id: 2, x: 0.8, y: 0.0, vx: 0.0, vy: 0.0, radius: 0.5 }, // Distance 0.8, min 1.0
        ];
        let overlaps = find_overlaps(&creatures);
        assert_eq!(overlaps.len(), 1);
        assert_eq!(overlaps[0].0, 1);
        assert_eq!(overlaps[0].1, 2);
        assert!((overlaps[0].2 - 0.2).abs() < 0.001); // Overlap by 0.2 units
    }

    #[test]
    fn test_find_overlaps_exactly_touching() {
        let creatures = vec![
            CreatureSnapshot { id: 1, x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, radius: 0.5 },
            CreatureSnapshot { id: 2, x: 1.0, y: 0.0, vx: 0.0, vy: 0.0, radius: 0.5 }, // Distance 1.0, min 1.0
        ];
        let overlaps = find_overlaps(&creatures);
        // Exactly touching is NOT overlap (distance == min_distance)
        assert!(overlaps.is_empty());
    }
}
