use std::{rc::Rc, borrow::BorrowMut};
use bevy::prelude::Vec3;
use crate::fluids::{particle::SmoothedParticle, GRAVITATIONAL_ACCELERATION};

pub fn advect_particle(particle: &Rc<SmoothedParticle>, delta_time: f32) {
    let &mut particle = particle.borrow_mut();

    particle.velocity_predict = particle.velocity + Vec3::new(0.0, GRAVITATIONAL_ACCELERATION, 0.0)
}

pub fn advect(particles: &Vec<Rc<SmoothedParticle>>, delta_time: f32) {
    for particle in particles {
        advect_particle(particle, delta_time);
    }
}
