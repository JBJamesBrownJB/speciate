//! Creature spawning events
//!
//! This module defines events for creature spawning that flow through Bevy's
//! event system. All spawn requests (state loading, dev commands, gameplay)
//! emit SpawnCreatureEvent, which is processed by a single authoritative handler.
//!
//! ## Architecture
//!
//! ```text
//! State Loading ──┐
//! Dev Commands ───┼──> SpawnCreatureEvent ──> spawn_crit_system ──> simulation.spawn_crit()
//! Gameplay ───────┘
//! ```
//!
//! ## Why Events?
//!
//! - **Single spawn gate**: All spawns go through `simulation.spawn_crit()`
//! - **Decoupling**: Request sources don't need direct access to Simulation
//! - **Testability**: Easy to inject test events without NATS/gameplay
//! - **Future-proof**: Easy to add hooks (validation, analytics, rate limiting)
//!
//! ## Exception: Snapshot Restore
//!
//! Snapshot restoration bypasses events because it needs to restore EXACT state
//! (positions, velocities, IDs) rather than spawn new creatures with defaults.

use super::builder::CritBuilder;
use bevy_ecs::event::Event;

/// Event requesting creature spawn
///
/// Emitted by:
/// - State loading (initial population from TOML config)
/// - Dev commands (admin UI spawn requests)
/// - Gameplay systems (breeding, purchases, etc. - future)
///
/// Processed by:
/// - `spawn_crit_system` which calls `simulation.spawn_crit(builder)`
///
/// # Example
///
/// ```no_run
/// use bevy_ecs::prelude::*;
/// use speciate::simulation::creatures::builder::CritBuilder;
/// use speciate::simulation::creatures::events::SpawnCreatureEvent;
///
/// fn some_system(mut events: EventWriter<SpawnCreatureEvent>) {
///     // Request a creature spawn
///     let builder = CritBuilder::new()
///         .at(100.0, 50.0)
///         .with_all_capabilities();
///
///     events.send(SpawnCreatureEvent { builder });
/// }
/// ```
#[derive(Event, Clone)]
pub struct SpawnCreatureEvent {
    /// The builder containing spawn configuration
    ///
    /// The builder specifies position, capabilities, behavior, energy, etc.
    /// The handler system will assign the creature ID when spawning.
    pub builder: CritBuilder,
}

impl SpawnCreatureEvent {
    /// Create a new spawn event from a builder
    pub fn new(builder: CritBuilder) -> Self {
        Self { builder }
    }
}
