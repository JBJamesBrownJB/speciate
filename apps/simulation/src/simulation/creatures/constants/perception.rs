//! Perception constants
//!
//! Field of view, range, and neighbor tracking parameters.
//!
//! BIOLOGICAL BASIS:
//! Perception range scales with body size. FOV creates predator/prey trade-off:
//! - Predators: Narrow FOV (60-120°), binocular depth perception, longer range
//! - Prey: Wide FOV (270-340°), early threat detection, shorter focused range

/// [ACTIVE] Maximum neighbors tracked simultaneously.
/// VALIDATED: Fish schools 4-7, Starling murmurations 6-7, Primates 3-5.
pub const MAX_PERCEIVED_NEIGHBORS: usize = 7;

/// [ACTIVE] Number of ticks to skip perception when neighbor cache is full.
/// Crowded creatures skip this many consecutive ticks after filling their cache.
/// 1 = skip every other tick (50% reduction)
/// 2 = skip 2 ticks per perception (66% reduction)
/// Higher values = more performance, more stale neighbor data.
pub const PERCEPTION_SKIP_TICKS: u8 = 1;

/// [ACTIVE] Perception range = body_length × this multiplier.
/// 10× gives 1m creature a 10m range - conservative, good for dense environments.
pub const PERCEPTION_MULTIPLIER: f32 = 10.0;

/// [ACTIVE] Neighbor selection strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborSortMode {
    /// Fast: Take first K neighbors found (pseudo-random order)
    PseudoRandom,
    /// Accurate: Collect all, sort by distance, keep K closest
    Topological,
}

pub const NEIGHBOR_SORT_MODE: NeighborSortMode = NeighborSortMode::Topological;

// =============================================================================
// FIELD OF VIEW
// =============================================================================

/// [ACTIVE] Range compensation for narrow FOV.
/// VALIDATED: Creates elegant predator-prey emergent design.
/// With exponent 0.4:
///   - 45° FOV gets ~2.5× range bonus (specialist predator)
///   - 180° FOV gets 1× baseline (generalist)
///   - 340° FOV gets ~0.8× range (extreme prey adaptation)
pub const FOV_RANGE_EXPONENT: f32 = 0.4;

/// [FUTURE] Minimum FOV - specialist predator (eagles ~45°, owls ~55°).
pub const MIN_FOV_DEGREES: f32 = 45.0;

/// [FUTURE] Maximum FOV - extreme prey (horses ~350°, rabbits ~360°).
pub const MAX_FOV_DEGREES: f32 = 340.0;

/// [ACTIVE] Default FOV - generalist omnivore (bears, dogs, wolves).
pub const DEFAULT_FOV_DEGREES: f32 = 180.0;
