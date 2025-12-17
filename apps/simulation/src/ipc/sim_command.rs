
#[derive(Debug, Clone)]
pub enum SimCommand {
    Spawn(u32),
    SpawnAt { x: f32, y: f32 },
    KillAll,
    LoadTrial { trial_name: String },
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
