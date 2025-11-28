use bevy::prelude::*;

#[derive(Resource)]
pub struct SimState {
    pub cell_count: usize,
    pub rebuild_requested: bool,
    pub wrap_enabled: bool,
    pub sites: Vec<Vec2>,
    pub diffusion_rates: Vec4, 
    pub decay_rates: Vec4,
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            cell_count: 50,
            rebuild_requested: true,
            wrap_enabled: true,
            sites: Vec::new(),
            // Default: varied rates to immediately show off the effect
            // R=Fast, G=Medium, B=Slow, E=Very Fast
            diffusion_rates: Vec4::new(3.0, 2.0, 1.0, 4.0),
            decay_rates: Vec4::new(0.5, 0.5, 0.5, 0.8),
        }
    }
}