use crate::state::SimState;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use bevy_panorbit_camera::PanOrbitCamera;

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
    ));
}

pub fn ui_system(mut contexts: EguiContexts, mut state: ResMut<SimState>) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Window::new("Vivarium Controls").show(ctx, |ui| {
            ui.label("Topology Settings");

            let mut count = state.cell_count;
            if ui
                .add(egui::Slider::new(&mut count, 10..=500).text("Cell Count"))
                .changed()
            {
                state.cell_count = count;
                state.rebuild_requested = true;
            }

            if ui
                .checkbox(&mut state.wrap_enabled, "Torus Wrap (Ghost Points)")
                .changed()
            {
                state.sites.clear(); 
                state.rebuild_requested = true;
            }

            if ui.button("Regenerate Graph").clicked() {
                state.rebuild_requested = true;
            }

            ui.separator();
            ui.label("Chemical Physics");

            // Grid layout for neatness
            egui::Grid::new("physics_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Channel");
                    ui.label("Diffusion");
                    ui.label("Decay");
                    ui.end_row();

                    // Red
                    ui.label(egui::RichText::new("Red").color(egui::Color32::RED));
                    ui.add(egui::Slider::new(&mut state.diffusion_rates.x, 0.0..=5.0));
                    ui.add(egui::Slider::new(&mut state.decay_rates.x, 0.0..=2.0));
                    ui.end_row();

                    // Green
                    ui.label(egui::RichText::new("Green").color(egui::Color32::GREEN));
                    ui.add(egui::Slider::new(&mut state.diffusion_rates.y, 0.0..=5.0));
                    ui.add(egui::Slider::new(&mut state.decay_rates.y, 0.0..=2.0));
                    ui.end_row();

                    // Blue
                    ui.label(egui::RichText::new("Blue").color(egui::Color32::BLUE));
                    ui.add(egui::Slider::new(&mut state.diffusion_rates.z, 0.0..=5.0));
                    ui.add(egui::Slider::new(&mut state.decay_rates.z, 0.0..=2.0));
                    ui.end_row();

                    // Emission
                    ui.label(egui::RichText::new("Glow").color(egui::Color32::GOLD));
                    ui.add(egui::Slider::new(&mut state.diffusion_rates.w, 0.0..=10.0));
                    ui.add(egui::Slider::new(&mut state.decay_rates.w, 0.0..=5.0));
                    ui.end_row();
                });
        });
    }
}
