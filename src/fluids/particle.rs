use glam::Vec3A;

use crate::{fluids, Neighborhood, smoothing_kernel};



pub trait Particle {
    fn id(&self) -> u32;
    fn position(&self) -> Vec3A;
    fn new(id: u32, position: Vec3A) -> Self;
    // fn pointer(&self) -> *const Self;
    
    const RADIUS: f32 = fluids::PARTICLE_RADIUS;
    const REST_DENSITY: f32 = fluids::REST_DENSITY;

    const RADIUS_POW_3: f32 = Self::RADIUS * Self::RADIUS * Self::RADIUS;
    const VOLUME: f32 = 4.0 / 3.0 * std::f32::consts::PI * Self::RADIUS_POW_3;
    const MASS: f32 = Self::VOLUME * Self::REST_DENSITY;
}


#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct BoundaryParticle {
    pub id: u32,
    pub position: Vec3A,
}

impl Particle for BoundaryParticle {
    fn id(&self) -> u32 {
        self.id
    }

    fn position(&self) -> Vec3A {
        self.position 
    }
    
    fn new(id: u32, position: Vec3A) -> Self {
        BoundaryParticle { id, position }
    }
}


#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct FluidParticle {
    pub id: u32,
    pub position: Vec3A,

    pub velocity: Vec3A,
    pub velocity_future: Vec3A,

    pub density: f32,
    pub density_future: f32,

    pub dsph_factor: f32,
    pub pressure: f32,
}

impl FluidParticle {
   pub fn interpolate_div_vf(&self, neighborhood: &Neighborhood<FluidParticle>) -> f32 {
        let mut qtity: f32 = 0.0;
        let mass_sum = neighborhood.get_len() * FluidParticle::MASS;
        
        for (i, neighbor) in neighborhood.neighbors.iter().enumerate() {
            unsafe {
                qtity += (self.velocity_future - (**neighbor).velocity_future).dot(neighborhood.gradients[i]); 
            }
        }

        -1.0/self.density * qtity * mass_sum
    }
}

impl FluidParticle {
    pub fn compute_density_derivate(&self, neighborhood: &Neighborhood<FluidParticle>) -> f32 {
        let mut density_div = 0.0;
       
        for (i, neighbor) in neighborhood.neighbors.iter().enumerate() {
            unsafe {
                density_div += (self.velocity - (**neighbor).velocity).dot(*neighborhood.gradients.get(i).unwrap());
            }
        }

        density_div * neighborhood.get_len() * Self::MASS
    }

    pub fn compute_dsph_factor(&self, neighborhood: &Neighborhood<FluidParticle>) -> f32 {
        let mut outter_sum = 0.0; 

        let mass_sum = neighborhood.get_len() * Self::MASS;
        let gradients_sum: Vec3A = neighborhood.gradients.iter().sum();

        for gradient in neighborhood.gradients.iter() {
            outter_sum += gradient.length().powi(2);
        }

        self.density.powi(2) / ((mass_sum * gradients_sum).length().powi(2) + (mass_sum.powi(2) * outter_sum))
    }

    pub fn compute_density_future(&self, neighborhood: &Neighborhood<FluidParticle>, boundary_neighborhood: &Neighborhood<BoundaryParticle>, delta_time: f32) -> f32 {
        let mut sum = 0.0;
        let mass_sum = neighborhood.get_len() * Self::MASS;

        for (i, neighbor) in neighborhood.neighbors.iter().enumerate() {
            unsafe {
                sum += (self.velocity_future - (**neighbor).velocity_future).dot(neighborhood.gradients[i]);
            }
        }

        let mut interpolants = 0.0;
        let mut b_interpolants = 0.0;
        neighborhood.neighbors.iter().for_each(|neighbor| interpolants += smoothing_kernel(self.position, unsafe{&**neighbor}.position));
        boundary_neighborhood.neighbors.iter().for_each(|neighbor| b_interpolants += smoothing_kernel(self.position, unsafe{&**neighbor}.position));

        let gamma_1 = if b_interpolants == 0.0 {
            0.0
        } else {
            (1.0 / FluidParticle::VOLUME - interpolants) / b_interpolants
        };

        // self.density + delta_time * sum * mass_sum
        self.density + gamma_1 * FluidParticle::MASS * b_interpolants + delta_time * sum * mass_sum
    }
}

impl Particle for FluidParticle {
    fn id(&self) -> u32 {
        self.id 
    }

    fn position(&self) -> Vec3A {
        self.position 
    }
    
    fn new(
        id: u32,
        position: Vec3A, 
    ) -> Self {
        FluidParticle {
            id,
            position,
            velocity: Vec3A::ZERO,
            velocity_future: Vec3A::ZERO,
            density: Self::REST_DENSITY,
            density_future: Self::REST_DENSITY,
            dsph_factor: 0.0,
            pressure: 0.0,
        }
    }
}

