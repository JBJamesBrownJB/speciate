//! Fused steering system - single query, single iteration for all steering behaviors.
//!
//! This replaces 4 separate systems (wander, seek, avoidance, flee) with 1 unified system.
//! Performance gain comes from:
//! - Single query setup instead of 4
//! - Single Vec::collect() for Rayon instead of 4
//! - Single Rayon sync barrier instead of 4
//! - Better cache utilization (each entity's components loaded once)

use super::seek::{calculate_arrival, ArrivalParams};
use super::wander::{calculate_wander, WanderParams};
use crate::simulation::core::components::{Acceleration, BodySize, Position, Velocity};
use crate::simulation::creatures::components::{
    BehaviorMode, Brain, CanAvoidObstacles, CanSeek, CanWander, CreatureState, HomePosition,
    Target, WanderState,
};
use crate::simulation::creatures::constants::{SEEK_FORCE_MULT, WANDER_FORCE_MULT};
use crate::simulation::math::SteeringContext;
use crate::simulation::perception::NeighborCache;
use bevy_ecs::prelude::*;
use rayon::prelude::*;

/// Clamp acceleration vector to maximum magnitude.
fn cap_acceleration(ax: f32, ay: f32, max_accel: f32) -> (f32, f32) {
    let mag_sq = ax * ax + ay * ay;
    let max_sq = max_accel * max_accel;
    if mag_sq > max_sq && mag_sq > 0.0001 {
        let scale = max_accel / mag_sq.sqrt();
        (ax * scale, ay * scale)
    } else {
        (ax, ay)
    }
}

/// Apply wander behavior, returning acceleration to accumulate.
fn apply_wander(ctx: &SteeringContext, wander_state: &mut WanderState) -> (f32, f32) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let wander_params = WanderParams {
        wander_angle: wander_state.wander_angle,
        wander_radius: wander_state.wander_radius,
        wander_distance: wander_state.wander_distance,
        force_multiplier: WANDER_FORCE_MULT.get(),
    };

    let angle_change = rng.gen_range(-wander_state.angle_change..wander_state.angle_change);
    let result = calculate_wander(ctx, &wander_params, angle_change.to_radians());

    wander_state.wander_angle = result.new_wander_angle;

    let (ax, ay) = result.acceleration;
    if ax.is_finite() && ay.is_finite() {
        (ax, ay)
    } else {
        (0.0, 0.0)
    }
}

/// Seek behavior output with arrival flag.
struct SeekOutput {
    acceleration: (f32, f32),
    arrived: bool,
}

/// Apply seek behavior, returning acceleration and arrival status.
fn apply_seek(
    position: &Position,
    velocity: &Velocity,
    target: &Target,
    size: &BodySize,
) -> SeekOutput {
    let params = ArrivalParams {
        velocity: (velocity.vx, velocity.vy),
        to_target: (target.x - position.x, target.y - position.y),
        self_radius: size.radius(),
        target_radius: target.radius.get(),
        max_speed: size.max_speed(),
        max_force: size.max_force() * SEEK_FORCE_MULT.get(),
        mass: size.mass(),
    };

    let result = calculate_arrival(&params);
    SeekOutput {
        acceleration: result.acceleration,
        arrived: result.arrived,
    }
}

/// Apply TTC-based avoidance behavior, returning acceleration to accumulate.
fn apply_avoidance(
    neighbor_cache: &NeighborCache,
    self_pos: (f32, f32),
    self_vel: (f32, f32),
    self_radius: f32,
    max_accel: f32,
) -> (f32, f32) {
    use super::avoidance::{calculate_avoidance, AvoidanceConfig, AvoidanceInput, Neighbor};

    let input = AvoidanceInput {
        self_pos,
        self_vel,
        self_radius,
        max_accel,
    };

    // Zero-allocation: lazy iterator conversion, no Vec
    let neighbors = neighbor_cache.iter_neighbors().map(|n| Neighbor {
        pos: (n.x, n.y),
        vel: (n.vx, n.vy),
        radius: n.radius,
    });

    let config = AvoidanceConfig::default();
    let output = calculate_avoidance(&input, neighbors, &config);

    output.accel
}

