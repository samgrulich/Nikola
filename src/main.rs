use std::env;

use fluid_renderer::create_cube;
use nikola::{run_simulation, compute_simulation};



const INSTANCE_PARTICLE_SIZE: f32 = 2.0;
const SIMULATION_PARTICLE_SIZE: f32 = 2.0;

const SIMULATION_PATH: &str = "./simulation.nk";
const FPS: u32 = 60;

const FLUID_STEP_TIME: f32 = 0.004;

fn main() {
    let particle_offset = (
       SIMULATION_PARTICLE_SIZE * 1.0, 
       SIMULATION_PARTICLE_SIZE * 1.0, 
       SIMULATION_PARTICLE_SIZE * 1.0, 
    );
    let instances = create_cube(0.04, (14, 14, 14), Some(particle_offset), (-1.0, -1.0, -1.0));

    let mut args = env::args().collect::<Vec<String>>();

    if args.len() == 1 {
        args.push(String::new());
    }

    match args[1].as_str() {
        "run" => {
            dbg!(&args);
            let path = if args.len() >= 3 {
                args[2].clone()
            } else {
                SIMULATION_PATH.to_string()
            };

            println!("Loading: {}", path);
            run_simulation(path, FPS, INSTANCE_PARTICLE_SIZE)
        },
        _ => compute_simulation(SIMULATION_PATH.to_string(), FPS, 10, FLUID_STEP_TIME, instances, SIMULATION_PARTICLE_SIZE, 1.0)
    }
}
