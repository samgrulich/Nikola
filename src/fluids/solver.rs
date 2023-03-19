use crate::{TableMap, SmoothedParticle};

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

