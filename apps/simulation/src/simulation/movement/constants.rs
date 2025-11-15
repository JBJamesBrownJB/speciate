use bevy_ecs::prelude::*;

pub const DEFAULT_BODY_LENGTH: f32 = 1.0; // Default creature body length in meters
pub const DEFAULT_MASS: f32 = 65.0; // Default mass for 1m creature (kg)
pub const MAX_SPEED: f32 = 50.0; // Maximum creature speed in m/s
pub const MAX_ACCELERATION: f32 = 5.0; // Maximum acceleration in m/s²
pub const MAX_TURN_RATE: f32 = 180.0; // Maximum turn rate in degrees/second
pub const VELOCITY_DAMPING: f32 = 0.95; // Per-frame velocity damping (mimics air resistance)
pub const DT: f32 = 0.05; // Simulation time step in seconds (20 Hz)
pub const SLOW_ZONE_MULTIPLIER: f32 = 30.0; // Slow zone size as multiple of personal_space

#[derive(Debug, Clone, Copy)]
pub struct SteeringConstants {
    pub seek_force: f32, // Force for goal-directed movement (Newtons)
    pub avoidance_force: f32, // Force for obstacle avoidance (Newtons)
    pub panic_force: f32, // Emergency evasion force (Newtons)
    pub wander_force: f32, // Random exploration force (Newtons)
    pub flee_force: f32, // Threat response force (Newtons)
}

