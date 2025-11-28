use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_panorbit_camera::PanOrbitCameraPlugin;

mod state;
mod ui;
mod voronoi;
mod chemistry;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Voronoi Vivarium".into(),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin::default(),
            PanOrbitCameraPlugin,
            MeshPickingPlugin,
        ))
        .init_resource::<state::SimState>()
        .init_resource::<chemistry::CellMap>() // Init the Map
        .add_systems(Startup, ui::setup_scene)
        .add_systems(EguiPrimaryContextPass, ui::ui_system)
        .add_systems(
            Update, 
            (
                voronoi::spawn_mesh_system,
                chemistry::reaction_diffusion_system,
                chemistry::state_update_system,
            ).chain()
        )
        .run();
}