use std::rc::Rc;
use crate::fluids::{particle::SmoothedParticle, neighborhoods::Neighborhoods};

// const SPEED_OF_SOUND: f32 = 0.3;
// const SPEED_OF_SOUND: f32 = 1480.0; // m/s in water
const SPEED_OF_SOUND_2: f32 = 2_190_400.0;

pub fn state_of_equation(density_i: f32, rest_density: f32, constant: f32) -> f32 {
    let pressure = constant * (density_i / rest_density - 1.0);

    pressure
}

pub fn state_of_equation_sound(density: f32) -> f32 {
    SPEED_OF_SOUND_2 * density
}

pub struct Fluid {
    particles: Vec<Rc<SmoothedParticle>>,
    neighborhoods: Neighborhoods,

    rest_density: f32,
    average_density: f32,

    max_velocity: f32,
    time_delta: f32,
}

impl Fluid {
    pub fn get_average_density(&self) -> f32 {
        let mut density_sum = 0.0;

        self.particles.iter().for_each(|particle| density_sum += particle.density);

        density_sum / self.particles.len() as f32
    }

    pub fn get_max_velocity(&self) -> f32 {
        let mut max_velocity = 0.0;

        self.particles.iter().for_each(|particle| if particle.velocity.length() > max_velocity { max_velocity = particle.velocity.length() });

        max_velocity
    }
}

impl Fluid {
    fn correct_density(&self, threshold: f32) {
        let mut iteration = 0;

        while (iteration < 2) && (self.average_density - self.rest_density > threshold) {
            iteration += 1;
        }
        // for particles i compute density_predict
        //
        // for particles i compute p_i
        //  velocity_predict = velocity_predict - delta_time * sum ...
    }

    fn correct_divergence() {
        // for particles i 
    }

    pub fn dfsph(&self, particles: &[SmoothedParticle], delta_time: f32) {
        // let k_factor = self.compute_dsph_factor(others);
        // let pressure_value = 1.0 / delta_time * self.compute_density_derivate(others) * self.density.powi(2) / k_factor;

        // compute nonp acceleration
        // adapt delta time
        
        // for particles i predict velocity v_predict = v_i + time_delta * a_i_nonp
        // correct density error using constant density solver

        // for particles i update position
        // update neighborhoods (refresh hash table)

        // for particles do 
        //  update density 
        //  update k_factor
        // correct divergence using divergence solver 
    }
}
