
use crate::simulation::creatures::dna::Dna;

#[derive(Debug, Clone)]
pub enum SimCommand {
    Spawn(u32),
    SpawnAt { x: f32, y: f32, dna: Option<Dna> },
    KillAll,
    LoadTrial { trial_name: String, randomize_dna: bool, dna: Option<Dna> },
    SelectCreatureDebug(Option<u32>),
    SetPaused(bool),
    SetTimeScale(f32),
    SetViewportBounds {
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
        margin: f32,
    },
}
