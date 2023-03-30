use fluid_renderer::Instance;
use glam::{vec3a, Vec3A};

use crate::{Solver, ParticleSystem, Config};


/// Weakly Compresible Smoothed Particle Hydrodynamics solver, stores
/// current state of fluid and provides functions to manipulate with 
/// fluid's particles
pub struct WCSPHSolver {
    ps: ParticleSystem,
    
    pub viscosity: f32,
    pub density_0: f32,

    pub stiffness: f32,
    pub surface_tension: f32,
    pub delta_time: f32,
}

impl WCSPHSolver {
    const G: Vec3A = vec3a(0.0, -9.81, 0.0);
}

impl Solver for WCSPHSolver {
    fn support_radius(&self) -> f32 {
        self.ps.support_radius
    }

    fn particle_radius(&self) -> f32 {
        self.ps.particle_radius
    }

    fn dimensions(&self) -> u32 {
        3
    }

    fn viscosity(&self) -> f32 {
        self.viscosity
    }

    
    fn ps(&self) -> &ParticleSystem {
        &self.ps
    }

    fn ps_mut(&mut self) -> &mut ParticleSystem {
        &mut self.ps
    }

    fn particle_num(&self) -> usize {
        self.ps.particle_num
    }

    fn padding(&self) -> Vec3A {
        Vec3A::splat(self.ps.particle_radius)
    }

    fn domain_size(&self) -> Vec3A {
        self.ps.domain_size
    }


    fn get_density(&self, p_i: usize) -> &f32 {
        &self.ps.density[p_i]
    }

    fn get_v(&self, p_i: usize) -> Vec3A {
        self.ps.v[p_i]
    }

    fn get_m(&self, p_i: usize) -> &f32 {
        &self.ps.m[p_i]
    }
    
    fn get_m_v(&self, p_i: usize) -> &f32 {
        &self.ps.m_v[p_i]
    }

    fn set_v(&mut self, p_i: usize, vel: glam::Vec3A) {
        self.ps.v[p_i] = vel
    }

    fn domain_start(&self) -> Vec3A {
        self.ps.domain_start
    }
    
    fn sub_step(&mut self) {
        self.compute_densities();
        self.compute_non_pressure_forces();
        self.compute_pressure_forces();
        self.advect();
    }
}

impl WCSPHSolver {
    /// Create new WCSPH solver 
    ///
    /// # Arguments 
    /// * `viscosity` - viscosity coeficient (user set)
    /// * `stiffness` - pressure multiplier (user set)
    /// * `surface_tension` - surface tension coeficient (user set)
    /// * `delta_time` - length of time step (s) 
    /// * `particle_config` - configuration of particle system
    pub fn new(
        viscosity: f32, 
        stiffness: f32, 
        surface_tension: f32, 
        delta_time: f32, 
        particle_config: Config
    ) -> Self {
        let density_0 = particle_config.density_0;
        let mut ps = ParticleSystem::new(particle_config);
        ps.initialize_particle_system();

        WCSPHSolver { 
            ps, 
            viscosity, 
            density_0, 
            stiffness, 
            surface_tension, 
            delta_time 
        }
    }

    /// Computes density for particle i influenced by j and adds the result to ret
    ///
    /// # Arguments
    /// * `p_i` - id of particle i
    /// * `p_j` - id of particle j
    /// * `ret` - mutable reference, where the result will be added
    fn compute_densities_task(&self, p_i: usize, p_j: usize, ret: &mut f32) {
        let x_i = self.ps.x[p_i];
        let x_j = self.ps.x[p_j];

        *ret += self.ps.m_v[p_j] * self.cubic_kernel((x_i - x_j).length());
    }

    /// Updates density for each particle 
    pub fn compute_densities(&mut self) {
        for p_i in 0..self.particle_num() {
            self.ps.density[p_i] = self.ps.m_v[p_i] * self.cubic_kernel(0.0);
            let mut density_i = 0.0;
            self.ps.for_all_neighbords(p_i, |p_i, p_j, ret| self.compute_densities_task(p_i, p_j, ret), &mut density_i);
            self.ps.density[p_i] += density_i;
            self.ps.density[p_i] *= self.density_0;
        }
    }

