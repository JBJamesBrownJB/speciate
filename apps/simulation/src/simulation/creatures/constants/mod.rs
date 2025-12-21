//! Creature behavior constants - ALL tunable parameters organized by domain
//!
//! BIOLOGICAL REVIEW: Audited by zoologist-tom (Sprint 16)
//! All values have been validated against empirical animal behavior research.
//!
//! USAGE KEY:
//!   [ACTIVE]  - Currently used in simulation systems
//!   [FUTURE]  - Defined for future DNA/allometric systems, not yet wired up
//!   [LEGACY]  - Alias for backwards compatibility, migrate away from these
//!
//! Organization:
//! - physics.rs     - Movement, drag, speed, turn rate
//! - perception.rs  - FOV, range, neighbor tracking
//! - behavior.rs    - Force budgets, avoidance, seek, wander
//! - state.rs       - Energy, age, transitions

mod behavior;
mod perception;
mod physics;
mod state;

// Re-export all contents for backwards compatibility
// (modules are private to avoid name collision with creatures/components submodules)
pub use behavior::*;
pub use perception::*;
pub use physics::*;
pub use state::*;