/// Fused steering system query - all components needed by any steering behavior.
/// Uses `Has<T>` for capability markers to avoid Option overhead.
pub type SteeringQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Position,
        &'static Velocity,
        &'static BodySize,
        &'static mut Acceleration,
        &'static mut CreatureState,
        // Brain for dormant check
        &'static Brain,
        // Wander-specific (mutable state)
        &'static mut WanderState,
        &'static HomePosition,
        // Seek-specific
        &'static Target,
        // Avoidance-specific
        &'static NeighborCache,
        // Capability markers (zero-sized, no cache impact)
        Has<CanWander>,
        Has<CanSeek>,
        Has<CanAvoidObstacles>,
    ),
>;

/// Fused steering system - calculates wander, seek, avoidance, and flee forces in single iteration.
///
/// This is the core Sprint 20 optimization: replacing 4 separate systems with 1.
/// Forces are accumulated additively (sum of all steering forces).
///
/// System ordering: Must run AFTER behavior_transition_system, BEFORE integrate_motion_system.
pub fn update_steering_system(
    mut query: SteeringQuery,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "steering");

    // Collect entities for parallel processing (required pattern for Rayon in NAPI context)
    let mut entities: Vec<_> = query.iter_mut().collect();

    // Parallel iteration with minimum batch size for efficiency
    // Steering: Medium workload - moderate chunks balance load vs overhead
    entities.par_iter_mut().with_min_len(256).for_each(
        |(
            _entity,
            position,
            velocity,
            size,
            acceleration,
            creature_state,
            brain,
            wander_state,
            _home,
            target,
            neighbor_cache,
            can_wander,
            can_seek,
            can_avoid,
        )| {
            // Skip steering for dormant brains (used in tests for stationary creatures)
            if !brain.mode.makes_decisions() {
                return;
            }

            // Build steering context for wander behavior
            let ctx = crate::simulation::math::SteeringContext {
                velocity: (velocity.vx, velocity.vy),
                max_speed: size.max_speed(),
                max_force: size.max_force(),
                mass: size.mass(),
            };

            // 1. Primary behavior (mutually exclusive based on BehaviorMode)
            match creature_state.behavior {
                BehaviorMode::Wandering if *can_wander => {
                    let (ax, ay) = apply_wander(&ctx, wander_state);
                    acceleration.ax += ax;
                    acceleration.ay += ay;
                }
                BehaviorMode::Seeking if *can_seek => {
                    let result = apply_seek(position, velocity, target, size);
                    if result.arrived {
                        creature_state.behavior = BehaviorMode::Catatonic;
                    } else {
                        acceleration.ax += result.acceleration.0;
                        acceleration.ay += result.acceleration.1;
                    }
                }
                _ => {}
            }

            // 2. Avoidance (additive with primary behavior)
            if *can_avoid && neighbor_cache.has_neighbors() {
                let max_accel = size.max_force() / size.mass();
                let (ax, ay) = apply_avoidance(
                    neighbor_cache,
                    (position.x, position.y),
                    (velocity.vx, velocity.vy),
                    size.radius(),
                    max_accel,
                );
                acceleration.ax += ax;
                acceleration.ay += ay;
            }

            // 3. Cap accumulated steering to creature's physical maximum
            let max_accel = size.max_force() / size.mass();
            (acceleration.ax, acceleration.ay) =
                cap_acceleration(acceleration.ax, acceleration.ay, max_accel);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::{IntoSystem, System};

    fn run_system(world: &mut World) {
        #[cfg(feature = "dev-tools")]
        world.insert_resource(crate::instrumentation::SystemTimings::new());

        let mut system = IntoSystem::into_system(update_steering_system);
        system.initialize(world);
        system.run((), world);
        system.apply_deferred(world);
    }

    fn test_wander_state() -> WanderState {
        WanderState {
            wander_angle: 0.0,
            wander_radius: 10.0,
            wander_distance: 20.0,
            angle_change: 4.5, // Non-zero to avoid empty range panic
        }
    }

    fn spawn_wanderer(world: &mut World) -> Entity {
        world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity { vx: 1.0, vy: 0.0 },
                Acceleration::default(),
                BodySize::default(),
                Brain::default(),
                test_wander_state(),
                HomePosition::new(0.0, 0.0),
                Target::at_point(0.0, 0.0),
                NeighborCache::new(),
                CreatureState {
                    behavior: BehaviorMode::Wandering,
                    ..Default::default()
                },
                CanWander,
                CanSeek,
                CanAvoidObstacles,
            ))
            .id()
    }

    fn spawn_seeker(world: &mut World, target_x: f32, target_y: f32) -> Entity {
        world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity { vx: 0.0, vy: 0.0 },
                Acceleration::default(),
                BodySize::default(),
                Brain::default(),
                test_wander_state(),
                HomePosition::new(0.0, 0.0),
                Target::at_point(target_x, target_y),
                NeighborCache::new(),
                CreatureState {
                    behavior: BehaviorMode::Seeking,
                    ..Default::default()
                },
                CanWander,
                CanSeek,
                CanAvoidObstacles,
            ))
            .id()
    }

    #[test]
    fn wandering_creature_produces_acceleration() {
        let mut world = World::new();
        let entity = spawn_wanderer(&mut world);

        run_system(&mut world);

        let accel = world.get::<Acceleration>(entity).unwrap();
        let mag = (accel.ax * accel.ax + accel.ay * accel.ay).sqrt();
        assert!(mag > 0.0, "Wandering creature should produce acceleration");
    }

    #[test]
    fn seeking_creature_accelerates_toward_target() {
        let mut world = World::new();
        let entity = spawn_seeker(&mut world, 100.0, 0.0);

        run_system(&mut world);

        let accel = world.get::<Acceleration>(entity).unwrap();
        assert!(
            accel.ax > 0.0,
            "Seeker should accelerate toward target (+X), got ax={}",
            accel.ax
        );
    }

    #[test]
    fn seeker_arrives_at_target() {
        let mut world = World::new();

        // Place creature very close to target (within snap threshold)
        let entity = world
            .spawn((
                Position { x: 0.95, y: 0.0 },  // Very close to target
                Velocity { vx: 0.1, vy: 0.0 }, // Moving slowly
                Acceleration::default(),
                BodySize::new(1.0), // radius = 0.5
                Brain::default(),
                test_wander_state(),
                HomePosition::new(0.0, 0.0),
                Target::at_point(1.0, 0.0),
                NeighborCache::new(),
                CreatureState {
                    behavior: BehaviorMode::Seeking,
                    ..Default::default()
                },
                CanWander,
                CanSeek,
                CanAvoidObstacles,
            ))
            .id();

        run_system(&mut world);

        let state = world.get::<CreatureState>(entity).unwrap();
        assert_eq!(
            state.behavior,
            BehaviorMode::Catatonic,
            "Seeker should transition to Catatonic on arrival"
        );
    }

    #[test]
    fn catatonic_creature_produces_no_acceleration() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity { vx: 0.0, vy: 0.0 },
                Acceleration::default(),
                BodySize::default(),
                Brain::default(),
                test_wander_state(),
                HomePosition::new(0.0, 0.0),
                Target::at_point(0.0, 0.0),
                NeighborCache::new(),
                CreatureState {
                    behavior: BehaviorMode::Catatonic,
                    ..Default::default()
                },
                CanWander,
                CanSeek,
                CanAvoidObstacles,
            ))
            .id();

        run_system(&mut world);

        let accel = world.get::<Acceleration>(entity).unwrap();
        assert_eq!(
            accel.ax, 0.0,
            "Catatonic creature should have no acceleration"
        );
        assert_eq!(accel.ay, 0.0);
    }

    #[test]
    fn multiple_creatures_process_in_parallel() {
        let mut world = World::new();

        // Spawn 100 creatures
        for i in 0..100 {
            let x = (i as f32 % 10.0) * 10.0;
            let y = (i as f32 / 10.0) * 10.0;
            world.spawn((
                Position { x, y },
                Velocity { vx: 1.0, vy: 0.0 },
                Acceleration::default(),
                BodySize::default(),
                Brain::default(),
                test_wander_state(),
                HomePosition::new(x, y),
                Target::at_point(0.0, 0.0),
                NeighborCache::new(),
                CreatureState {
                    behavior: BehaviorMode::Wandering,
                    ..Default::default()
                },
                CanWander,
                CanSeek,
                CanAvoidObstacles,
            ));
        }

        // Should not panic, should process all creatures
        run_system(&mut world);

        // Verify all creatures have non-zero acceleration
        let mut query = world.query::<&Acceleration>();
        let mut processed = 0;
        for accel in query.iter(&world) {
            let mag = (accel.ax * accel.ax + accel.ay * accel.ay).sqrt();
            if mag > 0.0 {
                processed += 1;
            }
        }
        assert!(processed > 90, "Most creatures should have acceleration");
    }

    #[test]
    fn steering_cap_respects_max_accel() {
        let mut world = World::new();

        let body = BodySize::default();
        let max_accel = body.max_force() / body.mass();

        let entity = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity { vx: 1.0, vy: 0.0 },
                Acceleration::default(),
                body,
                Brain::default(),
                test_wander_state(),
                HomePosition::new(0.0, 0.0),
                Target::at_point(0.0, 0.0),
                NeighborCache::new(),
                CreatureState {
                    behavior: BehaviorMode::Wandering,
                    ..Default::default()
                },
                CanWander,
                CanSeek,
                CanAvoidObstacles,
            ))
            .id();

        run_system(&mut world);

        let accel = world.get::<Acceleration>(entity).unwrap();
        let mag = (accel.ax * accel.ax + accel.ay * accel.ay).sqrt();

        // Acceleration should be capped to max_accel
        assert!(
            mag <= max_accel + 0.01,
            "Acceleration magnitude {} should be capped to max_accel {} (or below)",
            mag,
            max_accel
        );
    }

    #[test]
    fn avoidance_fires_at_spawn_energy() {
        // Regression guard: avoidance must fire for a creature at real spawn energy
        // (DEFAULT_ENERGY = 100.0) when an approaching neighbor is present.
        // Any behavioral gate on energy with a threshold between 0 and DEFAULT_ENERGY
        // would suppress avoidance for every freshly spawned creature — this test
        // would have caught the hunger-gated-avoidance regression (2026-06-26).
        //
        // Uses Catatonic so wander/seek forces are zero: avoidance is the sole
        // contributor, making a non-zero magnitude an unambiguous avoidance signal.
        use crate::simulation::creatures::constants::DEFAULT_ENERGY;
        use crate::simulation::perception::components::NeighborData;

        let mut cache = NeighborCache::new();
        cache.add_neighbor(NeighborData {
            entity: Entity::PLACEHOLDER,
            x: 10.0,
            y: 0.0,
            vx: -5.0, // approaching fast
            vy: 0.0,
            radius: 0.5,
        });

        let mut world = World::new();
        let entity = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity { vx: 0.0, vy: 0.0 },
                Acceleration::default(),
                BodySize::default(),
                Brain::default(),
                test_wander_state(),
                HomePosition::new(0.0, 0.0),
                Target::at_point(0.0, 0.0),
                cache,
                CreatureState {
                    behavior: BehaviorMode::Catatonic,
                    energy: DEFAULT_ENERGY,
                    ..Default::default()
                },
                CanWander,
                CanSeek,
                CanAvoidObstacles,
            ))
            .id();

        run_system(&mut world);

        let accel = world.get::<Acceleration>(entity).unwrap();
        let mag = (accel.ax * accel.ax + accel.ay * accel.ay).sqrt();
        assert!(
            mag > 0.0,
            "Creature at spawn energy ({DEFAULT_ENERGY}) with approaching neighbor must produce \
             avoidance acceleration — got zero (avoidance is being gated or neighbor not seen)"
        );
    }
}
