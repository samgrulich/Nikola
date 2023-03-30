use glam::Vec3A;

use crate::ParticleSystem;

/// Trait with helpful functions for solver implmentation 
pub trait Solver {
    /// Get support radius
    fn support_radius(&self) -> f32;
    /// Get particle radius
    fn particle_radius(&self) -> f32;
    /// Get dimensions of domain
    fn dimensions(&self) -> u32;
    /// Get viscosity parameter
    fn viscosity(&self) -> f32;
    
    /// Access reference to the particle system
    fn ps(&self) -> &ParticleSystem;
    /// Access mutable reference to the particle system
    fn ps_mut(&mut self) -> &mut ParticleSystem;
    /// Get particle num (nubmer of particles in simulation)
    fn particle_num(&self) -> usize;
    /// Get padding of simulation domain
    fn padding(&self) -> Vec3A;
    /// Get domain starting point
    fn domain_start(&self) -> Vec3A;
    /// Get domain ending point
    fn domain_size(&self) -> Vec3A;

    /// Get density at particle
    ///
    /// # Arguments 
    /// * `p_i` - particle id
    fn get_density(&self, p_i: usize) -> &f32;
    /// Get velocity at particle
    ///
    /// # Arguments 
    /// * `p_i` - particle id
    fn get_v(&self, p_i: usize) -> Vec3A;
    /// Get mass at particle
    ///
    /// # Arguments 
    /// * `p_i` - particle id
    fn get_m(&self, p_i: usize) -> &f32;
    /// Get volume bound mass at particle
    ///
    /// # Arguments 
    /// * `p_i` - particle id
    fn get_m_v(&self, p_i: usize) -> &f32;
    
    /// Set velocity of particle
    ///
    /// # Arguments 
    /// * `p_i` - particle id
    /// * `vel` - new velocity
    fn set_v(&mut self, p_i: usize, vel: Vec3A);

    /// Sub step is run by particle system when stepping simulation
    fn sub_step(&mut self);

    /// Compute cubic spline smoothing kernel 
    ///
    /// # Arguments 
    /// * `r_norm` - normalized distance between particles
    fn cubic_kernel(&self, r_norm: f32) -> f32 {
        let h = self.support_radius();
        let mut l = match self.dimensions() {
            1 => 4.0 / 3.0,
            2 => 40.0 / 7.0 / std::f32::consts::PI,
            3 => 4.0 / std::f32::consts::PI,
            _ => 1.0
        };

        l /= h.powi(self.dimensions() as i32);

        let q = r_norm / h;
        
        if q <= 1.0 {
            if q <= 0.5 {
                let q2 = q * q;
                let q3 = q2 * q;

                l * (6.0 * q3 - 6.0 * q2 + 1.0)
            } else {
                l * 2.0 * (1.0 - q).powi(3)
            }
        } else {
            0.0        
        }
    }

    /// Computes gradient of cubic spline smoothing kernel
    ///
    /// # Arguments 
    /// * `r` - difference vector between two particles
    fn cubic_kernel_derivative(&self, r: Vec3A) -> Vec3A  {
        let h = self.support_radius();
        let mut l = match self.dimensions() {
            1 => 4.0 / 3.0,
            2 => 40.0 / 7.0 / std::f32::consts::PI,
            3 => 8.0 / std::f32::consts::PI,
            _ => 1.0
        };
        
        l = 6.0 * l / h * self.dimensions() as f32;
        let r_norm = r.length();
        let q = r_norm / h;

        if r_norm > 1e-5 && q <= 1.0 {
            let grad_q = r / (r_norm * h);
            if q <= 0.5 {
                l * q * (3.0 * q - 2.0) * grad_q
            } else {
                let factor = 1.0 - q;
                l * (-factor * factor) * grad_q
            }
        } else {
            Vec3A::ZERO
        }
    }

    /// Computes viscosity force acting between particles i and j
    ///
    /// # Arguments 
    /// * `p_i` - id of particle i
    /// * `p_j` - id of particle j
    /// * `r` - position difference between these two particles
    fn viscosity_force(&self, p_i: usize, p_j: usize, r: Vec3A) -> Vec3A {
        let v_xy = (self.get_v(p_i) - self.get_v(p_j)).dot(r);

        2.0 * ((self.dimensions() + 2) as f32) * self.viscosity() * (self.get_m(p_j) / self.get_density(p_j)) * v_xy / (
            r.length().powi(2) * 2.0 + self.particle_radius() * self.support_radius().powi(2)) * self.cubic_kernel_derivative(r)
    }

    /// Simulate collision for particle i along normal of the collision surface
    ///
    /// # Arguments
    /// * `p_i` - particle id
    /// * `vec` - normal vector of collision surface
    fn simulate_collisions(&mut self, p_i: usize, vec: Vec3A) {
        let c_f = 0.2;
        let new_v = self.get_v(p_i) - (1.0 + c_f) * self.get_v(p_i).dot(vec) * vec;
        self.set_v(p_i, new_v);
    }

    /// Keeps all particles inside given domain 
    fn enforce_boundary_3d(&mut self) {
       for p_i in 0..self.particle_num() {
            let mut collision_normal = Vec3A::ZERO;

            let max = (self.domain_start() + self.domain_size()) - self.padding();
            let min = self.domain_start() + self.padding();
            let x_i = &mut self.ps_mut().x[p_i];
            
            if x_i.x > max.x {
                collision_normal.x += 1.0;
                x_i.x = max.x;
            } else if x_i.x <= min.x {
                collision_normal.x -= 1.0;
                x_i.x = min.x; 
            }
            
            if x_i.y > max.y {
                collision_normal.y += 1.0;
                x_i.y = max.y;
            } else if x_i.y <= min.y {
                collision_normal.y -= 1.0;
                x_i.y = min.y; 
            }
            
            if x_i.z > max.z {
                collision_normal.z += 1.0;
                x_i.z = max.z;
            } else if x_i.z <= min.z {
                collision_normal.z -= 1.0;
                x_i.z = min.z; 
            }

            if collision_normal.length() > 1e-6 {
                self.simulate_collisions(p_i, collision_normal.normalize());
            }
       }
    }

    /// Step simulation
    fn step(&mut self) {
        self.ps_mut().initialize_particle_system();
        self.sub_step();
        self.enforce_boundary_3d();
    }
}
