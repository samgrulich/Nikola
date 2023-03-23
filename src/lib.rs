mod fluids;
use fluid_renderer::{Instance, State, create_dense_rect, create_grid};
pub use fluids::*;
use glam::{Vec3A, vec3a};


pub fn calculate_boundaries_rect_count(dimensions: (u32, u32)) -> u32 {
    (2.0 * (
            dimensions.0 as f32 / fluids::PARTICLE_RADIUS 
          + dimensions.1 as f32 / fluids::PARTICLE_RADIUS)
    ).ceil() as u32
}

pub fn setup_fluid_sim() -> (Fluid, Vec<Instance>) {
    let instances = create_grid((10, 10), (2, 2), (-0.0, -0.0, 0.0));
    // let instances = vec![
    //     Instance::default(),
    //     Instance::default(),
    // ];

    // // SmoothedParticle::new(i as u32, instance.position.into())
    let particles: Vec<SmoothedParticle> = instances.iter().enumerate().map(|(i, instance)| {
        SmoothedParticle { 
            id: i as u32, 
            position: instance.position.into(), 
            velocity: -Vec3A::from(instance.position),
            ..Default::default()
        }
    }).collect();

    // let particles = vec![
    //     SmoothedParticle {
    //         id: 0,
    //         position: vec3a(0.5, 0.0, 0.0),
    //         velocity: Vec3A::NEG_X,
    //         ..Default::default()
    //     },
    //     SmoothedParticle {
    //         id: 1,
    //         position: vec3a(-0.5, 0.0, 0.0),
    //         velocity: Vec3A::X,
    //         ..Default::default()
    //     }
    // ];

    let fluid = Fluid {
        table: TableMap::from_particles(particles),
        cfl_parameter: 0.1,
        ..Default::default()
    };

    (fluid, instances)
}

pub fn setup_boundary() -> Vec<Instance> {
    let boundary_dimensions = (5, 5);
    let boundary_instances = create_dense_rect(boundary_dimensions, (-2.0, -2.0, 0.0), Some(fluids::PARTICLE_RADIUS), None);

    boundary_instances
}

pub fn step_fluid_sim(state: &mut State, fluid: &mut Fluid) {
    // dfsph(fluid);
    fluid.step();

    fluid.table.particles
        .iter()
        .zip(state.instances.iter_mut())
        .for_each(
            |(particle, instance)| {
                instance.position = particle.position.into();          
            }
        ); 
}

