use std::env;

use fluid_renderer::create_cube;
use nikola::{run_simulation, compute_simulation};



const INSTANCE_PARTICLE_SIZE: f32 = 1.0;
const SIMULATION_PARTICLE_SIZE: f32 = 0.08;

const SIMULATION_PATH: &str = "./simulation.nk";
const FPS: u32 = 60;

const FLUID_STEP_TIME: f32 = 0.0004;

fn main() {
    let particle_offset = (
       INSTANCE_PARTICLE_SIZE * 1.1, 
       INSTANCE_PARTICLE_SIZE * 1.1, 
       INSTANCE_PARTICLE_SIZE * 1.1, 
    );
    let instances = create_cube(0.01, (20, 20, 20), Some(particle_offset), (-1.0, -1.0, -2.0));

    let mut args = env::args().collect::<Vec<String>>();
    args.push(String::new());

    match args[1].as_str() {
        "run" => run_simulation(SIMULATION_PATH.to_string(), FPS, instances, INSTANCE_PARTICLE_SIZE),
        _ => compute_simulation(SIMULATION_PATH.to_string(), FPS, 10, FLUID_STEP_TIME, instances, SIMULATION_PARTICLE_SIZE)
    }
}
