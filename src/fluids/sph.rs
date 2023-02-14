use bevy::prelude::*;

use crate::fluids::kernel;
use crate::fluids;

struct SmoothedParticle {
    position: Vec3,
    mass: f32,
    density: f32
}

impl SmoothedParticle {
    pub fn interpolate(&self, others: &[SmoothedParticle], qtities_j: &[f32]) -> f32 {
        let mut qtity_i: f32 = 0.0;

        for (other, qtity_j) in others.iter().zip(qtities_j.iter()) {
            qtity_i += other.mass / other.density * qtity_j * kernel::smoothing_kernel(self.position, other.position, fluids::SMOOTHING_LENGHT);
        }

        qtity_i
    }

    pub fn interpolate_grad(&self, others: &[SmoothedParticle], qtity_i: f32, qtities_j: &[f32]) -> Vec3 {
        let mut qtity: Vec3 = Vec3::ZERO;
        
        for (other, qtity_j) in others.iter().zip(qtities_j.iter()) {
            qtity += other.mass * ( qtity_i / self.density.powi(2) + qtity_j / other.density.powi(2)) * kernel::smoothing_kernel_grad(self.position, other.position, fluids::SMOOTHING_LENGHT);
        }

        self.density * qtity
    }

    pub fn interpolate_div(&self, others: &[SmoothedParticle], qtity_i: &Vec3, qtities_j: &[Vec3]) -> f32 {
        let mut qtity: f32 = 0.0;
        
        for (other, qtity_j) in others.iter().zip(qtities_j.iter()) {
            qtity += other.mass * (*qtity_i - *qtity_j).dot(kernel::smoothing_kernel_grad(self.position, other.position, fluids::SMOOTHING_LENGHT)); 
        }

        -1.0/self.density * qtity
    }

    pub fn interpolate_curl(&self, others: &[SmoothedParticle], qtity_i: &Vec3, qtities_j: &[Vec3]) -> Vec3 {
        let mut qtity: Vec3 = Vec3::ZERO;
        
        for (other, qtity_j) in others.iter().zip(qtities_j.iter()) {
            qtity += other.mass * (*qtity_i - *qtity_j).cross(kernel::smoothing_kernel_grad(self.position, other.position, fluids::SMOOTHING_LENGHT)); 
        }

        1.0/self.density * qtity
    }
    
    pub fn interpolate_lap(&self, others: &[SmoothedParticle], qtity_i: f32, qtities_j: &[f32]) -> f32 {
        let mut qtity: f32 = 0.0;
        
        for (other, qtity_j) in others.iter().zip(qtities_j.iter()) {
            let x_ij = self.position - other.position;
            qtity += other.mass / other.density * (qtity_i - qtity_j) * (x_ij.dot(kernel::smoothing_kernel_grad(self.position, other.position, fluids::SMOOTHING_LENGHT)) / (x_ij.dot(x_ij) + 0.01 * fluids::SMOOTHING_LENGHT.powi(2))); 
        }

        2.0 * qtity 
    }
}

