use std::slice::{Iter, IterMut};

use glam::Vec3A;

use crate::{TableMap, SmoothedParticle, GRAVITATIONAL_ACCELERATION, Neighborhood, state_of_equation};

pub struct Fluid {
    pub table: TableMap,

    pub cfl_parameter: f32, // ~0.4
    pub density_threshold: f32, // ~0.125-0.3
    pub divergence_threshold: f32, // ?? probably ~0.125-0.3

    pub delta_time: f32,
}

impl Fluid {
    pub fn get_average_density(&self) -> f32 {
        let mut density_sum = 0.0;

        self.table.particles.iter().for_each(|particle| density_sum += particle.density);

        density_sum / self.table.particles.len() as f32
    }

    pub fn get_max_velocity(&self) -> f32 {
        let mut max_velocity = 0.0;

        self.table.particles.iter().for_each(|particle| if particle.velocity.length() > max_velocity { max_velocity = particle.velocity.length() });

        max_velocity
    }

    pub fn apply_cfl(&mut self) {
        self.delta_time = self.cfl_parameter * SmoothedParticle::RADIUS / self.get_max_velocity().max(1.0);
    }
}

impl Fluid {
    pub fn particles(&self) -> Iter<SmoothedParticle> {
        self.table.particles.iter()
    }

    pub fn particles_mut(&mut self) -> IterMut<SmoothedParticle> {
        self.table.particles.iter_mut()
    }
}

impl Fluid  {
    pub fn step(&mut self) {
        // adapt delta time
        self.apply_cfl();
        let delta_time = self.delta_time;

        // for particles i predict velocity v_predict = v_i + time_delta * a_i_nonp
        for particle in self.table.particles.iter_mut() {
            particle.velocity_future = particle.velocity + delta_time * GRAVITATIONAL_ACCELERATION;
        }

        let neighborhoods: Vec<Neighborhood> = self.particles().map(|particle| {
            self.table.get_neighborhood_2d(particle.id) 
        }).collect();

        for (particle, neighborhood) in self.particles_mut().zip(&neighborhoods) {
            particle.density = particle.compute_density(&neighborhood);
        }

        // compute pressure at particles
        for particle in self.particles_mut() {
            particle.pressure = state_of_equation(particle.density, SmoothedParticle::REST_DENSITY, 2.0);
        }
        
        for (particle, neighborhood) in self.particles_mut().zip(&neighborhoods) {
            // dbg!(&particle, &neighborhood.gradients);
            if particle.density <= 1.0 {
                particle.velocity = particle.velocity_future;
                particle.position += particle.velocity * delta_time;
                continue;
            }

            let sum: f32 = neighborhood.neighbors.iter().map(|neighbor| {
                let neighbor = unsafe {&**neighbor};

                if neighbor.density <= 1.0 {
                    return 0.0;
                }

                neighbor.pressure / neighbor.density.powi(2)
            }).sum::<f32>() + particle.pressure / particle.density.powi(2) * neighborhood.get_len();
            let gradients: Vec3A = neighborhood.gradients.iter().sum();

            let pressure = particle.density * SmoothedParticle::REST_DENSITY * sum * gradients;
            let pressure_force = -1.0/particle.density*pressure;

            particle.velocity_future = particle.velocity_future + delta_time * pressure_force * SmoothedParticle::MASS;
            particle.velocity = particle.velocity_future;
            particle.position += particle.velocity * delta_time;
        }

        self.table.update();
    }
}

impl Default for Fluid {
    fn default() -> Self {
        Fluid { 
            table: TableMap::new(), 
            cfl_parameter: 0.4, 
            density_threshold: 0.125, 
            divergence_threshold: 0.3, 
            delta_time: 1.0 / 20.0
        } 
    }
}

