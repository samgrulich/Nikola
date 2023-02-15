use bevy::prelude::*;
// use iyes_loopless::prelude::*;

use bevy_flycam::{PlayerPlugin, FlyCam};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod particles;
pub use particles::*;

mod simulation;
pub use simulation::*;

mod fluids;
pub use fluids::*;


pub const WIDTH: f32 = 1280f32;
pub const HEIGHT: f32 = 720f32;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {size: 444.0})),
        material: materials.add(Color::rgb(0.2, 0.3, 0.4).into()),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            ..default()
        },
        transform: Transform::from_xyz(0., 0., 0.)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 210f32.to_radians(), 20f32.to_radians(), 0.)), 
        ..default()
    });
}

fn additional_camera_setup(
    mut camera: Query<&mut Transform, With<FlyCam>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>
) {
    if !keyboard.pressed(KeyCode::LControl) {
        return;
    }

    let mut camera = camera.single_mut();
    let speed = 10f32;

    camera.translation -= Vec3::Y * time.delta_seconds() * speed;
}


pub fn run() { 
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
        .add_startup_system(setup)
        .add_system(additional_camera_setup)
        .run();
}
