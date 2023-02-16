use bevy::prelude::*;

use crate::fluids::kernel;
use crate::fluids;

pub struct SmoothedParticle {
    pub id: i32,
    pub position: Vec3,
    pub velocity: Vec3,
    velocity_predict: Vec3,

    pub density: f32,
    density_predict: f32,

    mass: f32,
}

impl SmoothedParticle {
    pub fn new(
        id: i32,
        position: Vec3, 
        density: f32,
        mass: f32,
    ) -> Self {
        SmoothedParticle {
            id,
            position,
            velocity: Vec3::ZERO,
            velocity_predict: Vec3::ZERO,
            density,
            density_predict: density,
            mass,
        }
    }
}

impl SmoothedParticle {
    pub fn interpolate(&self, others: &[SmoothedParticle], qtities_j: &[f32]) -> f32 {
        let mut qtity_i: f32 = 0.0;

        for (other, qtity_j) in others.iter().zip(qtities_j.iter()) {
            qtity_i += other.mass / other.density * qtity_j * kernel::smoothing_kernel(self.position, other.position, None);
        }

        qtity_i
    }

    pub fn interpolate_grad(&self, others: &[SmoothedParticle], qtity_i: f32, qtities_j: &[f32]) -> Vec3 {
        let mut qtity: Vec3 = Vec3::ZERO;
        
        for (other, qtity_j) in others.iter().zip(qtities_j.iter()) {
            qtity += other.mass * ( qtity_i / self.density.powi(2) + qtity_j / other.density.powi(2)) * kernel::smoothing_kernel_grad(self.position, other.position, None);
        }

        self.density * qtity
    }

    pub fn interpolate_div(&self, others: &[SmoothedParticle], qtity_i: &Vec3, qtities_j: &[Vec3]) -> f32 {
        let mut qtity: f32 = 0.0;
        
        for (other, qtity_j) in others.iter().zip(qtities_j.iter()) {
            qtity += other.mass * (*qtity_i - *qtity_j).dot(kernel::smoothing_kernel_grad(self.position, other.position, None)); 
        }

        -1.0/self.density * qtity
    }

    pub fn interpolate_curl(&self, others: &[SmoothedParticle], qtity_i: &Vec3, qtities_j: &[Vec3]) -> Vec3 {
        let mut qtity: Vec3 = Vec3::ZERO;
        
        for (other, qtity_j) in others.iter().zip(qtities_j.iter()) {
            qtity += other.mass * (*qtity_i - *qtity_j).cross(kernel::smoothing_kernel_grad(self.position, other.position, None)); 
        }

        1.0/self.density * qtity
    }
    
    pub fn interpolate_lap(&self, others: &[SmoothedParticle], qtity_i: f32, qtities_j: &[f32]) -> f32 {
        let mut qtity: f32 = 0.0;
        
        for (other, qtity_j) in others.iter().zip(qtities_j.iter()) {
            let x_ij = self.position - other.position;
            qtity += other.mass / other.density * (qtity_i - qtity_j) * (x_ij.dot(kernel::smoothing_kernel_grad(self.position, other.position, None)) / (x_ij.dot(x_ij) + 0.01 * fluids::SMOOTHING_LENGHT.powi(2))); 
        }

        2.0 * qtity 
    }
}

impl SmoothedParticle {
    pub fn compute_density_derivate(&self, others: &[SmoothedParticle]) -> f32 {
        let mut density_div = 0.0;
        
        for other in others.iter() {
            density_div += other.mass * (self.velocity - other.velocity).dot(kernel::smoothing_kernel_grad(self.position, other.position, None));
        }

        density_div
    }

    pub fn compute_dsph_factor(&self, others: &[SmoothedParticle]) -> f32 {
        let mut inner_sum = Vec3::ZERO;
        let mut outter_sum = 0.0; 

        for other in others {
            inner_sum += other.mass * kernel::smoothing_kernel_grad(self.position, other.position, None);
            outter_sum += (other.mass * kernel::smoothing_kernel_grad(self.position, other.position, None)).length().powi(2);
        }

        inner_sum.length().powi(2) + outter_sum
    }

    pub fn compute_density_predict(&self, others: &[SmoothedParticle], delta_time: f32) -> f32 {
        let mut sum = 0.0;

        for other in others {
            sum += other.mass * (self.velocity_predict - other.velocity_predict).dot(kernel::smoothing_kernel_grad(self.position, other.position, None));
        }

        self.density + delta_time * sum
    }

    pub fn compute_density_predict_inplace(&mut self, others: &[SmoothedParticle], delta_time: f32) {
        self.density = self.compute_density_predict(others, delta_time);
    }
}

