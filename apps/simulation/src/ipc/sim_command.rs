
#[derive(Debug, Clone)]
pub enum SimCommand {
    Spawn(u32),
    SpawnAt { x: f32, y: f32 },
    KillAll,
    LoadTrial { trial_name: String },
}
