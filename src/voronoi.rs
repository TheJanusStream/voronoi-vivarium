use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashSet;
use voronator::VoronoiDiagram;
use voronator::delaunator::{self, Point};

use crate::chemistry::{CellMap, Chemicals, Neighbors, NextChemicals};
use crate::state::SimState;

pub const DOMAIN_SIZE: f64 = 20.0;

// 1. Component to track identity across mesh rebuilds
#[derive(Component)]
pub struct CellIndex(pub usize);

pub fn spawn_mesh_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<SimState>,
    mut cell_map: ResMut<CellMap>,
    // Note: We don't query for cleanup anymore, we manage IDs in cell_map
) {
    if !state.rebuild_requested {
        return;
    }

    // 1. Synchronize Sites with Cell Count
    let mut rng = rand::thread_rng();
    let half_size = DOMAIN_SIZE / 2.0;
    let current_count = state.sites.len();
    let target_count = state.cell_count;

    if current_count < target_count {
        // Grow: Add new random sites
        for _ in 0..(target_count - current_count) {
            state.sites.push(Vec2::new(
                rng.gen_range(-half_size..half_size) as f32,
                rng.gen_range(-half_size..half_size) as f32,
            ));
        }
    } else if current_count > target_count {
        // Shrink: Truncate the list
        state.sites.truncate(target_count);
    }

    // 2. Prepare Computation Points (Ghost Strategy)
    // Map Vec2 (f32) to Point (f64) for voronator
    let sites: Vec<Point> = state
        .sites
        .iter()
        .map(|v| Point {
            x: v.x as f64,
            y: v.y as f64,
        })
        .collect();
    let mut computation_points = sites.clone();

    if state.wrap_enabled {
        let offsets = [
            (-DOMAIN_SIZE, -DOMAIN_SIZE),
            (0.0, -DOMAIN_SIZE),
            (DOMAIN_SIZE, -DOMAIN_SIZE),
            (-DOMAIN_SIZE, 0.0),
            (DOMAIN_SIZE, 0.0),
            (-DOMAIN_SIZE, DOMAIN_SIZE),
            (0.0, DOMAIN_SIZE),
            (DOMAIN_SIZE, DOMAIN_SIZE),
        ];
        for offset in offsets {
            for site in &sites {
                computation_points.push(Point {
                    x: site.x + offset.0,
                    y: site.y + offset.1,
                });
            }
        }
    }

    // 3. Calculate Topology (Neighbors)
    let mut adjacency: Vec<HashSet<usize>> = vec![HashSet::new(); state.cell_count];
    if let Some(triangulation) = delaunator::triangulate(&computation_points) {
        for i in (0..triangulation.triangles.len()).step_by(3) {
            let p = [
                triangulation.triangles[i],
                triangulation.triangles[i + 1],
                triangulation.triangles[i + 2],
            ];
            for &u in &p {
                for &v in &p {
                    if u == v {
                        continue;
                    }
                    if u < state.cell_count {
                        let v_real = v % state.cell_count;
                        if u != v_real {
                            adjacency[u].insert(v_real);
                        }
                    }
                }
            }
        }
    }

    // 4. Compute Geometry & Spawn/Update
    let bound = DOMAIN_SIZE * 2.0;
    let diagram = VoronoiDiagram::new(
        &Point {
            x: -bound,
            y: -bound,
        },
        &Point { x: bound, y: bound },
        &computation_points,
    );

    if let Some(diagram) = diagram {
        // Entity Recycling Check
        let reuse_entities = cell_map.entities.len() == state.cell_count;

        if !reuse_entities {
            // Hard reset if count changed
            for e in &cell_map.entities {
                commands.entity(*e).despawn();
            }
            cell_map.entities.clear();
        }

        let mut rng = rand::thread_rng();
        let range_max = (DOMAIN_SIZE / 2.0) as f32;

        for (i, cell) in diagram.cells().iter().take(state.cell_count).enumerate() {
            let points: Vec<Vec3> = cell
                .points()
                .iter()
                .map(|p| Vec3::new(p.x as f32, 0.0, p.y as f32))
                .collect();

            if points.len() < 3 {
                continue;
            } // Skip degenerate cells

            // Triangulate the polygon fan
            let mut indices = Vec::new();
            for j in 1..points.len() - 1 {
                indices.push(0);
                indices.push(j as u32);
                indices.push((j + 1) as u32);
            }

            let mut mesh = Mesh::new(
                bevy::mesh::PrimitiveTopology::TriangleList,
                bevy::asset::RenderAssetUsages::RENDER_WORLD
                    | bevy::asset::RenderAssetUsages::MAIN_WORLD,
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
            mesh.insert_indices(bevy::mesh::Indices::U32(indices));
            mesh.compute_smooth_normals();

            let mesh_handle = meshes.add(mesh);

            if reuse_entities {
                // RECYCLE PATH
                let id = cell_map.entities[i];
                commands
                    .entity(id)
                    .insert(Mesh3d(mesh_handle)) // Hot-swap Mesh
                    .insert(Neighbors {
                        indices: adjacency[i].iter().cloned().collect(),
                    }) // Update Neighbors
                    .remove::<Aabb>(); // CRITICAL: Force AABB regeneration for picking!
            } else {
                // SPAWN PATH: Gradient Initialization
                let pos = state.sites[i];

                // Normalize -10..10 to 0..1
                let nx = (pos.x / range_max + 1.0) * 0.5;
                let ny = (pos.y / range_max + 1.0) * 0.5;
                let dist_center = 1.0 - (pos.length() / range_max).clamp(0.0, 1.0);

                // Base Colors: Red (X), Green (Y), Blue (Center)
                let r_base = nx;
                let g_base = ny;
                let b_base = dist_center;

                // Add Noise
                let noise = 0.05;

                let chem = Chemicals {
                    r: (r_base + rng.gen_range(-noise..noise)).clamp(0.0, 1.0),
                    g: (g_base + rng.gen_range(-noise..noise)).clamp(0.0, 1.0),
                    b: (b_base + rng.gen_range(-noise..noise)).clamp(0.0, 1.0),
                    e: rng.r#gen(), // Emission can be random
                };

                let id = commands
                    .spawn((
                        Mesh3d(mesh_handle),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::BLACK,
                            metallic: 0.1,
                            perceptual_roughness: 0.8,
                            cull_mode: None,
                            ..default()
                        })),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        // Components
                        chem,
                        NextChemicals::default(),
                        Neighbors {
                            indices: adjacency[i].iter().cloned().collect(),
                        },
                        CellIndex(i), // Track Identity
                    ))
                    .observe(crate::chemistry::on_click_splash) // Left Click
                    .observe(on_cell_drag) // Drag
                    .id();

                cell_map.entities.push(id);
            }
        }
    }

    state.rebuild_requested = false;
}

