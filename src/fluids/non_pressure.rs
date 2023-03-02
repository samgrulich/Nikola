use std::ops::DerefMut;

use bevy::prelude::Vec3;
use crate::{
    fluids::{
        particle::SmoothedParticle, 
        GRAVITATIONAL_ACCELERATION,
    },
    memory::Rcc,
};

pub fn advect_particle(mut particle: Rcc<SmoothedParticle>, delta_time: f32) {
    let particle = particle.deref_mut();

    particle.velocity_predict = particle.velocity + Vec3::new(0.0, GRAVITATIONAL_ACCELERATION * delta_time, 0.0)
}

pub fn advect(particles: &Vec<Rcc<SmoothedParticle>>, delta_time: f32) {
    for particle in particles {
        advect_particle(particle.clone(), delta_time);
    }
}
