use crate::simulation::creatures::dna::Dna;
#[cfg(feature = "dev-tools")]
use crossbeam_channel::Sender;

/// L1 cell metadata returned by on-demand queries
#[derive(Debug, Clone, Default)]
pub struct L1CellInfo {
    /// Cell X coordinate (grid units, not world units)
    pub cell_x: i32,
    /// Cell Y coordinate (grid units, not world units)
    pub cell_y: i32,
    /// World X coordinate of cell center (meters)
    pub world_center_x: f32,
    /// World Y coordinate of cell center (meters)
    pub world_center_y: f32,
    /// Number of creatures in this cell
    pub creature_count: u32,
    /// Total mass of all creatures in this cell
    pub total_mass: f32,
    /// Maximum creature size in this cell (length in meters)
    pub max_size: f32,
    /// Average creature size in meters (derived from mass)
    pub avg_size: f32,
}

#[derive(Debug, Clone)]
pub enum SimCommand {
    Spawn(u32),
    SpawnAt {
        x: f32,
        y: f32,
        dna: Option<Dna>,
    },
    KillAll,
    LoadTrial {
        trial_name: String,
        randomize_dna: bool,
        dna: Option<Dna>,
    },
    SelectCreatureDebug(Option<u32>),
    SetPaused(bool),
    SetTimeScale(f32),
    /// Set frequency divisor for a cognitive system (perception, behavior, steering)
    SetSystemFrequency { system: String, divisor: u8 },
    SetViewportBounds {
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
        margin: f32,
    },
    /// Query L1 cell at world position (dev-tools only)
    #[cfg(feature = "dev-tools")]
    QueryL1Cell {
        world_x: f32,
        world_y: f32,
        response_tx: Sender<Option<L1CellInfo>>,
    },
}
