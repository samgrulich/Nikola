mod fluids;
use fluid_renderer::{Instance, State, create_dense_rect};
pub use fluids::*;


pub fn calculate_boundaries_rect_count(dimensions: (u32, u32)) -> u32 {
    (2.0 * (
            dimensions.0 as f32 / fluids::PARTICLE_RADIUS 
          + dimensions.1 as f32 / fluids::PARTICLE_RADIUS)
    ).ceil() as u32
}

pub fn setup_fluid_sim(instances: &Vec<Instance>) -> Fluid {
    let particles: Vec<SmoothedParticle> = instances.iter().enumerate().map(|(i, instance)| {
        SmoothedParticle::new(i as u32, instance.position.into())
    }).collect();

    let fluid = Fluid {
        table: TableMap::from_particles(particles),
        cfl_parameter: 0.02,
        ..Default::default()
    };

    fluid
}

pub fn setup_boundary() -> Vec<Instance> {
    let dimensions = (4, 3);
    let offset = (
        -2.0,
        -2.0 * 3.0 / 4.0,
        0.0
    );

    let boundary_instances = create_dense_rect(dimensions, offset, Some(fluids::PARTICLE_RADIUS), None);

    boundary_instances
}

pub fn step_fluid_sim(state: &mut State, fluid: &mut Fluid) {
    dfsph(fluid);

    fluid.table.particles
        .iter()
        .zip(state.instances.iter_mut())
        .for_each(
            |(particle, instance)| {
                instance.position = particle.position.into();          
            }
        ); 
}

