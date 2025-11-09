//! Perception components for spatial awareness
//!
//! See `/workspace/docs/biology/dna-driven-design.md` for future DNA integration.

use bevy_ecs::prelude::*;

/// Spatial awareness component - what a creature can detect
///
/// Updated by `update_perception_system` every frame (naive O(n²) for now).
/// Stores cached list of nearby entities to avoid repeated distance calculations.
///
/// # Parameters
/// - **range:** Detection distance in meters (10× body length default)
/// - **nearby:** Cached entities within perception range
///
/// # Threading
/// This component is **read-only** during behavior systems, so avoidance
/// can run in parallel with other behaviors without data races.
///
/// # Future Optimizations
/// - Spatial hash for faster neighbor queries
/// - Staggered updates (not all crits update same frame)
/// - Different update rates based on movement speed
#[derive(Component, Debug, Clone)]
pub struct Perception {
    /// Maximum detection distance in meters
    ///
    /// **Current:** 10× body length (10m for 1m creature)
    /// **Biological rationale:** Active forager sensory range
    ///
    /// **Future DNA gene:** `perception_multiplier` (3.0-20.0×)
    /// - 3×: Ambush predator (short-range)
    /// - 10×: Active forager (default)
    /// - 20×: Vigilant prey (long-range)
    ///
    /// TODO: Replace with `body_length * dna.perception_multiplier` (Future DNA system)
    pub range: f32,

    /// Cached list of entities within perception range
    ///
    /// Updated by `update_perception_system` every frame.
    /// Avoids repeated distance calculations in behavior systems.
    ///
    /// **Performance:** Expected size ~5-20 entities in typical densities.
    /// May grow larger in crowded areas (bottlenecks, resource hotspots).
    pub nearby: Vec<Entity>,
}

impl Perception {
    /// Create perception component with specified range
    ///
    /// # Example
    /// ```
    /// use speciate::simulation::perception::Perception;
    /// // 1m creature with default 10m perception
    /// let perception = Perception::new(10.0);
    /// assert_eq!(perception.range, 10.0);
    /// ```
    pub fn new(range: f32) -> Self {
        Self {
            range,
            nearby: Vec::with_capacity(32), // Preallocate for typical densities
        }
    }

    /// Create perception with default range (10× body length)
    ///
    /// Assumes 1m body length. For DNA-driven approach,
    /// use `Perception::from_body_size(dna.body_length)` instead.
    pub fn default_range() -> Self {
        Self::new(10.0)
    }

    /// Create perception from body size using biological scaling
    ///
    /// Uses `PERCEPTION.perception_multiplier` constant (10× default).
    ///
    /// # Example
    /// ```
    /// use speciate::simulation::perception::Perception;
    /// // 0.5m creature: 0.5 * 10 = 5m perception
    /// let perception = Perception::from_body_size(0.5);
    /// assert_eq!(perception.range, 5.0);
    ///
    /// // 2.0m creature: 2.0 * 10 = 20m perception
    /// let perception = Perception::from_body_size(2.0);
    /// assert_eq!(perception.range, 20.0);
    /// ```
    ///
    /// TODO: Replace with DNA-driven multiplier (Future DNA system)
    pub fn from_body_size(body_length: f32) -> Self {
        use crate::simulation::movement::PERCEPTION;
        Self::new(body_length * PERCEPTION.perception_multiplier)
    }

    /// Check if there are any detected entities
    pub fn has_neighbors(&self) -> bool {
        !self.nearby.is_empty()
    }

    /// Get number of detected entities
    pub fn neighbor_count(&self) -> usize {
        self.nearby.len()
    }

    /// Clear cached neighbor list (called by perception system)
    pub fn clear(&mut self) {
        self.nearby.clear();
    }

    /// Add entity to neighbor list (called by perception system)
    pub fn add_neighbor(&mut self, entity: Entity) {
        self.nearby.push(entity);
    }
}

impl Default for Perception {
    fn default() -> Self {
        Self::default_range()
    }
}

/// Avoidance behavior parameters - how creature reacts to obstacles
///
/// Defines the creature's personal space and avoidance force strength.
/// Works in conjunction with `Perception` to create separation behavior.
///
/// # Force Calculation
/// ```ignore
/// if distance < personal_space:
///     force = base_force * (personal_space / distance)²  // Inverse square
///     if distance < panic_threshold:
///         force = min(force, max_panic_force)  // Cap to prevent instability
/// ```
///
/// # DNA Integration (Future DNA system)
/// Both parameters will be derived from DNA:
/// - `personal_space = body_length * dna.spacing_multiplier`
/// - `max_force` will remain constant (or scale with body mass)
#[derive(Component, Debug, Clone, Copy)]
pub struct AvoidanceBehavior {
    /// Desired minimum distance from other creatures (meters)
    ///
    /// **Current:** 2.5× body length (2.5m for 1m creature)
    /// **Biological rationale:** Solitary animal comfort zone
    ///
    /// **Future DNA gene:** `spacing_multiplier` (1.5-4.0×)
    /// - 1.5×: Colonial/tolerant species (tight spacing)
    /// - 2.5×: Solitary animal (default)
    /// - 4.0×: Territorial species (wide spacing)
    ///
    /// **Behavioral zones:**
    /// - `distance > perception_range`: Not detected
    /// - `personal_space < distance ≤ perception_range`: Detected but comfortable
    /// - `panic_threshold < distance ≤ personal_space`: Repulsion active (inverse square)
    /// - `distance ≤ panic_threshold`: Panic mode (maximum force)
    ///
    /// TODO: Replace with `body_length * dna.spacing_multiplier` (Future DNA system)
    pub personal_space: f32,

