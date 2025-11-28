use bevy::prelude::*;

#[derive(Resource)]
pub struct SimState {
    pub cell_count: usize,
    pub rebuild_requested: bool,
    pub wrap_enabled: bool,
    pub sites: Vec<Vec2>,
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            cell_count: 50,
            rebuild_requested: true,
            wrap_enabled: true,
            sites: Vec::new(),
        }
    }
}
