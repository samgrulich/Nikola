mod fluids;
use fluid_renderer::{Instance, State};
pub use fluids::*;


pub fn setup_fluid_sim(instances: &Vec<Instance>) -> Fluid {
    let particles: Vec<SmoothedParticle> = instances.iter().enumerate().map(|(i, instance)| {
        SmoothedParticle::new(i as u32, instance.position.into())
    }).collect();

    let fluid = Fluid {
        table: TableMap::from_particles(particles),
        delta_time: 0.0001,
        ..Default::default()
    };
    
    dbg!("Initialization", &fluid.table.particles, "Initialization stop");

    fluid
}

pub fn step_fluid_sim(state: &mut State, fluid: &mut Fluid) {
    dfsph(fluid);

    fluid.table.particles
        .iter()
        .zip(state.instances.iter_mut())
        .for_each(
            |(particle, instance)| {
                if particle.id == 0 {
                    dbg!(particle.position);
                }

                instance.position = particle.position.into();          
            }
        ); 
}