    /// Maximum avoidance force magnitude (Newtons)
    ///
    /// **Current:** 15N base, 50N panic
    /// **Rationale:** Stronger than seeking (10N) to prevent collisions
    ///
    /// **Force scaling:**
    /// - Normal avoidance: `15N * (personal_space / distance)²`
    /// - Panic (collision imminent): Capped at 50N
    ///
    /// TODO: Consider making this DNA-driven or scaling with body mass
    pub max_force: f32,
}

impl AvoidanceBehavior {
    /// Create avoidance behavior with specified parameters
    ///
    /// # Example
    /// ```
    /// use speciate::simulation::perception::AvoidanceBehavior;
    /// // Standard solitary creature
    /// let avoidance = AvoidanceBehavior::new(2.5, 15.0);
    /// assert_eq!(avoidance.personal_space, 2.5);
    ///
    /// // Territorial creature (larger personal space)
    /// let territorial = AvoidanceBehavior::new(4.0, 15.0);
    /// assert_eq!(territorial.personal_space, 4.0);
    ///
    /// // Colonial creature (smaller personal space)
    /// let colonial = AvoidanceBehavior::new(1.5, 15.0);
    /// assert_eq!(colonial.personal_space, 1.5);
    /// ```
    pub fn new(personal_space: f32, max_force: f32) -> Self {
        Self {
            personal_space,
            max_force,
        }
    }

    /// Create avoidance with default parameters
    ///
    /// Uses constants from `STEERING` and `PERCEPTION`.
    pub fn default_params() -> Self {
        use crate::simulation::movement::{PERCEPTION, STEERING};
        // Assume 1m body length for now
        let personal_space = 1.0 * PERCEPTION.personal_space;
        Self::new(personal_space, STEERING.avoidance_force)
    }

    /// Create avoidance from body size using biological scaling
    ///
    /// Uses `PERCEPTION.personal_space` constant (1.5× body length default).
    ///
    /// # Example
    /// ```
    /// use speciate::simulation::perception::AvoidanceBehavior;
    /// // 0.5m creature: 0.5 * 1.5 = 0.75m personal space
    /// let avoidance = AvoidanceBehavior::from_body_size(0.5);
    /// assert_eq!(avoidance.personal_space, 0.75);
    ///
    /// // 2.0m creature: 2.0 * 1.5 = 3.0m personal space
    /// let avoidance = AvoidanceBehavior::from_body_size(2.0);
    /// assert_eq!(avoidance.personal_space, 3.0);
    /// ```
    ///
    /// TODO: Replace with DNA-driven multiplier (Future DNA system)
    pub fn from_body_size(body_length: f32) -> Self {
        use crate::simulation::movement::{PERCEPTION, STEERING};
        let personal_space = body_length * PERCEPTION.personal_space;
        Self::new(personal_space, STEERING.avoidance_force)
    }

    /// Calculate panic threshold distance (50% of personal space)
    ///
    /// When distance < panic_threshold, avoidance force is capped
    /// at `STEERING.panic_force` to prevent physics instability.
    pub fn panic_threshold(&self) -> f32 {
        use crate::simulation::movement::PERCEPTION;
        self.personal_space * PERCEPTION.panic_threshold_ratio
    }
}

impl Default for AvoidanceBehavior {
    fn default() -> Self {
        Self::default_params()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perception_scaling_with_body_size() {
        // Small creature (0.5m)
        let small_perception = Perception::from_body_size(0.5);
        assert_eq!(small_perception.range, 5.0); // 0.5 * 10

        // Standard creature (1.0m)
        let standard_perception = Perception::from_body_size(1.0);
        assert_eq!(standard_perception.range, 10.0); // 1.0 * 10

        // Large creature (2.0m)
        let large_perception = Perception::from_body_size(2.0);
        assert_eq!(large_perception.range, 20.0); // 2.0 * 10
    }

    #[test]
    fn test_avoidance_scaling_with_body_size() {
        // Small creature (0.5m)
        let small_avoid = AvoidanceBehavior::from_body_size(0.5);
        assert_eq!(small_avoid.personal_space, 0.75); // 0.5 * 1.5

        // Standard creature (1.0m)
        let standard_avoid = AvoidanceBehavior::from_body_size(1.0);
        assert_eq!(standard_avoid.personal_space, 1.5); // 1.0 * 1.5

        // Large creature (2.0m)
        let large_avoid = AvoidanceBehavior::from_body_size(2.0);
        assert_eq!(large_avoid.personal_space, 3.0); // 2.0 * 1.5
    }

    #[test]
    fn test_panic_threshold() {
        let avoidance = AvoidanceBehavior::new(2.5, 15.0);
        let panic = avoidance.panic_threshold();

        assert_eq!(panic, 1.25); // 50% of 2.5m
        assert!(panic < avoidance.personal_space);
    }

    #[test]
    fn test_perception_neighbor_tracking() {
        let mut perception = Perception::new(10.0);

        assert!(!perception.has_neighbors());
        assert_eq!(perception.neighbor_count(), 0);

        // Add some neighbors
        perception.add_neighbor(Entity::PLACEHOLDER);
        perception.add_neighbor(Entity::PLACEHOLDER);

        assert!(perception.has_neighbors());
        assert_eq!(perception.neighbor_count(), 2);

        // Clear
        perception.clear();
        assert!(!perception.has_neighbors());
        assert_eq!(perception.neighbor_count(), 0);
    }
}
