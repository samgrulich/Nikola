use std::{rc::Rc, borrow::BorrowMut};
use crate::fluids::{particle::SmoothedParticle, neighborhoods::Neighborhoods};

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
                // compute p_i for particle
            }

            for particle in &mut self.particles {
                // update future velocity
            }

            iteration += 1;
        }
    }

    fn correct_divergence(&mut self, threshold: f32, delta_time: f32) {
        let mut iteration = 0;
        // todo: compute average_density_over_time

        while (iteration < 1) || (average_density_over_time > threshold) {
            for particle in &mut self.particles {
                // compute density_over_time
            }

            for particle in &mut self.particles {
                // compute p_v_i 
            }
            
            for particle in &mut self.particles {
                // update future velocities
            }
            
            // update average_density_over_time
            iteration += 1;
        }
        // for particles i 
    }

    pub fn dfsph(&self, delta_time: f32) {
        for particle in &mut self.particles {
            let neighbors = self.neighborhoods.get_neighbors(particle.position);

            if let Some(others) = neighbors {
                particle.borrow_mut() = particle.compute_dsph_factor(&others);
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
