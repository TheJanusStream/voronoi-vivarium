use crate::state::SimState;
use bevy::prelude::*;

// --- Components ---

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Chemicals {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub e: f32, // Emission
}

// The "Future" state buffer
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct NextChemicals {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub e: f32,
}

#[derive(Component, Default, Debug, Clone)]
pub struct Neighbors {
    // Indices of neighbors in the CellMap
    pub indices: Vec<usize>,
}

// --- Resources ---

#[derive(Resource, Default)]
pub struct CellMap {
    pub entities: Vec<Entity>,
}

// --- Systems ---

pub fn reaction_diffusion_system(
    mut query: Query<(Entity, &Neighbors, &Chemicals, &mut NextChemicals)>,
    all_chemicals: Query<&Chemicals>,
    cell_map: Res<CellMap>,
    time: Res<Time>,
    state: Res<SimState>,
) {
    let dt = time.delta_secs();
    let diff = state.diffusion_rates;
    let decay = state.decay_rates;

    for (_entity, neighbors, my_chem, mut next_chem) in query.iter_mut() {
        let mut laplacian_r = 0.0;
        let mut laplacian_g = 0.0;
        let mut laplacian_b = 0.0;
        let mut laplacian_e = 0.0;

        for &neighbor_idx in &neighbors.indices {
            if let Some(neighbor_entity) = cell_map.entities.get(neighbor_idx) {
                if let Ok(neighbor_chem) = all_chemicals.get(*neighbor_entity) {
                    laplacian_r += neighbor_chem.r - my_chem.r;
                    laplacian_g += neighbor_chem.g - my_chem.g;
                    laplacian_b += neighbor_chem.b - my_chem.b;
                    laplacian_e += neighbor_chem.e - my_chem.e;
                }
            }
        }

        // Normalize by neighbor count to get average difference (optional, but more stable)
        // let neighbor_count = neighbors.indices.len() as f32;
        // if neighbor_count > 0.0 { ... }
        // For pure diffusion, just summing is standard if distances are roughly equal.

        // 1. Diffusion Step
        next_chem.r = my_chem.r + (diff.x * laplacian_r * dt);
        next_chem.g = my_chem.g + (diff.y * laplacian_g * dt);
        next_chem.b = my_chem.b + (diff.z * laplacian_b * dt);
        next_chem.e = my_chem.e + (diff.w * laplacian_e * dt);

        // 2. Reaction / Decay Step (Simple logic for now)
        // Global decay prevents saturation
        next_chem.r -= decay.x * dt * next_chem.r;
        next_chem.g -= decay.y * dt * next_chem.g;
        next_chem.b -= decay.z * dt * next_chem.b;
        next_chem.e -= decay.w * dt * next_chem.e;

        // Clamp
        next_chem.r = next_chem.r.clamp(0.0, 1.0);
        next_chem.g = next_chem.g.clamp(0.0, 1.0);
        next_chem.b = next_chem.b.clamp(0.0, 1.0);
        next_chem.e = next_chem.e.clamp(0.0, 1.0);
    }
}

pub fn state_update_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(
        &mut Chemicals,
        &NextChemicals,
        &MeshMaterial3d<StandardMaterial>,
    )>,
) {
    for (mut current, next, mat_handle) in query.iter_mut() {
        // Double-Buffer Swap: Explicitly copy fields
        current.r = next.r;
        current.g = next.g;
        current.b = next.b;
        current.e = next.e;

        // Update Visuals
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.base_color = Color::srgb(current.r, current.g, current.b);
            // Boost emission slightly for glow effect
            mat.emissive = LinearRgba::new(current.r, current.g, current.b, current.e * 2.0);
        }
    }
}

pub fn on_click_splash(trigger: On<Pointer<Press>>, mut query: Query<&mut Chemicals>) {
    if let Ok(mut chem) = query.get_mut(trigger.original_event_target()) {
        chem.r += 50.0; // Massive Red boost
        chem.g += 50.0; // Massive Green boost
        chem.b += 50.0; // White hot
        chem.e += 50.0; // Extreme emission flash
        info!("Splashed cell {:?}", trigger.original_event_target());
    }
}
