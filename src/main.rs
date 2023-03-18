use bevy::prelude::*;
use bevy_flycam::PlayerPlugin;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;

use nikola::*;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor { 
                width: WIDTH, 
                height: HEIGHT, 
                title: "Nikola - bevy".to_string(),
                resizable: false,
                ..default()
            },
            ..default()
        }))
        // .add_plugin(WorldInspectorPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(ParticlePlugin)
        .add_plugin(FluidSimulationPlugin)
        .add_startup_system(setup)
        .add_startup_system_to_stage(StartupStage::PostStartup, additional_camera_setup)
        .add_system(additional_camera_system)
        .run();
}