impl Default for SteeringConstants {
    fn default() -> Self {
        Self {
            seek_force: 10.0,
            avoidance_force: 35.0,
            panic_force: 90.0,
            wander_force: 5.0,
            flee_force: 20.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PerceptionConstants {
    pub perception_multiplier: f32, // Perception range as multiple of body length
    pub personal_space: f32, // Spacing buffer distance in meters
    pub panic_threshold_ratio: f32, // Panic threshold as fraction of personal_space
}

impl Default for PerceptionConstants {
    fn default() -> Self {
        Self {
            perception_multiplier: 10.0,
            personal_space: 1.5,
            panic_threshold_ratio: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TerritoryConstants {
    pub comfort_radius: f32, // Territory core radius where creature explores freely
    pub blend_center: f32, // Distance where wander/homeward forces are 50/50
    pub max_wander_distance: f32, // Hard limit for excursions from home
    pub homeward_force: f32, // Force magnitude pulling creature toward home (Newtons)
    pub sigmoid_steepness: f32, // Steepness of elastic tether transition curve
}

impl Default for TerritoryConstants {
    fn default() -> Self {
        Self {
            comfort_radius: 10.0,
            blend_center: 20.0,
            max_wander_distance: 30.0,
            homeward_force: 50.0,
            sigmoid_steepness: 1.5,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SeekingConstants {
    pub max_force: f32, // Maximum seeking force (Newtons)
    pub brake_force: f32, // Emergency brake force when overshooting (Newtons)
    pub pounce_distance: f32, // Snap-to-target distance threshold (meters)
    pub pounce_speed: f32, // Maximum speed for pounce snap (m/s)
    pub arrival_tolerance: f32, // Stop when this close to target (meters)
    pub slow_zone_decay: f32, // Exponential decay factor for deceleration curve
}

impl Default for SeekingConstants {
    fn default() -> Self {
        Self {
            max_force: 50.0,
            brake_force: 70.0,
            pounce_distance: 0.5,
            pounce_speed: 5.5,
            arrival_tolerance: 0.5,
            slow_zone_decay: 1.5,
        }
    }
}

pub static STEERING: SteeringConstants = SteeringConstants {
    seek_force: 10.0,
    avoidance_force: 35.0,
    panic_force: 90.0,
    wander_force: 5.0,
    flee_force: 20.0,
};

pub static PERCEPTION: PerceptionConstants = PerceptionConstants {
    perception_multiplier: 10.0,
    personal_space: 1.5,
    panic_threshold_ratio: 0.5,
};

pub static TERRITORY: TerritoryConstants = TerritoryConstants {
    comfort_radius: 10.0,
    blend_center: 20.0,
    max_wander_distance: 30.0,
    homeward_force: 50.0,
    sigmoid_steepness: 1.5,
};

pub static SEEKING: SeekingConstants = SeekingConstants {
    max_force: 50.0,
    brake_force: 70.0,
    pounce_distance: 0.5,
    pounce_speed: 5.5,
    arrival_tolerance: 0.5,
    slow_zone_decay: 1.5,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_hierarchy() {
        let steering = SteeringConstants::default();
        assert!(steering.panic_force > steering.avoidance_force);
        assert!(steering.avoidance_force > steering.seek_force);
        assert!(steering.seek_force > steering.wander_force);
    }

    #[test]
    fn test_perception_scaling() {
        let perception = PerceptionConstants::default();
        let body_size = 1.0;
        let perception_range = body_size * perception.perception_multiplier;
        let personal_space = body_size + perception.personal_space;
        let panic_threshold = personal_space * perception.panic_threshold_ratio;

        assert_eq!(perception_range, 10.0);
        assert_eq!(personal_space, 2.5);
        assert_eq!(panic_threshold, 1.25);
        assert!(panic_threshold < personal_space);
    }

    #[test]
    fn test_constants_are_positive() {
        let steering = SteeringConstants::default();
        let perception = PerceptionConstants::default();

        assert!(steering.seek_force > 0.0);
        assert!(steering.avoidance_force > 0.0);
        assert!(steering.panic_force > 0.0);
        assert!(steering.wander_force > 0.0);
        assert!(steering.flee_force > 0.0);

        assert!(perception.perception_multiplier > 0.0);
        assert!(perception.personal_space > 0.0);
        assert!(perception.panic_threshold_ratio > 0.0);
        assert!(perception.panic_threshold_ratio < 1.0);
    }

    #[test]
    fn test_territory_constants_valid() {
        let territory = TerritoryConstants::default();

        assert!(territory.comfort_radius > 0.0);
        assert!(territory.blend_center > 0.0);
        assert!(territory.max_wander_distance > 0.0);
        assert!(territory.homeward_force > 0.0);
        assert!(territory.sigmoid_steepness > 0.0);

        assert!(territory.comfort_radius < territory.blend_center,
            "Comfort radius ({}) should be less than blend center ({})",
            territory.comfort_radius, territory.blend_center);
        assert!(territory.blend_center < territory.max_wander_distance,
            "Blend center ({}) should be less than max wander distance ({})",
            territory.blend_center, territory.max_wander_distance);

        assert!(territory.sigmoid_steepness >= 0.1 && territory.sigmoid_steepness <= 5.0,
            "Sigmoid steepness ({}) should be between 0.1 and 5.0",
            territory.sigmoid_steepness);
    }

    #[test]
    fn test_territory_global_instance() {
        assert!(TERRITORY.comfort_radius > 0.0);
        assert!(TERRITORY.blend_center > TERRITORY.comfort_radius);
        assert!(TERRITORY.max_wander_distance > TERRITORY.blend_center);
    }

    #[test]
    fn test_seeking_constants_valid() {
        let seeking = SeekingConstants::default();

        assert!(seeking.max_force > 0.0);
        assert!(seeking.brake_force > 0.0);
        assert!(seeking.pounce_distance > 0.0);
        assert!(seeking.pounce_speed > 0.0);
        assert!(seeking.arrival_tolerance > 0.0);
        assert!(seeking.slow_zone_decay > 0.0);

        assert!(seeking.brake_force > seeking.max_force,
            "Brake force ({}) should exceed max force ({}) for emergency stopping",
            seeking.brake_force, seeking.max_force);

        assert!(seeking.pounce_distance < 5.0,
            "Pounce distance ({}) should be small for precise arrival",
            seeking.pounce_distance);

        assert!(seeking.arrival_tolerance < 5.0,
            "Arrival tolerance ({}) should be small for target precision",
            seeking.arrival_tolerance);

        assert!(seeking.slow_zone_decay >= 0.5 && seeking.slow_zone_decay <= 5.0,
            "Slow zone decay ({}) should be between 0.5 and 5.0",
            seeking.slow_zone_decay);
    }

    #[test]
    fn test_seeking_global_instance() {
        assert!(SEEKING.max_force > 0.0);
        assert!(SEEKING.brake_force > SEEKING.max_force);
        assert!(SEEKING.pounce_distance > 0.0);
    }

    #[test]
    fn test_force_magnitudes_relative() {
        assert!(TERRITORY.homeward_force > STEERING.seek_force,
            "Homeward force ({}) should be stronger than general seeking ({})",
            TERRITORY.homeward_force, STEERING.seek_force);

        assert!(SEEKING.max_force >= STEERING.seek_force,
            "Seeking max_force ({}) should be at least as strong as general seek ({})",
            SEEKING.max_force, STEERING.seek_force);
    }
}
