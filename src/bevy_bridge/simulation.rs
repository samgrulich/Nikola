use bevy::prelude::*;
use iyes_loopless::prelude::AppLooplessFixedTimestepExt;

use crate::{
    ParticleType, Fluid, FluidConfig, REST_DENSITY, PARTICLE_RADIUS, FLUID_TIMESTEP
};


#[derive(Component)]
pub struct FluidManager(Fluid);

pub fn move_particles(
    mut particles: Query<(Entity, &mut Transform), With<ParticleType>>,
    mut fluid_manager: Query<&mut FluidManager>
) {
    dbg!("-----------------------------------");
    let mut fluid_manager = fluid_manager.single_mut(); 
    fluid_manager.0.dfsph();

    dbg!("--------------step-----------------");
    for particle in particles.iter_mut() {
        let (id, mut transform) = particle;

        for smoothed_particle in &fluid_manager.0.particles {
            if smoothed_particle.id == id.index() {
                // dbg!(smoothed_particle);
                transform.translation = smoothed_particle.position;
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    particles: Query<(Entity, &Transform), With<ParticleType>>,
) {
    let fluid_config = FluidConfig::default();
    let mut raw_particles = Vec::new();

    for particle in particles.iter() {
        raw_particles.push((particle.0.index(), particle.1.translation));
    }

    // let delta_time = FLUID_TIMESTEP.as_secs_f32();
    let delta_time = 0.01;
    let fluid = Fluid::from_particles(fluid_config, raw_particles, REST_DENSITY, PARTICLE_RADIUS, delta_time);

    commands.spawn(FluidManager(fluid));
    dbg!("initialized");
    // todo: add fluid instance into the scene, fluid update, particle update
}

pub struct FluidSimulationPlugin;

impl Plugin for FluidSimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system_to_stage(StartupStage::PostStartup, setup)
            .add_fixed_timestep(
                FLUID_TIMESTEP, 
                "fluid_update"
            )
            .add_fixed_timestep_system("fluid_update", 0, move_particles);
            // .add_system_to_stage(CoreStage::PostUpdate, move_particles);
    }
}
