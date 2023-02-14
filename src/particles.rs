use bevy::prelude::*;

pub const DIMENSIONS: (i32, i32, i32) = (20, 20, 20);
pub const PARTICLE_RADIUS: f32 = 0.1;
pub const PARTICLE_OFFSET: f32 = 0.1;
pub const FLUID_OFFSET: f32 = 10.0;

pub const GRAVITY_ACCELERATION: f32 = 9.81;


#[derive(Resource)]
struct Sphere {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct Particle;

#[derive(Component, Default)]
pub struct Movement {
    velocity: Vec3,
}

#[derive(Bundle)]
pub struct ParticleBundle {
    _p: Particle,
    movement: Movement,

    #[bundle]
    object: PbrBundle,
}

impl ParticleBundle {
    pub fn new(object: PbrBundle) -> Self {
        ParticleBundle { 
            _p: Particle, 
            movement: Movement::default(),
            object 
        }
    }
}



fn spawn(
    dimensions: (i32, i32, i32),
    commands: &mut Commands,
    sphere: &Sphere
)
{
    let offset = PARTICLE_RADIUS + PARTICLE_OFFSET;

    for z in 0..dimensions.2 {
        for y in 0..dimensions.1 {
            for x in 0..dimensions.0 {
                let position = Vec3::new(x as f32 * offset, y as f32 * offset + FLUID_OFFSET, z as f32 * offset);

                commands.spawn(ParticleBundle::new(PbrBundle {
                        mesh: sphere.mesh.clone(),
                        material: sphere.material.clone(),
                        transform: Transform::from_xyz(position.x, position.y, position.z),
                        ..default()
                    })
                );
            }
        }
    }
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sphere_mesh = meshes.add(Mesh::from(shape::Icosphere { radius: 0.1f32, subdivisions: 4 }));
    // let sphere_mesh = meshes.add(Mesh::from(shape::Cube { size: 1f32 }));
    let sphere_material = materials.add(Color::rgb(0.8, 0.4, 0.1).into());

    commands.insert_resource( Sphere {
        mesh: sphere_mesh,
        material: sphere_material,
    });
}

fn spawner(
    mut commands: Commands,
    sphere: Res<Sphere>,
) {
    spawn(DIMENSIONS, &mut commands, &sphere);
}

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system_to_stage(StartupStage::PreStartup, setup)
            .add_startup_system(spawner);
    }
}
