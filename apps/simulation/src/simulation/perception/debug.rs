//! Debug types for perception visualization (dev-tools only)
//!
//! These types are used for developer tooling and visualization,
//! not for core simulation logic.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Default)]
pub struct PerceptionDebugTarget(pub Option<Entity>);

impl PerceptionDebugTarget {
    pub fn set_by_crit_id(&mut self, crit_id: Option<u32>, lookup: impl Fn(u32) -> Option<Entity>) {
        self.0 = crit_id.and_then(lookup);
    }

    pub fn clear(&mut self) {
        self.0 = None;
    }

    pub fn get(&self) -> Option<Entity> {
        self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NeighborDebugInfo {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueriedCell {
    pub x: i32,
    pub y: i32,
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerceptionDebugSnapshot {
    pub entity_id: u32,
    pub x: f32,
    pub y: f32,
    pub perception_range: f32,
    pub query_radius: f32,
    pub fov_angle: f32,
    pub rotation: f32,
    pub ax: f32,
    pub ay: f32,
    pub neighbors: Vec<NeighborDebugInfo>,
    pub queried_cells: Vec<QueriedCell>,
    pub checked_cells: Vec<QueriedCell>,
    pub creature_cell: QueriedCell,
}

impl PerceptionDebugSnapshot {
    pub fn clear(&mut self) {
        self.entity_id = 0;
        self.x = 0.0;
        self.y = 0.0;
        self.perception_range = 0.0;
        self.query_radius = 0.0;
        self.fov_angle = 0.0;
        self.rotation = 0.0;
        self.ax = 0.0;
        self.ay = 0.0;
        self.neighbors.clear();
        self.queried_cells.clear();
        self.checked_cells.clear();
        self.creature_cell = QueriedCell::default();
    }

    pub fn update(
        &mut self,
        entity_id: u32,
        x: f32,
        y: f32,
        perception_range: f32,
        query_radius: f32,
        fov_angle: f32,
        rotation: f32,
        ax: f32,
        ay: f32,
        neighbors: impl IntoIterator<Item = NeighborDebugInfo>,
        queried_cells: impl IntoIterator<Item = QueriedCell>,
        checked_cells: impl IntoIterator<Item = QueriedCell>,
        creature_cell: QueriedCell,
    ) {
        self.entity_id = entity_id;
        self.x = x;
        self.y = y;
        self.perception_range = perception_range;
        self.query_radius = query_radius;
        self.fov_angle = fov_angle;
        self.rotation = rotation;
        self.ax = ax;
        self.ay = ay;
        self.neighbors.clear();
        self.neighbors.extend(neighbors);
        self.queried_cells.clear();
        self.queried_cells.extend(queried_cells);
        self.checked_cells.clear();
        self.checked_cells.extend(checked_cells);
        self.creature_cell = creature_cell;
    }
}
