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
        Transform::from_xyz(0.0, 20.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            focus: Vec3::ZERO,
            button_orbit: MouseButton::Right,
            button_pan: MouseButton::Middle,
            ..default()
        },
        // CRITICAL: This component enables mouse picking from this camera
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
                state.rebuild_requested = true;
            }

            if ui.button("Regenerate Graph").clicked() {
                state.rebuild_requested = true;
            }
        });
    }
}