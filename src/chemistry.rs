use crate::state::SimState;
use crate::voronoi::{CellIndex, DOMAIN_SIZE};
use bevy::prelude::*;
use rand::Rng;

// --- Components ---

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Chemicals {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub e: f32, // Emission
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct NextChemicals {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub e: f32,
}

#[derive(Component, Default, Debug, Clone)]
pub struct Neighbors {
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
    let reaction = state.reaction_matrix;

    for (_entity, neighbors, my_chem, mut next_chem) in query.iter_mut() {
        // 1. Laplacian (Diffusion)
        let mut laplacian_r = 0.0;
        let mut laplacian_g = 0.0;
        let mut laplacian_b = 0.0;
        let mut laplacian_e = 0.0;

        for &neighbor_idx in &neighbors.indices {
            if let Some(neighbor_entity) = cell_map.entities.get(neighbor_idx)
                && let Ok(neighbor_chem) = all_chemicals.get(*neighbor_entity)
            {
                laplacian_r += neighbor_chem.r - my_chem.r;
                laplacian_g += neighbor_chem.g - my_chem.g;
                laplacian_b += neighbor_chem.b - my_chem.b;
                laplacian_e += neighbor_chem.e - my_chem.e;
            }
        }

        // 2. Reaction (Alchemy)
        // dC/dt = Matrix * C
        let my_rgb = Vec3::new(my_chem.r, my_chem.g, my_chem.b);
        let delta_rgb = reaction * my_rgb; // Mat3 * Vec3

        // 3. Integration
        next_chem.r = my_chem.r + (diff.x * laplacian_r * dt) + (delta_rgb.x * dt);
        next_chem.g = my_chem.g + (diff.y * laplacian_g * dt) + (delta_rgb.y * dt);
        next_chem.b = my_chem.b + (diff.z * laplacian_b * dt) + (delta_rgb.z * dt);
        next_chem.e = my_chem.e + (diff.w * laplacian_e * dt);

        // 4. Decay
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

pub fn chemical_motility_system(
    query: Query<(&Chemicals, &Neighbors, &CellIndex)>,
    all_chemicals: Query<&Chemicals>,
    cell_map: Res<CellMap>,
    mut state: ResMut<SimState>,
    time: Res<Time>,
) {
    // Don't simulate physics if dragging/rebuilding
    // if state.rebuild_requested { return; }

    let dt = time.delta_secs();
    let forces = state.force_matrix;
    let friction = state.friction;
    let jitter = state.emission_jitter;

    // Pre-fetch sites to avoid borrow issues or cloning entire vector inside loop?
    // We have to clone the sites to mutate them safely while reading.
    let mut next_sites = state.sites.clone();
    let mut moved = false;
    let bound = (DOMAIN_SIZE / 2.0) as f32;
    let domain_width = DOMAIN_SIZE as f32;

    // Iterate all cells
    for (chem, neighbors, cell_index) in query.iter() {
        let idx = cell_index.0;
        // Safety check
        if idx >= state.sites.len() {
            continue;
        }

        let my_pos = state.sites[idx];
        let my_rgb = Vec3::new(chem.r, chem.g, chem.b);
        let mut total_force = Vec2::ZERO;

        // 1. Temperature / Brownian Motion
        // Driven by Emission channel
        if jitter > 0.0 {
            let mut rng = rand::thread_rng();
            // Random direction * intensity * emission
            let noise = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
            total_force += noise * jitter * chem.e * 50.0;
        }

        // 2. Interactive Forces
        for &n_idx in &neighbors.indices {
            if let Some(n_entity) = cell_map.entities.get(n_idx)
                && let Ok(n_chem) = all_chemicals.get(*n_entity)
            {
                let n_pos = state.sites[n_idx];
                let mut dir = n_pos - my_pos;

                // Torus Wrap Distance Logic
                if state.wrap_enabled {
                    if dir.x > bound {
                        dir.x -= domain_width;
                    } else if dir.x < -bound {
                        dir.x += domain_width;
                    }

                    if dir.y > bound {
                        dir.y -= domain_width;
                    } else if dir.y < -bound {
                        dir.y += domain_width;
                    }
                }

                let dist_sq = dir.length_squared();
                if dist_sq > 0.0001 {
                    let dist = dist_sq.sqrt();
                    let norm_dir = dir / dist;

                    // Force Calculation:
                    // Strength = Self(RGB) * Matrix * Neighbor(RGB)
                    // This gives a scalar: >0 Attract, <0 Repel
                    let n_rgb = Vec3::new(n_chem.r, n_chem.g, n_chem.b);

                    // Transpose isn't built-in for Vec3 dot logic easily in this algebra,
                    // so: Strength = Self dot (Matrix * Neighbor)
                    let interaction_vec = forces * n_rgb;
                    let strength = my_rgb.dot(interaction_vec);

                    // Normalize by distance?
                    // Usually force falls off with distance (gravity/magnetic)
                    // Let's say Force ~ Strength / dist
                    // Clamp distance to avoid singularity
                    let safe_dist = dist.max(0.1);

                    total_force += norm_dir * (strength / safe_dist) * 10.0;
                }
            }
        }

        // 3. Integrate Position
        // F = ma, assume unit mass. V += F * dt.
        // Friction: V *= (1 - friction)
        // Position += V * dt.
        // Simplified: Position += TotalForce * (1-friction) * dt

        let velocity = total_force * (1.0 - friction);

        if velocity.length_squared() > 0.00001 {
            let mut new_pos = my_pos + velocity * dt;

            // 4. Boundary Wrapping
            if state.wrap_enabled {
                // rem_euclid handles negative wrapping correctly
                new_pos.x = (new_pos.x + bound).rem_euclid(domain_width) - bound;
                new_pos.y = (new_pos.y + bound).rem_euclid(domain_width) - bound;
            } else {
                // Hard clamp
                new_pos.x = new_pos.x.clamp(-bound, bound);
                new_pos.y = new_pos.y.clamp(-bound, bound);
            }

            next_sites[idx] = new_pos;
            moved = true;
        }
    }

    if moved {
        state.sites = next_sites;
        state.rebuild_requested = true;
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
        current.r = next.r;
        current.g = next.g;
        current.b = next.b;
        current.e = next.e;

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.base_color = Color::srgb(current.r, current.g, current.b);
            mat.emissive = LinearRgba::new(current.r, current.g, current.b, current.e * 2.0);
        }
    }
}

pub fn on_click_splash(trigger: On<Pointer<Press>>, mut query: Query<&mut Chemicals>) {
    if let Ok(mut chem) = query.get_mut(trigger.original_event_target()) {
        chem.r += 5.0;
        chem.g += 5.0;
        chem.b += 5.0;
        chem.e += 50.0; // Flash
        info!("Splashed cell");
    }
}