// 5. The Drag Handler
pub fn on_cell_drag(
    trigger: On<Pointer<Drag>>,
    mut state: ResMut<SimState>,
    query: Query<&CellIndex>,
) {
    // Identify which site we are dragging
    if let Ok(cell_idx) = query.get(trigger.original_event_target()) {
        let idx = cell_idx.0;

        // Get drag delta (screen space)
        let drag_event = trigger.event();
        let delta = drag_event.delta;

        // Map Screen Delta to World Plane (X/Z)
        // Camera is looking DOWN Y axis.
        // Screen X+ (Right) -> World X+ (Right)
        // Screen Y+ (Down)  -> World Z+ (Toward/Back depending on camera up)
        // With standard look_at(ZERO, Y), screen Up (Y-) moves into the screen (Z-)

        // Sensitivity factor (approximate for height=20.0)
        let sensitivity = 0.05;

        if let Some(site) = state.sites.get_mut(idx) {
            site.x += delta.x * sensitivity;
            site.y -= delta.y * sensitivity;

            // Clamp to domain
            let bound = (DOMAIN_SIZE / 2.0) as f32;
            site.x = site.x.clamp(-bound, bound);
            site.y = site.y.clamp(-bound, bound);
        }

        // Request immediate rebuild
        state.rebuild_requested = true;
    }
}
