use bevy::prelude::*;
use bevy_flycam::FlyCam;
// use iyes_loopless::prelude::*;

mod bevy_bridge;
pub use bevy_bridge::*;

mod fluids;
pub use fluids::*;

mod memory;
pub use memory::*;


pub const WIDTH: f32 = 1280f32;
pub const HEIGHT: f32 = 720f32;

pub fn setup(
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

pub fn additional_camera_setup(
    mut camera: Query<&mut Transform, With<FlyCam>>,
) {
    let mut camera = camera.single_mut();

    camera.translation = Vec3::new(0.0, 12.5, 5.0);
    camera.look_at(Vec3::new(0.0, 11.0, 2.5), Vec3::Y);
}


pub fn additional_camera_system(
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

