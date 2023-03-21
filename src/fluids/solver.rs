use std::slice::{Iter, IterMut};

use crate::{TableMap, FluidParticle, Particle, Neighborhood};

pub struct Fluid {
    table: TableMap<FluidParticle>,

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
        self.delta_time = self.cfl_parameter * FluidParticle::RADIUS / self.get_max_velocity().max(1.0);
    }
}

impl Fluid {
    pub fn from_particles(
        particles: Vec<FluidParticle>, 
        cfl_parameter: Option<f32>, 
        density_threshold: Option<f32>,
        divergence_threshold: Option<f32>,
    ) -> Self {
        let mut fluid = Self { 
            table: TableMap::from_particles(particles),
            cfl_parameter: cfl_parameter.unwrap_or(0.4),
            density_threshold: density_threshold.unwrap_or(0.125),
            divergence_threshold: divergence_threshold.unwrap_or(0.3),
            ..Default::default()
        };

        fluid.table.update_particle_factors();

        fluid
    }

    pub fn update(&mut self) {
        self.table.update();
        self.table.update_particle_factors();
    }

    pub fn particles(&self) -> Iter<FluidParticle>{
        self.table.particles.iter()
    }

    pub fn particles_mut(&mut self) -> IterMut<FluidParticle> {
        self.table.particles.iter_mut()
    }

    pub fn get_neighborhood_2d(&self, id: u32) -> Neighborhood<FluidParticle> {
        self.table.get_neighborhood_2d(id)
    }

    pub fn len(&self) -> usize {
        self.table.particles.len()
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

