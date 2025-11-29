use crate::state::SimState;
use bevy::{post_process::bloom::Bloom, prelude::*};
use bevy_egui::{EguiContexts, egui};
use bevy_panorbit_camera::PanOrbitCamera;
use rand::Rng;

pub fn setup_scene(mut commands: Commands) {
    // Lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform {
            translation: Vec3::new(10.0, 10.0, 10.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::PI / 4.),
            ..default()
        },
    ));

    // Camera
    commands.spawn((
        Transform::from_xyz(0.0, -20.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            focus: Vec3::ZERO,
            button_orbit: MouseButton::Right,
            button_pan: MouseButton::Middle,
            ..default()
        },
        MeshPickingCamera,
        Bloom::default(),
    ));
}

// Helper for rendering 3x3 Matrix UI
fn matrix_ui(ui: &mut egui::Ui, matrix: &mut Mat3, id_salt: &str) {
    egui::Grid::new(id_salt).min_col_width(30.0).show(ui, |ui| {
        // Header Row: Input Sources
        ui.label(""); // Corner
        ui.label(egui::RichText::new("R In").color(egui::Color32::RED));
        ui.label(egui::RichText::new("G In").color(egui::Color32::GREEN));
        ui.label(egui::RichText::new("B In").color(egui::Color32::BLUE));
        ui.end_row();

        // Rows: Output Targets
        let labels = [
            ("R Out", egui::Color32::RED),
            ("G Out", egui::Color32::GREEN),
            ("B Out", egui::Color32::BLUE),
        ];

        for r in 0..3 {
            ui.label(egui::RichText::new(labels[r].0).color(labels[r].1));
            // Bevy Mat3 is Column-Major [col][row]
            ui.add(
                egui::DragValue::new(&mut matrix.x_axis[r])
                    .speed(0.01)
                    .fixed_decimals(2),
            );
            ui.add(
                egui::DragValue::new(&mut matrix.y_axis[r])
                    .speed(0.01)
                    .fixed_decimals(2),
            );
            ui.add(
                egui::DragValue::new(&mut matrix.z_axis[r])
                    .speed(0.01)
                    .fixed_decimals(2),
            );
            ui.end_row();
        }
    });
}

pub fn ui_system(mut contexts: EguiContexts, mut state: ResMut<SimState>) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Window::new("Vivarium Controls")
            .default_width(300.0)
            .show(ctx, |ui| {
                egui::CollapsingHeader::new("Topology & Simulation")
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut count = state.cell_count;
                        if ui
                            .add(egui::Slider::new(&mut count, 10..=1500).text("Cell Count"))
                            .changed()
                        {
                            state.cell_count = count;
                            state.rebuild_requested = true;
                        }

                        if ui.checkbox(&mut state.wrap_enabled, "Torus Wrap").changed() {
                            state.sites.clear();
                            state.rebuild_requested = true;
                        }

                        if ui.button("Regenerate Graph").clicked() {
                            state.rebuild_requested = true;
                        }

                        ui.add(egui::Slider::new(&mut state.friction, 0.0..=1.0).text("Friction"));
                        ui.add(
                            egui::Slider::new(&mut state.emission_jitter, 0.0..=1.0)
                                .text("Temp (Jitter)"),
                        );
                    });

                ui.separator();

                egui::CollapsingHeader::new("Reaction Rules (Alchemy)")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Equation: dC/dt = M * C");
                            if ui.button("🎲 Randomize").clicked() {
                                let mut rng = rand::thread_rng();
                                for c in 0..3 {
                                    for r in 0..3 {
                                        // Random range -1.0 to 1.0
                                        // Positive = Catalyze, Negative = Inhibit
                                        let val = rng.gen_range(-1.0..1.0);
                                        match c {
                                            0 => state.reaction_matrix.x_axis[r] = val,
                                            1 => state.reaction_matrix.y_axis[r] = val,
                                            2 => state.reaction_matrix.z_axis[r] = val,
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        });
                        matrix_ui(ui, &mut state.reaction_matrix, "reaction_matrix");
                    });

                ui.separator();

                egui::CollapsingHeader::new("Motility Forces")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Equation: Force = Self * M * Neighbor");
                            if ui.button("🎲 Randomize").clicked() {
                                let mut rng = rand::thread_rng();
                                for c in 0..3 {
                                    for r in 0..3 {
                                        // Random range -0.5 to 0.5 (forces need to be gentler)
                                        let val = rng.gen_range(-0.5..0.5);
                                        match c {
                                            0 => state.force_matrix.x_axis[r] = val,
                                            1 => state.force_matrix.y_axis[r] = val,
                                            2 => state.force_matrix.z_axis[r] = val,
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        });
                        ui.label("Pos = Attract, Neg = Repel");
                        matrix_ui(ui, &mut state.force_matrix, "force_matrix");
                    });

                ui.separator();

                egui::CollapsingHeader::new("Base Physics")
                    .default_open(true)
                    .show(ui, |ui| {
                        if ui.button("🎲 Randomize Physics").clicked() {
                            let mut rng = rand::thread_rng();
                            // Diffusion: 0.5 to 4.0
                            state.diffusion_rates = Vec4::new(
                                rng.gen_range(0.1..2.0),
                                rng.gen_range(0.1..2.0),
                                rng.gen_range(0.1..2.0),
                                rng.gen_range(0.1..4.0), // Emission diffuses faster
                            );
                            // Decay: 0.01 to 0.3 (keep it low to sustain life)
                            state.decay_rates = Vec4::new(
                                rng.gen_range(0.01..0.4),
                                rng.gen_range(0.01..0.4),
                                rng.gen_range(0.01..0.4),
                                rng.gen_range(0.1..0.8), // Emission decays faster
                            );
                        }

                        egui::Grid::new("diff_decay_grid")
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Ch");
                                ui.label("Diff");
                                ui.label("Decay");
                                ui.end_row();

                                ui.label("R");
                                ui.add(egui::Slider::new(&mut state.diffusion_rates.x, 0.0..=2.0));
                                ui.add(egui::Slider::new(&mut state.decay_rates.x, 0.0..=1.0));
                                ui.end_row();

                                ui.label("G");
                                ui.add(egui::Slider::new(&mut state.diffusion_rates.y, 0.0..=2.0));
                                ui.add(egui::Slider::new(&mut state.decay_rates.y, 0.0..=1.0));
                                ui.end_row();

                                ui.label("B");
                                ui.add(egui::Slider::new(&mut state.diffusion_rates.z, 0.0..=2.0));
                                ui.add(egui::Slider::new(&mut state.decay_rates.z, 0.0..=1.0));
                                ui.end_row();

                                ui.label("E");
                                ui.add(egui::Slider::new(&mut state.diffusion_rates.w, 0.0..=4.0));
                                ui.add(egui::Slider::new(&mut state.decay_rates.w, 0.0..=2.0));
                                ui.end_row();
                            });
                    });
            });
    }
}
