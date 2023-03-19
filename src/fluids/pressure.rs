use core::time;
use std::thread;

use glam::Vec3A;
use crate::{
    fluids::{
        particle::SmoothedParticle, 
        neighborhoods::Neighborhood,
    }, 
    Fluid, GRAVITATIONAL_ACCELERATION,
};

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


pub fn correct_density(solver: &mut Fluid, neighborhoods: &Vec<Neighborhood>) {
    let mut iteration = 0;
    let frac_delta_time_squared: f32 = 1.0 / solver.delta_time.powi(2);


    while (iteration < 2) || (solver.get_average_density() - SmoothedParticle::REST_DENSITY > solver.density_threshold) {
        for particle in solver.table.particles.iter_mut() {
            let neighborhood = &neighborhoods[particle.id as usize];
            
            particle.density_future = particle.compute_density_future(&neighborhood, solver.delta_time);
        }

        for particle in solver.table.particles.iter_mut() {
            particle.pressure = frac_delta_time_squared * (particle.density_future - SmoothedParticle::REST_DENSITY) * particle.dsph_factor;
        }

        for particle in solver.table.particles.iter_mut() {
            let mut sum = Vec3A::ZERO;
            let neighborhood = &neighborhoods[particle.id as usize];

            let neighbors_len = neighborhood.neighbors.len() as f32;
            let mass_sum = neighbors_len * SmoothedParticle::MASS;
            let gradients_sum: Vec3A = neighborhood.gradients.iter().sum();

            for neighbor in neighborhood.neighbors.iter() {
                let neighbor = unsafe { &**neighbor };
                sum += neighbor.pressure / neighbor.density.powi(2)
            }

            particle.velocity_future = 
                particle.velocity_future - solver.delta_time * mass_sum * gradients_sum 
                * (particle.pressure / particle.density.powi(2) * neighbors_len + sum);
        }

        iteration += 1;
    }
}


fn correct_divergence(solver: &mut Fluid, neighborhoods: &Vec<Neighborhood>) {
    let mut iteration = 0;
    let mut average_density_over_time = 0.0;

    let frac_delta_time = 1.0 / solver.delta_time;

    let mut densities_over_time: Vec<f32> = vec![0.0; solver.table.particles.len()];
    let mut pressure_values: Vec<f32> = vec![0.0; solver.table.particles.len()];

    while (iteration < 1) || (average_density_over_time > solver.divergence_threshold) {
        for (i, particle) in solver.table.particles.iter().enumerate() {
            let density_over_time_i = -particle.density * particle.interpolate_div_vf(&neighborhoods[i]); 

            densities_over_time[particle.id as usize] = density_over_time_i;
            pressure_values[i] = frac_delta_time * densities_over_time[i] * particle.dsph_factor;
        }

        for (i, particle) in solver.table.particles.iter_mut().enumerate() {
            let mut sum = Vec3A::ZERO;
            let neighborhood = &neighborhoods[i];

            let neighbors_len = neighborhood.neighbors.len() as f32;
            let mass_sum = neighbors_len * SmoothedParticle::MASS;
            let gradients_sum: Vec3A = neighborhood.gradients.iter().sum();
            let pressure_value_i_sum = pressure_values[i] / particle.density.powi(2) * neighbors_len;

            for neighbor in neighborhood.neighbors.iter() {
                let neighbor = unsafe { &**neighbor };
                sum += pressure_values[neighbor.id as usize] / neighbor.density.powi(2);
            }

            particle.velocity_future = 
                particle.velocity_future - solver.delta_time 
                * gradients_sum * mass_sum 
                * (pressure_value_i_sum + sum);
        }
        
        average_density_over_time = densities_over_time.iter().sum::<f32>() / solver.table.particles.len() as f32;
        iteration += 1;
    }
}

pub fn dfsph(solver: &mut Fluid) {
    // compute nonp acceleration
    
    // adapt delta time
    solver.apply_cfl();
    
    // for particles i predict velocity v_predict = v_i + time_delta * a_i_nonp
    for particle in solver.table.particles.iter_mut() {
        particle.velocity_future = particle.velocity + solver.delta_time * GRAVITATIONAL_ACCELERATION;
    }

    let neighborhoods: Vec<Neighborhood> = solver.table.particles.iter().map(|particle| {
        solver.table.get_neighborhood_2d(particle.id) 
    }).collect();

    // correct density error using constant density solver
    correct_density(solver, &neighborhoods);

    // for particles i update position
    for particle in solver.table.particles.iter_mut() {
        particle.position += particle.velocity_future * solver.delta_time;
    }

    // update neighborhoods (refresh hash table)
    solver.table.update();

    let neighborhoods: Vec<Neighborhood> = solver.table.particles.iter().map(|particle| {
        solver.table.get_neighborhood_2d(particle.id) 
    }).collect();

    // for particles do 
    for (i, particle) in solver.table.particles.iter_mut().enumerate() {
        particle.density = particle.density_future;

        let neighborhood = &neighborhoods[i];
        particle.dsph_factor = particle.compute_dsph_factor(&neighborhood);
    }

    // correct divergence using divergence solver 
    correct_divergence(solver, &neighborhoods);

    // update velocity
    for particle in solver.table.particles.iter_mut() {
        particle.velocity = particle.velocity_future;
    }
}

