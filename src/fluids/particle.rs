use glam::Vec3A;

use crate::{fluids, Neighborhood};

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct SmoothedParticle {
    pub id: u32,
    pub position: Vec3A,

    pub velocity: Vec3A,
    pub velocity_future: Vec3A,

    pub density: f32,
    pub density_future: f32,

    pub dsph_factor: f32,
    pub pressure: f32,
}

impl SmoothedParticle {
    pub const RADIUS: f32 = fluids::PARTICLE_RADIUS;
    pub const REST_DENSITY: f32 = fluids::REST_DENSITY;

    const RADIUS_POW_3: f32 = Self::RADIUS * Self::RADIUS * Self::RADIUS;
    pub const MASS: f32 = 4.0 / 3.0 * std::f32::consts::PI * Self::RADIUS_POW_3 * Self::REST_DENSITY;
}

impl SmoothedParticle {
    pub fn new(
        id: u32,
        position: Vec3A, 
    ) -> Self {
        SmoothedParticle {
            id,
            position,
            ..Default::default()
        }
    }
}

impl Default for SmoothedParticle {
    fn default() -> Self {
        SmoothedParticle {
            id: 0,
            position: Vec3A::ZERO,
            velocity: Vec3A::ZERO,
            velocity_future: Vec3A::ZERO,
            density: Self::REST_DENSITY,
            density_future: Self::REST_DENSITY,
            dsph_factor: 0.0, 
            pressure: 0.0,
        }
    }
}

impl SmoothedParticle {
   pub fn interpolate_div_vf(&self, neighborhood: &Neighborhood) -> f32 {
        let mut qtity: f32 = 0.0;
        let mass_sum = neighborhood.get_len() * SmoothedParticle::MASS;
        
        for (i, neighbor) in neighborhood.neighbors.iter().enumerate() {
            unsafe {
                qtity += (self.velocity_future - (**neighbor).velocity_future).dot(neighborhood.gradients[i]); 
            }
        }

        -1.0/self.density * qtity * mass_sum
    }
}

impl SmoothedParticle {
    pub fn compute_density_derivate(&self, neighborhood: &Neighborhood) -> f32 {
        let mut density_div = 0.0;
       
        for (i, neighbor) in neighborhood.neighbors.iter().enumerate() {
            unsafe {
                density_div += (self.velocity - (**neighbor).velocity).dot(*neighborhood.gradients.get(i).unwrap());
            }
        }

        density_div * neighborhood.get_len() * Self::MASS
    }

    pub fn compute_dfsph_factor(&self, neighborhood: &Neighborhood) -> f32 {
        let mut outter_sum = 0.0; 

        if neighborhood.get_len() == 0.0 {
            return 0.0;
        }

        let mass_sum = neighborhood.get_len() * Self::MASS;
        let gradients_sum: Vec3A = neighborhood.gradients.iter().sum();

        for gradient in neighborhood.gradients.iter() {
            outter_sum += gradient.length().powi(2);
        }

        dbg!(gradients_sum, mass_sum);

        self.density.powi(2) / ((mass_sum * gradients_sum).length().powi(2) + (mass_sum.powi(2) * outter_sum))
    }

    pub fn compute_density_future(&self, neighborhood: &Neighborhood, delta_time: f32) -> f32 {
        let mut sum = 0.0;
        let mass_sum = neighborhood.get_len() * Self::MASS;

        for (i, neighbor) in neighborhood.neighbors.iter().enumerate() {
            unsafe {
                sum += (self.velocity_future - (**neighbor).velocity_future).dot(neighborhood.gradients[i]);
            }
        }

        self.density + delta_time * sum * mass_sum
    }
}

