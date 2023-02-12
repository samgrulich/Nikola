use bevy::prelude::*;
use crate::{Velocity, ParticleType};


pub fn move_particles(
    mut particles: Query<&mut Velocity, With<ParticleType>>
) {

}

pub struct FluidSimulationPlugin;

impl Plugin for FluidSimulationPlugin {
    fn build(&self, app: &mut App) {
       app; 
    }
}
