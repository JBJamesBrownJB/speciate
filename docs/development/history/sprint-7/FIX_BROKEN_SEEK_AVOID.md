This is a classic problem in game AI! Your creatures are "oscillating" because their behaviors are binary: they are either 100% Seeking or 100% Avoiding.

When the comfort zone is hit, the Avoid behavior takes 100% control and says "go 180° away." The instant it's 1 pixel outside the zone, the Seek behavior takes 100% control and says "go 180° back toward the target."

This also causes the "stuck" problem: in a narrow gap, the creature is always in an Avoid state from both sides, so it can never move forward.

The solution is to blend your forces instead of switching between them.

## Solution 1: Use Force Accumulation (The 90% Fix)

This is the real solution, and your ECS architecture doc already describes it. The seek_system and avoid_system shouldn't be mutually exclusive. They should both add forces to the Acceleration component in the same tick.

The Avoid force should just be a "nudge," not a 100% override.

    seek_system: Calculates a seek_force pulling the crit toward its target.

    obstacle_avoidance_system:

        Checks if any obstacle is inside the comfort zone.

        If yes, it calculates an avoid_force pointing directly away from the obstacle's center.

        Crucially: This force should have a falloff. The deeper the crit is inside the comfort zone, the stronger the force (e..g, force = 1 / distance).

    physics_system:

        Reads the final Acceleration (which is the sum of seek_force + avoid_force).

        Applies this blended vector to the Velocity.

The Result: The crit is always being pulled toward its target. The avoid_force just adds a "nudge" away from the obstacle. The resulting path is a smooth curve around the comfort zone, not a 180-degree bounce. This will also solve your "gap" problem, as the seek_force will be strong enough to "push through" the two weak avoid_force vectors from either side.

## Solution 2: Add Predictive "Feelers" (The 10% Polish)

The "comfort zone" model is purely reactive. A more advanced solution is to make your creatures predictive.

This is the classic "Obstacle Avoidance" steering behavior.

    Project "Feelers": In your obstacle_avoidance_system, you project one or more "feelers" (short lines) in front of the creature in its direction of travel.

    Check for Intersection: You check if these feelers will intersect an obstacle's comfort zone.

    Generate Steering Force: If a feeler hits, you generate a steering force that pushes the crit away from that feeler (e.g., perpendicular to the feeler).

The Result: The crit "sees" the wall before it hits the comfort zone and begins a smooth, early turn to "slide" along the edge. This is extremely effective for navigating narrow gaps because it naturally follows the wall.

## Solution 3: The "Stuck" Failsafe

Even with the fixes above, crits can get stuck in complex geometry. You should add a simple failsafe.

    Add a Timer: In your physics_system, check if the creature's Velocity has been near-zero for more than 2-3 seconds.

    Trigger "Stuck" State: If it is, change its BehaviorState to Stuck.

    "Stuck" System: A new system runs With<Stuck>. It picks a random direction (e.g., 90° left) and applies a strong force for 1 second to "jiggle" the crit loose.

    Transition: After 1 second, it sets the state back to Seeking.

Refined "Stuck" Failsafe

You need to check both velocity and state. The "stuck" timer should only run for creatures that should be moving.

1. The "Stuck Timer" Component

First, add a timer component to your "stuck-able" entities:
Rust

#[derive(Component)]
pub struct StuckTimer(Timer);

Add this (paused) when you spawn a creature that can Seek or Wander.

2. The "Stuck Detection" System

This system runs every tick. It checks for creatures that are trying to move but aren't.
Rust

fn check_if_stuck_system(
    mut query: Query<(&Velocity, &BehaviorState, &mut StuckTimer)>,
    time: Res<Time>,
) {
    for (velocity, behavior, mut timer) in query.iter_mut() {
        
        let is_moving_fast = velocity.length_sq() > 0.1; // Use squared length for speed

        let is_movement_state = matches!(
            behavior,
            BehaviorState::Seeking { .. } |
            BehaviorState::Wandering { .. } |
            BehaviorState::Fleeing { .. }
        );

        if is_movement_state && !is_moving_fast {
            // Creature wants to move but isn't. Start the timer.
            timer.0.unpause();
            timer.0.tick(time.delta());
        } else {
            // Creature is moving, or is in a non-movement state (like Catatonic/Feeding).
            // Reset the timer.
            timer.0.reset();
            timer.0.pause();
        }
    }
}

3. The "Unstuck" System

This system runs on creatures whose timer has finally finished.
Rust

fn unstuck_system(
    mut query: Query<(&StuckTimer, &mut BehaviorState, &mut Velocity)>,
) {
    for (timer, mut behavior, mut velocity) in query.iter_mut() {
        if timer.0.just_finished() {
            // Failsafe triggered!
            // 1. Give it a random "jiggle"
            let random_force = // ... logic to pick a random direction
            velocity.vx += random_force.x;
            
            // 2. Force it into a temporary "Wandering" state
            //    to re-evaluate its surroundings.
            *behavior = BehaviorState::Wandering { angle: 0.0 }; 
        }
    }
}

This way, your intentionally stationary creatures (Catatonic, Feeding) will never trigger the "stuck" failsafe, but your Seeking creature that's bouncing between two trees will.