use std::{rc::Rc, borrow::BorrowMut};
use bevy::prelude::Vec3;
use crate::{fluids::{particle::SmoothedParticle, neighborhoods::Neighborhoods, kernel}, smoothing_kernel_grad};

// const SPEED_OF_SOUND: f32 = 0.3;
// const SPEED_OF_SOUND: f32 = 1480.0; // m/s in water
const SPEED_OF_SOUND_2: f32 = 2_190_400.0;

pub fn state_of_equation(density_i: f32, rest_density: f32, constant: f32) -> f32 {
    let pressure = constant * (density_i / rest_density - 1.0);

    pressure
}

pub fn state_of_equation_sound(density: f32) -> f32 {
    SPEED_OF_SOUND_2 * density
}

pub struct Fluid {
    particles: Vec<Rc<SmoothedParticle>>,
    neighborhoods: Neighborhoods,

    rest_density: f32,
    average_density: f32,

    max_velocity: f32,
    time_delta: f32,
}

impl Fluid {
    pub fn get_average_density(&self) -> f32 {
        let mut density_sum = 0.0;

        self.particles.iter().for_each(|particle| density_sum += particle.density);

        density_sum / self.particles.len() as f32
    }

    pub fn get_max_velocity(&self) -> f32 {
        let mut max_velocity = 0.0;

        self.particles.iter().for_each(|particle| if particle.velocity.length() > max_velocity { max_velocity = particle.velocity.length() });

        max_velocity
    }
}

impl Fluid {
    fn correct_density(&mut self, threshold: f32, delta_time: f32) {
        let mut iteration = 0;

        // todo: change average density to include density predict instead i guess
        while (iteration < 2) || (self.average_density - self.rest_density > threshold) {
            for particle in &mut self.particles {
                let j_particles = self.neighborhoods.get_neighbors(particle.position);

                if let Some(others) = j_particles {
                    particle.borrow_mut().compute_density_predict_inplace(&others, delta_time);
                }
            }

            for particle in &mut self.particles {
                particle.pressure = 1.0 / delta_time.powi(2) * (particle.density_predict - self.rest_density) * particle.dsph_factor;
            }

            for particle in &mut self.particles {
                let mut sum = Vec3::ZERO;
                let neighbors = self.neighborhoods.get_neighbors(particle.position).unwrap_or_default();

                for neighbor in neighbors {
                    sum += neighbor.mass
                            * (particle.pressure / particle.density.powi(2)   // these may be predicts
                                + neighbor.pressure / neighbor.density.powi(2) // here too
                                )
                            * kernel::smoothing_kernel_grad(particle.position, neighbor.position, None);
                }

                particle.velocity_predict = particle.velocity_predict - delta_time * sum;
            }

            iteration += 1;
        }
    }

    fn correct_divergence(&mut self, threshold: f32, delta_time: f32) {
        let mut iteration = 0;
        // todo: compute average_density_over_time
        let mut average_density_over_time = 0.0;
        let mut density_over_time_sum = 0.0;

        while (iteration < 1) || (average_density_over_time > threshold) {
            for particle in &mut self.particles {
                let mut neighbors = self.neighborhoods.get_neighbors(particle.position).unwrap_or_default();

                let density_over_time_i = -particle.density * particle.interpolate_div(&neighbors, "velocity_predict"); 
                density_over_time_sum += density_over_time_i;
            }

            for particle in &mut self.particles {
                let mut density_over_time = 0.0;
                let neighbors = self.neighborhoods.get_neighbors(particle.position).unwrap_or_default();

                for neighbor in neighbors {
                    density_over_time += neighbor.mass * (particle.velocity - neighbor.velocity).dot(kernel::smoothing_kernel_grad(particle.position, neighbor.position, None));
                }

                particle.pressure_value = 1.0 / delta_time * 0.0 * particle.dsph_factor;
            }
            
            for particle in &mut self.particles {
                let mut sum = Vec3::ZERO;
                let neighbors = self.neighborhoods.get_neighbors(particle.position).unwrap_or_default();

                for neighbor in neighbors {
                    sum += neighbor.mass * (particle.pressure_value / particle.density.powi(2) + neighbor.pressure_value / neighbor.density.powi(2)) * smoothing_kernel_grad(particle.position, neighbor.position, None)
                }

                particle.velocity_predict = particle.velocity_predict - delta_time * sum;
            }
            
            average_density_over_time = density_over_time_sum / self.particles.len() as f32;
            iteration += 1;
        }
    }

    pub fn dfsph(&self, delta_time: f32) {
        for particle in &mut self.particles {
            let particle = particle.borrow_mut();
            let neighbors = self.neighborhoods.get_neighbors(particle.position);

            if let Some(others) = neighbors {
                particle.dsph_factor = particle.compute_dsph_factor(&others);
            }
        }
        // let pressure_value = 1.0 / delta_time * self.compute_density_derivate(others) * self.density.powi(2) / k_factor;

        // compute nonp acceleration
        // adapt delta time
        
        // for particles i predict velocity v_predict = v_i + time_delta * a_i_nonp
        // correct density error using constant density solver

        // for particles i update position
        // update neighborhoods (refresh hash table)

        // for particles do 
        //  update density 
        //  update k_factor
        // correct divergence using divergence solver 
    }
}
