use glam::Vec3A;

pub trait Solver {
    fn support_radius(&self) -> f32;
    fn particle_radius(&self) -> f32;
    fn dimensions(&self) -> u32;
    fn viscosity(&self) -> f32;

    fn get_density(&self, p_i: usize) -> &f32;
    fn get_velocity(&self, p_i: usize) -> &Vec3A;
    fn get_mass(&self, p_i: usize) -> &f32;
    
    fn set_velocity(&mut self, p_i: usize, vel: Vec3A);

    fn cubic_kernel(&self, r_norm: f32) -> f32 {
        let h = self.support_radius();
        let mut l = match self.dimensions() {
            1 => 4.0 / 3.0,
            2 => 40.0 / 7.0 / std::f32::consts::PI,
            3 => 4.0 / std::f32::consts::PI,
            _ => 1.0
        };

        l /= h.powi(self.dimensions());

        let q = r_norm / h;
        
        if q <= 1.0 {
            if q <= 0.5 {
                let q2 = q * q;
                let q3 = q2 * q;

                l * (6.0 * q3 - 6.0 * q2 + 1)
            } else {
                l * 2.0 * (1.0 - q).powi(3)
            }
        } else {
            0.0        
        }
    }

    fn cubic_kernel_derivative(&self, r: &Vec3A) -> Vec3A  {
        let h = self.support_radius();
        let mut l = match self.dimensions() {
            1 => 4.0 / 3.0,
            2 => 40.0 / 7.0 / std::f32::consts::PI,
            3 => 8.0 / std::f32::consts::PI,
            _ => 1.0
        };
        
        l = 6.0 * l / h * self.dimensions();
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

    fn viscosity_foce(&self, p_i: usize, p_j: usize, r: &Vec3A) {
        let v_xy = (self.get_velocity(p_i) - self.get_velocity(p_j)).dot(r);

        2.0 * ((self.dimensions() + 2) as f32) * self.viscosity() * (self.get_mass(p_j) / self.get_density(p_j)) * v_xy / (
            r.length().powi(2) * 2.0 + self.particle_radius() * self.support_radius().powi(2)) * self.cubic_kernel_derivative(r)
    }

    fn simulate_collisions(&mut self, p_i: usize, vec: Vec3A) {
        let c_f = 0.2;
        let new_v = self.get_velocity(p_i) - (1.0 + c_f) * self.get_velocity(p_i).dot(vec) * vec;
        self.set_velocity(p_i, new_v);
    }

    fn enforce_boundary_2D(&self) {

    }

    fn enforce_boundary_3D(&self) {

    }

    fn step(&self) {

    }
}