    /// Compute pressure force acting on particle i from particle j and adds result to ret
    ///
    /// # Arguments
    /// * `p_i` - id of particle i
    /// * `p_j` - id of particle j
    /// * `ret` - mutable reference, where the result will be added
    fn compute_pressure_forces_task(&self, p_i: usize, p_j: usize, ret: &mut Vec3A) {
        let x_i = self.ps.x[p_i];
        let dpi = self.ps.pressure[p_i] / self.ps.density[p_i].powi(2);

        let x_j = self.ps.x[p_j];
        let dpj = self.ps.density[p_j] / self.ps.density[p_j].powi(2);

        *ret += -self.density_0 * self.ps.m_v[p_j] * (dpi + dpj) * self.cubic_kernel_derivative(x_i - x_j);
    }

    /// Updates pressure forces for each particle
    pub fn compute_pressure_forces(&mut self) {
        for p_i in 0..self.ps.x.len() {
            self.ps.density[p_i] = self.ps.density[p_i].max(self.density_0);
            self.ps.pressure[p_i] = self.stiffness * ((self.ps.density[p_i] / self.density_0) - 1.0);
        }
        for p_i in 0..self.ps.x.len() {
            let mut dv = Vec3A::ZERO;
            self.ps.for_all_neighbords(p_i, |p_i, p_j, ret| self.compute_pressure_forces_task(p_i, p_j, ret), &mut dv);
            self.ps.acceleration[p_i] += dv;
        }
    }

    /// Computes non-pressure forces acting on particle i from particle j and adds the result to
    /// the ret
    ///
    /// # Arguments
    /// * `p_i` - id of particle i
    /// * `p_j` - id of particle j
    /// * `ret` - mutable reference, where the result will be added
    fn compute_non_pressure_forces_task(&self, p_i: usize, p_j: usize, ret: &mut Vec3A) {
        let x_i = self.ps.x[p_i];
        let x_j = self.ps.x[p_j];

        // Compute Surface Tension
        let diam2 = self.ps.particle_diameter.powi(2);

        let r = x_i - x_j;
        let r2 = r.dot(r);

        if r2 > diam2 {
            *ret -= self.surface_tension / self.ps.m[p_i] * self.ps.m[p_j] * r * self.cubic_kernel(r.length());
        } else {
            *ret -= self.surface_tension / self.ps.m[p_i] * self.ps.m[p_j] * r * self.cubic_kernel(self.ps.particle_diameter); // possible bug
        }

        // Viscosity Force
        let d = 2.0 * (self.dimensions() + 2) as f32;
        let v_xy = (self.ps.v[p_i] - self.ps.v[p_j]).dot(r);

        let f_v = d * self.viscosity * (self.ps.m[p_j] / (self.ps.density[p_j])) * v_xy / (
            r.length().powi(2) + 0.01 * self.ps.support_radius.powi(2)) * self.cubic_kernel_derivative(r);
        *ret += f_v;
    }

    /// Updates non-pressure acceleration for each particle
    pub fn compute_non_pressure_forces(&mut self) {
        for p_i in 0..self.ps.x.len() {
            let mut d_v = Self::G;
            self.ps.for_all_neighbords(p_i, |p_i, p_j, ret| self.compute_non_pressure_forces_task(p_i, p_j, ret), &mut d_v);
            self.ps.acceleration[p_i] = d_v;
        }
    }

    /// For each particle applies its acceleration and velocity
    pub fn advect(&mut self) {
        for p_i in 0..self.ps.x.len() {
            self.ps.v[p_i] += self.delta_time * self.ps.acceleration[p_i];
            self.ps.x[p_i] += self.delta_time * self.ps.v[p_i];
        }
    }

    /// Set position of each instance to according particle position
    ///
    /// # Arguments
    /// * `instances` - instances to advect
    pub fn advect_instances(&self, instances: &mut Vec<Instance>) {
        for (particle_id, instance_id) in self.ps.ids.iter().enumerate() {
            instances[*instance_id].position = self.ps.x[particle_id].into();
        }
    }
}
