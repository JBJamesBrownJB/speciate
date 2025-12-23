use super::events::SpawnCreatureEvent;
use bevy_ecs::prelude::*;
use bevy_ecs::system::Resource;
use log::info;

#[derive(Resource)]
pub struct NextCreatureId {
    pub next_id: u32,
}

impl Default for NextCreatureId {
    fn default() -> Self {
        Self { next_id: 1 }
    }
}

impl NextCreatureId {
    pub fn generate(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn set_next(&mut self, next_id: u32) {
        self.next_id = next_id;
    }
}

pub fn process_spawn_events(
    mut commands: Commands,
    mut events: EventReader<SpawnCreatureEvent>,
    mut next_id: ResMut<NextCreatureId>,
) {
    for event in events.read() {
        let id = next_id.generate();

        let bundle = event.builder.clone().build(id);

        commands.spawn(bundle);

        info!("[SPAWN] Creature #{} spawned via event system", id);
    }
}
