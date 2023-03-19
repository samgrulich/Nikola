use glam::Vec3A;

use crate::{fluids, Neighborhood};

#[repr(C)]
#[derive(Debug)]
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
            velocity: Vec3A::ZERO,
            velocity_future: Vec3A::ZERO,
            density: Self::REST_DENSITY,
            density_future: Self::REST_DENSITY,
            dsph_factor: 0.0, // todo: check if the intial values need to be changed
            pressure: 0.0,
        }
    }
}

impl SmoothedParticle {
    // pub fn interpolate(&self, others: &Vec<Rcc<SmoothedParticle>>, field: &str) -> f32 {
    //     let mut qtity_i: f32 = 0.0;
    //
    //     for other in others.iter() {
    //         if let Some(qtity_j) = other.get_f32(field) {
    //             qtity_i += other.MASS / other.density * qtity_j * kernel::smoothing_kernel(self.position, other.position, None);
    //         }
    //     }
    //
    //     qtity_i
    // }
    //
    // pub fn interpolate_grad(&self, others: &Vec<Rcc<SmoothedParticle>>, field: &str) -> Vec3 {
    //     let mut qtity = Vec3::ZERO;
    //     let qtity_i = self.get_vec3(field).unwrap();
    //     
    //     for other in others.iter() {
    //         if let Some(qtity_j) = other.get_vec3(field) {
    //             qtity += other.MASS * ( qtity_i / self.density.powi(2) + qtity_j / other.density.powi(2)) * kernel::smoothing_kernel_grad(self.position, other.position, None);
    //         }
    //     }
    //
    //     self.density * qtity
    // }
    // 
    // pub fn interpolate_grad_f32(&self, others: &Vec<Rcc<SmoothedParticle>>, field: &str) -> Vec3 {
    //     let mut qtity = Vec3::ZERO;
    //     let qtity_i = self.get_f32(field).unwrap();
    //     
    //     for other in others.iter() {
    //         if let Some(qtity_j) = other.get_f32(field) {
    //             qtity += other.MASS * ( qtity_i / self.density.powi(2) + qtity_j / other.density.powi(2)) * kernel::smoothing_kernel_grad(self.position, other.position, None);
    //         }
    //     }
    //
    //     self.density * qtity
    // }
    //
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
    //
    // pub fn interpolate_curl(&self, others: &Vec<Rcc<SmoothedParticle>>, field: &str) -> Vec3 {
    //     let mut qtity: Vec3 = Vec3::ZERO;
    //     let qtity_i = self.get_vec3(field).unwrap();
    //     
    //     for other in others.iter() {
    //         if let Some(qtity_j) = other.get_vec3(field) {
    //             qtity += other.mass * (qtity_i - qtity_j).cross(kernel::smoothing_kernel_grad(self.position, other.position, None)); 
    //         }
    //     }
    //
    //     1.0/self.density * qtity
    // }
    // 
    // // todo: implement laplacian for scalars
    // pub fn interpolate_lap(&self, others: &Vec<Rcc<SmoothedParticle>>, field: &str) -> Vec3 {
    //     let mut qtity = Vec3::ZERO;
    //     let qtity_i = self.get_vec3(field).unwrap();
    //     
    //     for other in others.iter() {
    //         let x_ij = self.position - other.position;
    //
    //         if let Some(qtity_j) = self.get_vec3(field) {
    //             qtity += other.mass / other.density * (qtity_i - qtity_j) * (x_ij.dot(kernel::smoothing_kernel_grad(self.position, other.position, None)) / (x_ij.dot(x_ij) + 0.01 * fluids::SMOOTHING_LENGHT.powi(2))); 
    //         }
    //     }
    //
    //     2.0 * qtity 
    // }
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

    pub fn compute_dsph_factor(&self, neighborhood: &Neighborhood) -> f32 {
        let mut outter_sum = 0.0; 

        let mass_sum = neighborhood.get_len() * Self::MASS;
        let gradients_sum: Vec3A = neighborhood.gradients.iter().sum();

        for gradient in neighborhood.gradients.iter() {
            outter_sum += gradient.length().powi(2);
        }

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

