use bevy::{
    prelude::*,
    utils::Duration
};
use iyes_loopless::prelude::AppLooplessFixedTimestepExt;

use crate::{
    ParticleType, Fluid, FluidConfig, REST_DENSITY, PARTICLE_RADIUS
};


pub const FLUID_TIMESTEP: Duration = Duration::from_millis(300);

pub fn move_particles(
    mut particles: Query<(), With<ParticleType>>
) {
    
}

fn setup(
    mut particles: Query<(Entity, &Transform), With<ParticleType>>
) {
    let fluid_config = FluidConfig::default();
    let mut raw_particles = Vec::new();

    for particle in particles.iter() {
        raw_particles.push((particle.0.index(), particle.1.translation));
    }

    let delta_time = FLUID_TIMESTEP.as_secs_f32();
    let fluid = Fluid::from_particles(fluid_config, raw_particles, REST_DENSITY, PARTICLE_RADIUS, delta_time);
    // todo: add fluid instance into the scene, fluid update, particle update
}

pub struct FluidSimulationPlugin;

impl Plugin for FluidSimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system_to_stage(StartupStage::Startup, setup)
            .add_fixed_timestep(
                FLUID_TIMESTEP, 
                "fluid_update"
            )
            .add_fixed_timestep_system("fluid_update", 0, move_particles);
    }
}
