use super::step::{step, SteeringCtx};
use crate::simulation::core::components::{Acceleration, BodySize, Position, Velocity};
use crate::simulation::creatures::components::{
    BehaviorMode, Brain, CanAvoidObstacles, CanSeek, CanWander, CreatureState, HomePosition,
    Target, WanderState,
};
use crate::simulation::creatures::constants::{SEEK_FORCE_MULT, WANDER_FORCE_MULT};
use crate::simulation::perception::NeighborCache;
use bevy_ecs::prelude::*;

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
        &'static Brain,
        &'static mut WanderState,
        &'static HomePosition,
        &'static Target,
        &'static NeighborCache,
        Has<CanWander>,
        Has<CanSeek>,
        Has<CanAvoidObstacles>,
    ),
>;

pub fn update_steering_system(
    mut query: SteeringQuery,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "steering");

    let ctx = SteeringCtx {
        wander_force_mult: WANDER_FORCE_MULT.get(),
        seek_force_mult: SEEK_FORCE_MULT.get(),
    };

    query.par_iter_mut().for_each(
        |(
            _entity,
            position,
            velocity,
            size,
            mut acceleration,
            mut creature_state,
            brain,
            mut wander_state,
            _home,
            target,
            neighbor_cache,
            can_wander,
            can_seek,
            can_avoid,
        )| {
            if !brain.mode.makes_decisions() {
                return;
            }

            let wander_angle_change_radians =
                if creature_state.behavior == BehaviorMode::Wandering && can_wander {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    rng.gen_range(-wander_state.angle_change..wander_state.angle_change)
                        .to_radians()
                } else {
                    0.0
                };

            let output = step(
                &position,
                &velocity,
                &size,
                &creature_state,
                &mut wander_state,
                &target,
                &neighbor_cache,
                can_wander,
                can_seek,
                can_avoid,
                wander_angle_change_radians,
                &ctx,
            );

            debug_assert!(
                acceleration.ax == 0.0 && acceleration.ay == 0.0,
                "steering cap assumes zero Acceleration at entry"
            );
            acceleration.ax += output.ax;
            acceleration.ay += output.ay;
            if output.arrived {
                creature_state.behavior = BehaviorMode::Catatonic;
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::{IntoSystem, System};

    fn run_system(world: &mut World) {
        bevy_tasks::ComputeTaskPool::get_or_init(bevy_tasks::TaskPool::default);

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
            angle_change: 4.5,
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

        let entity = world
            .spawn((
                Position { x: 0.95, y: 0.0 },
                Velocity { vx: 0.1, vy: 0.0 },
                Acceleration::default(),
                BodySize::new(1.0),
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

        run_system(&mut world);

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

        assert!(
            mag <= max_accel + 0.01,
            "Acceleration magnitude {} should be capped to max_accel {} (or below)",
            mag,
            max_accel
        );
    }

    #[test]
    fn avoidance_fires_at_spawn_energy() {
        use crate::simulation::creatures::constants::DEFAULT_ENERGY;
        use crate::simulation::perception::components::NeighborData;

        let mut cache = NeighborCache::new();
        cache.add_neighbor(NeighborData {
            entity: Entity::PLACEHOLDER,
            x: 10.0,
            y: 0.0,
            vx: -5.0,
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
