pub mod grid;
pub mod systems;

pub use grid::{PlantGrid, FLOATS_PER_PLANT_CELL, P0_CELL_SIZE};
pub use systems::update_plants;
