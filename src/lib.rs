mod fluids;
use fluid_renderer::{Instance, State, create_dense_rect};
pub use fluids::*;


pub fn calculate_boundaries_rect_count(dimensions: (u32, u32)) -> u32 {
    (2.0 * (
            dimensions.0 as f32 / fluids::PARTICLE_RADIUS 
          + dimensions.1 as f32 / fluids::PARTICLE_RADIUS)
    ).ceil() as u32
}

fn instances_to_particles<T: Particle>(instances: &Vec<Instance>) -> Vec<T> {
    instances.iter().enumerate().map(|(i, instance)| {
        Particle::new(i as u32, instance.position.into())
    }).collect()
}

pub fn setup_fluid_sim(instances: &Vec<Instance>) -> Fluid {
    let particles: Vec<FluidParticle> = instances_to_particles(instances);

    let fluid = Fluid::from_particles(particles, None, None, None);

    fluid
}

pub fn setup_boundary() -> (Vec<Instance>, TableMap<BoundaryParticle>) {
    let dimensions = (4, 3);
    let offset = (
        -2.0,
        -2.0 * 3.0 / 4.0,
        0.0
    );

    let boundary_instances = create_dense_rect(dimensions, offset, Some(fluids::PARTICLE_RADIUS), None);
    let boundary_particles = instances_to_particles(&boundary_instances);
    let boundary_table = TableMap::from_particles(boundary_particles);

    (boundary_instances, boundary_table)
}

pub fn step_fluid_sim(state: &mut State, fluid: &mut Fluid, boundary_table: &TableMap<BoundaryParticle>) {
    dfsph(fluid, boundary_table);

    fluid.particles()
        .zip(state.instances.iter_mut())
        .for_each(
            |(particle, instance)| {
                instance.position = particle.position.into();          
            }
        ); 
}

