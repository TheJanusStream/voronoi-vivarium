use bevy::prelude::*;

#[derive(Resource)]
pub struct SimState {
    pub cell_count: usize,
    pub rebuild_requested: bool,
    pub wrap_enabled: bool,
    pub sites: Vec<Vec2>,
    pub diffusion_rates: Vec4,
    pub decay_rates: Vec4,
    pub reaction_matrix: Mat3,
    pub force_matrix: Mat3,
    pub friction: f32,
    pub emission_jitter: f32,
}

impl Default for SimState {
    fn default() -> Self {
        // --- The "Chromatic Pursuit" Preset ---

        // 1. Reaction (Alchemy): Cyclic Predation
        // Columns = Input (Prey), Rows = Output (Predator)
        // 0.5 value means "Presence of Col boosts Row"
        let reaction = Mat3::from_cols(
            Vec3::new(0.0, 0.0, 1.0), // Col 0 (Red In): Feeds Blue (Row 2)
            Vec3::new(1.0, 0.0, 0.0), // Col 1 (Green In): Feeds Red (Row 0)
            Vec3::new(0.0, 1.0, 0.0), // Col 2 (Blue In): Feeds Green (Row 1)
        ) * 0.4; // Scale intensity

        // 2. Forces (Motility): Cyclic Attraction & Self-Repulsion
        // Columns = Neighbor (Target), Rows = Self (Agent)
        // Positive = Chase, Negative = Flee
        let force = Mat3::from_cols(
            Vec3::new(-0.2, 0.3, -0.1), // Neighbor is Red: Repels Red, Attracts Blue
            Vec3::new(-0.1, -0.2, 0.3), // Neighbor is Green: Attracts Red, Repels Green
            Vec3::new(0.3, -0.1, -0.2), // Neighbor is Blue: Repels Blue, Attracts Green
        ) * 0.05;

        Self {
            cell_count: 200, // Increased for better pattern resolution
            rebuild_requested: true,
            wrap_enabled: true,
            sites: Vec::new(),

            // Lower diffusion to prevent flickering (Stability)
            diffusion_rates: Vec4::new(0.2, 0.3, 0.4, 0.5),

            // Decay balances the reaction growth
            decay_rates: Vec4::new(0.3, 0.4, 0.5, 0.6),

            reaction_matrix: reaction,
            force_matrix: force,

            friction: 0.8,        // Fluid movement
            emission_jitter: 0.1, // Slight temperature noise
        }
    }
}
