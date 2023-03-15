use std::{borrow::BorrowMut, ops::DerefMut, time::Duration};
use bevy::prelude::Vec3;
use crate::{
    fluids::{
        particle::SmoothedParticle, 
        neighborhoods::Neighborhoods, 
        non_pressure::advect,
        kernel,
    }, 
    memory::Rcc,
    smoothing_kernel_grad,
};

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

pub struct FluidConfig {
    cfl_parameter: f32, // ~0.4
    density_threshold: f32, // ~0.125-0.3
    divergence_threshold: f32, // ?? probably ~0.125-0.3
}

impl Default for FluidConfig {
    fn default() -> Self {
        Self { 
            cfl_parameter: 0.4, 
            density_threshold: 0.150, 
            divergence_threshold: 0.22
        } 
    }
}

pub struct Fluid {
    pub particles: Vec<Rcc<SmoothedParticle>>,
    neighborhoods: Neighborhoods,
    particle_radius: f32,

    cfl_parameter: f32, // ~0.4
    density_threshold: f32, // ~0.125-0.3
    divergence_threshold: f32, // ?? probably ~0.125-0.3

    rest_density: f32,
    average_density: f32,

    max_velocity: f32, // todo: check if being set properly
    delta_time: f32,
}

impl Fluid {
    pub fn from_particles(config: FluidConfig, raw_particles: Vec<(u32, Vec3)>, rest_density: f32, particle_radius: f32, delta_time: f32) -> Self {
        let mut particles: Vec<Rcc<SmoothedParticle>> = Vec::new();
        let particle_mass: f32 = 4.0/3.0 * std::f32::consts::PI * particle_radius.powi(3) * rest_density;

        for raw_particle in raw_particles {
            let particle = SmoothedParticle::new(raw_particle.0, raw_particle.1, rest_density, particle_mass);
            particles.push(Rcc::new(particle));
        }

        let neighborhoods = Neighborhoods::from(&mut particles);

        Fluid { 
            particles, 
            neighborhoods,
            particle_radius, 
            cfl_parameter: config.cfl_parameter, 
            density_threshold: config.density_threshold, 
            divergence_threshold: config.divergence_threshold, 
            rest_density, 
            average_density: rest_density, 
            max_velocity: 0.0, 
            delta_time,
        }
    }
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
    fn correct_density(&mut self, threshold: f32) {
        let mut iteration = 0;

        // todo: change average density to include density predict instead i guess
        while (iteration < 2) || (self.average_density - self.rest_density > threshold) {
            for particle in &mut self.particles {
                let j_particles = self.neighborhoods.get_neighbors(particle.position);

                if let Some(others) = j_particles {
                    particle.compute_density_predict_inplace(&others, self.delta_time);
                }
            }

            for particle in &mut self.particles {
                particle.pressure = 1.0 / self.delta_time.powi(2) * (particle.density_predict - self.rest_density) * particle.dsph_factor;
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

                particle.velocity_predict = particle.velocity_predict - self.delta_time * sum;
            }

            iteration += 1;
        }
    }

    fn correct_divergence(&mut self, threshold: f32) {
        let mut iteration = 0;
        // todo: compute average_density_over_time
        let mut average_density_over_time = 0.0;
        let mut density_over_time_sum = 0.0;

        while (iteration < 1) || (average_density_over_time > threshold) {
            for particle in &mut self.particles {
                let neighbors = self.neighborhoods.get_neighbors(particle.position).unwrap_or_default();

                let density_over_time_i = -particle.density * particle.interpolate_div(&neighbors, "velocity_predict"); 
                density_over_time_sum += density_over_time_i;
            }

            for particle in &mut self.particles {
                let mut density_over_time = 0.0;
                let particle = particle.deref_mut();
                let neighbors = self.neighborhoods.get_neighbors(particle.position).unwrap_or_default();

                for neighbor in neighbors {
                    density_over_time += neighbor.mass * (particle.velocity - neighbor.velocity).dot(kernel::smoothing_kernel_grad(particle.position, neighbor.position, None));
                }

                particle.pressure_value = 1.0 / self.delta_time * 0.0 * particle.dsph_factor;
            }
            
            for particle in &mut self.particles {
                let mut sum = Vec3::ZERO;
                let neighbors = self.neighborhoods.get_neighbors(particle.position).unwrap_or_default();

                for neighbor in neighbors {
                    sum += neighbor.mass * (particle.pressure_value / particle.density.powi(2) + neighbor.pressure_value / neighbor.density.powi(2)) * smoothing_kernel_grad(particle.position, neighbor.position, None)
                }

                particle.velocity_predict = particle.velocity_predict - self.delta_time * sum;
            }
            
            average_density_over_time = density_over_time_sum / self.particles.len() as f32;
            iteration += 1;
        }
    }

    pub fn apply_cfl(&mut self) {
        self.delta_time = self.cfl_parameter * self.particle_radius / self.get_max_velocity().max(1.0);
    }

    pub fn dfsph(&mut self) {
        let time_start = std::time::Instant::now();
        dbg!("starting dfsph");
        for particle in &mut self.particles {
            let particle: &mut SmoothedParticle = particle.borrow_mut();
            let neighbors = self.neighborhoods.get_neighbors(particle.position);

            if let Some(others) = neighbors {
                particle.dsph_factor = particle.compute_dsph_factor(&others);
            }
        }
        dbg!("dfsph factors updated", time_start.elapsed().as_millis());
        // let pressure_value = 1.0 / delta_time * self.compute_density_derivate(others) * self.density.powi(2) / k_factor;

        // compute nonp acceleration

        // adapt delta time
        self.apply_cfl();
        dbg!("cfl applied", time_start.elapsed().as_millis());
        
        // for particles i predict velocity v_predict = v_i + time_delta * a_i_nonp
        advect(&mut self.particles, self.delta_time);
        dbg!(self.delta_time);
        dbg!("advected", time_start.elapsed().as_millis());
        // correct density error using constant density solver
        self.correct_density(self.density_threshold);
        dbg!("density corrected", time_start.elapsed().as_millis());

        // for particles i update position
        for particle in &mut self.particles {
            let particle: &mut SmoothedParticle = particle.borrow_mut();

            particle.position += particle.velocity_predict * self.delta_time;
        }
        dbg!("updated positions", time_start.elapsed().as_millis());

        // update neighborhoods (refresh hash table)
        self.neighborhoods = Neighborhoods::from(&mut self.particles);

        // for particles do 
        //  update density 
        //  update k_factor

        // correct divergence using divergence solver 
        self.correct_divergence(self.divergence_threshold);
        dbg!("div corrected", time_start.elapsed().as_millis());
        // update velocity
        for particle in &mut self.particles {
            particle.velocity = particle.velocity_predict;
        }
    }
}
