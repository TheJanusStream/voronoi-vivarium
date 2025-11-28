use bevy::prelude::*;
use rand::Rng;
use std::collections::HashSet;
use voronator::VoronoiDiagram;
use voronator::delaunator::{self, Point};

use crate::chemistry::{CellMap, Chemicals, Neighbors, NextChemicals};
use crate::state::SimState;

pub const DOMAIN_SIZE: f64 = 20.0;

pub fn spawn_mesh_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<SimState>,
    mut cell_map: ResMut<CellMap>,
    query: Query<Entity, With<Mesh3d>>,
) {
    if !state.rebuild_requested {
        return;
    }

    // 1. Cleanup
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    cell_map.entities.clear();

    info!(
        "Rebuilding Voronoi Graph with {} cells...",
        state.cell_count
    );

    let mut rng = rand::thread_rng();
    let half_size = DOMAIN_SIZE / 2.0;

    // 2. Generate Real Sites
    let sites: Vec<Point> = (0..state.cell_count)
        .map(|_| Point {
            x: rng.gen_range(-half_size..half_size),
            y: rng.gen_range(-half_size..half_size),
        })
        .collect();

    state.sites = sites
        .iter()
        .map(|p| Vec2::new(p.x as f32, p.y as f32))
        .collect();

    // 3. Prepare Computation Points
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

    // 4. Calculate Topology
    let mut adjacency: Vec<HashSet<usize>> = vec![HashSet::new(); state.cell_count];

    if let Some(triangulation) = delaunator::triangulate(&computation_points) {
        for i in (0..triangulation.triangles.len()).step_by(3) {
            let p0 = triangulation.triangles[i];
            let p1 = triangulation.triangles[i + 1];
            let p2 = triangulation.triangles[i + 2];

            let points = [p0, p1, p2];

            for &u_idx in &points {
                for &v_idx in &points {
                    if u_idx == v_idx {
                        continue;
                    }
                    if u_idx < state.cell_count {
                        let v_real = v_idx % state.cell_count;
                        if u_idx != v_real {
                            adjacency[u_idx].insert(v_real);
                        }
                    }
                }
            }
        }
    }

    // 5. Compute Geometry
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
        let mut spawned_count = 0;
        for (i, cell) in diagram.cells().iter().take(state.cell_count).enumerate() {
            let points: Vec<Vec3> = cell
                .points()
                .iter()
                .map(|p| Vec3::new(p.x as f32, 0.0, p.y as f32))
                .collect();

            if points.len() < 3 {
                let id = commands.spawn(Transform::default()).id();
                cell_map.entities.push(id);
                continue;
            }

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

            let chem = Chemicals {
                r: rng.r#gen(),
                g: rng.r#gen(),
                b: rng.r#gen(),
                e: rng.r#gen(),
            };

            let id = commands
                .spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::BLACK,
                        metallic: 0.1,
                        perceptual_roughness: 0.8,
                        cull_mode: None, // Disable culling to ensure visibility
                        ..default()
                    })),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    chem,
                    NextChemicals::default(),
                    Neighbors {
                        indices: adjacency[i].iter().cloned().collect(),
                    },
                ))
                .observe(crate::chemistry::on_click_splash)
                .id();

            cell_map.entities.push(id);
            spawned_count += 1;
        }
        info!("Spawned {} visual cells.", spawned_count);
    }

    state.rebuild_requested = false;
}
