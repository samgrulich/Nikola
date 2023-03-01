use bevy::prelude::*;

use std::rc::Rc;

use crate::fluids::kernel;
use crate::fluids;

#[derive(Debug, Clone, Copy)]
pub struct SmoothedParticle {
    pub id: i32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub velocity_predict: Vec3,

    pub density: f32,
    pub density_over_time:f32,
    pub density_predict: f32,

    pub dsph_factor: f32,
    pub pressure: f32,
    pub pressure_value: f32,

    pub mass: f32,
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
            density_over_time: 0.0, // todo: 
            density_predict: density,
            dsph_factor: 0.0, // todo: check if the intial values need to be changed
            pressure: 0.0,
            pressure_value: 0.0, // todo: are the pressures collapsable?
            mass,
        }
    }
}

impl SmoothedParticle {
    pub fn get_vec3(&self, field: &str) -> Option<Vec3> {
        match field {
            "velocity" => Some(self.velocity),
            "velocity_predict" => Some(self.velocity_predict),
            _ => None,
        }
    }

    pub fn get_f32(&self, field: &str) -> Option<f32> {
        match field {
            "density" => Some(self.density),
            "density_over_time" => Some(self.density_over_time),
            "density_predict" => Some(self.density_predict),
            "dsph_factor" => Some(self.dsph_factor),
            "pressure" => Some(self.pressure),
            "mass" => Some(self.mass),
            _ => None,
        }
    }
}

impl SmoothedParticle {
    pub fn interpolate(&self, others: &Vec<Rc<SmoothedParticle>>, field: &str) -> f32 {
        let mut qtity_i: f32 = 0.0;

        for other in others.iter() {
            if let Some(qtity_j) = other.get_f32(field) {
                qtity_i += other.mass / other.density * qtity_j * kernel::smoothing_kernel(self.position, other.position, None);
            }
        }

        qtity_i
    }

    pub fn interpolate_grad(&self, others: &Vec<Rc<SmoothedParticle>>, field: &str) -> Vec3 {
        let mut qtity = Vec3::ZERO;
        let qtity_i = self.get_vec3(field).unwrap();
        
        for other in others.iter() {
            if let Some(qtity_j) = other.get_vec3(field) {
                qtity += other.mass * ( qtity_i / self.density.powi(2) + qtity_j / other.density.powi(2)) * kernel::smoothing_kernel_grad(self.position, other.position, None);
            }
        }

        self.density * qtity
    }
    
    pub fn interpolate_grad_f32(&self, others: &Vec<Rc<SmoothedParticle>>, field: &str) -> Vec3 {
        let mut qtity = Vec3::ZERO;
        let qtity_i = self.get_f32(field).unwrap();
        
        for other in others.iter() {
            if let Some(qtity_j) = other.get_f32(field) {
                qtity += other.mass * ( qtity_i / self.density.powi(2) + qtity_j / other.density.powi(2)) * kernel::smoothing_kernel_grad(self.position, other.position, None);
            }
        }

        self.density * qtity
    }

    pub fn interpolate_div(&self, others: &Vec<Rc<SmoothedParticle>>, field: &str) -> f32 {
        let mut qtity: f32 = 0.0;
        let qtity_i = self.get_vec3(field).unwrap();
        
        for other in others.iter() {
            if let Some(qtity_j) = other.get_vec3(field) {
                qtity += other.mass * (qtity_i - qtity_j).dot(kernel::smoothing_kernel_grad(self.position, other.position, None)); 
            }
        }

        -1.0/self.density * qtity
    }

    pub fn interpolate_curl(&self, others: &Vec<Rc<SmoothedParticle>>, field: &str) -> Vec3 {
        let mut qtity: Vec3 = Vec3::ZERO;
        let qtity_i = self.get_vec3(field).unwrap();
        
        for other in others.iter() {
            if let Some(qtity_j) = other.get_vec3(field) {
                qtity += other.mass * (qtity_i - qtity_j).cross(kernel::smoothing_kernel_grad(self.position, other.position, None)); 
            }
        }

        1.0/self.density * qtity
    }
    
    // todo: implement laplacian for scalars
    pub fn interpolate_lap(&self, others: &Vec<Rc<SmoothedParticle>>, field: &str) -> Vec3 {
        let mut qtity = Vec3::ZERO;
        let qtity_i = self.get_vec3(field).unwrap();
        
        for other in others.iter() {
            let x_ij = self.position - other.position;

            if let Some(qtity_j) = self.get_vec3(field) {
                qtity += other.mass / other.density * (qtity_i - qtity_j) * (x_ij.dot(kernel::smoothing_kernel_grad(self.position, other.position, None)) / (x_ij.dot(x_ij) + 0.01 * fluids::SMOOTHING_LENGHT.powi(2))); 
            }
        }

        2.0 * qtity 
    }
}

impl SmoothedParticle {
    pub fn compute_density_derivate(&self, others: &Vec<Rc<SmoothedParticle>>) -> f32 {
        let mut density_div = 0.0;
        
        for other in others.iter() {
            density_div += other.mass * (self.velocity - other.velocity).dot(kernel::smoothing_kernel_grad(self.position, other.position, None));
        }

        density_div
    }

    pub fn compute_dsph_factor(&self, others: &Vec<Rc<SmoothedParticle>>) -> f32 {
        let mut inner_sum = Vec3::ZERO;
        let mut outter_sum = 0.0; 

        for other in others {
            inner_sum += other.mass * kernel::smoothing_kernel_grad(self.position, other.position, None);
            outter_sum += (other.mass * kernel::smoothing_kernel_grad(self.position, other.position, None)).length().powi(2);
        }

        self.density.powi(2) / inner_sum.length().powi(2) + outter_sum
    }

    pub fn compute_density_predict(&self, others: &Vec<Rc<SmoothedParticle>>, delta_time: f32) -> f32 {
        let mut sum = 0.0;

        for other in others {
            sum += other.mass * (self.velocity_predict - other.velocity_predict).dot(kernel::smoothing_kernel_grad(self.position, other.position, None));
        }

        self.density + delta_time * sum
    }

    pub fn compute_density_predict_inplace(&mut self, others: &Vec<Rc<SmoothedParticle>>, delta_time: f32) {
        self.density = self.compute_density_predict(others, delta_time);
    }
}

