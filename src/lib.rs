mod config;
mod solver;
mod wcsph;
mod particles_system;
mod simulation;

use std::thread::sleep;
use std::time::{Instant, Duration};

pub use config::*;
pub use solver::*;
pub use wcsph::*;
pub use particles_system::*;
pub use simulation::*;

use glam::vec3a;
use fluid_renderer::*;
use fluid_renderer::winit::event::*;


pub fn run_simulation(simulation_path: String, fps: u32, particle_size: f32) {
    let InitOutput{event_loop, window, aspect_ratio} = init(); 
    let shader_source = fluid_renderer::wgpu::ShaderSource::Wgsl(std::fs::read_to_string("libs/fluid-renderer/src/shader.wgsl").unwrap().into());
    let vertices = Quad.scale(particle_size);
    let indices = Quad::INDICES;
    
    let mut simulation = Simulation::from_file(simulation_path).unwrap();
    
    let camera = Camera {
        aspect: aspect_ratio,
        fovy: 45.0,
        eye: vec3a(-200.0, 200.0, 1000.0) / 2.0,
        zfar: 10000.0,
        ..Default::default()
    };

    let instances = (0..simulation.particle_num).map(|_id| Instance::new()).collect();

    let mut state = pollster::block_on(
        State::new(
            window, 
            shader_source, 
            vertices.as_slice(), 
            indices, 
            instances, 
            camera
        )
    );

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                crate::hadnle_windowing(&mut state, event, control_flow)
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();

                // fluid.step(&mut state.instances);
                simulation.step_instances(&mut state.instances);
                state.update_instances();

                fluid_renderer::handle_rendering(&mut state, control_flow);
                sleep(Duration::from_millis((1000.0 / fps as f32) as u64));
            }
            Event::MainEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}


pub fn compute_simulation(
    path: String, 
    fps: u32, 
    simulation_time: u32,
    fluid_step_time: f32, 
    instances: Vec<Instance>, 
    particle_size: f32,
) {
    let config = Config::from_instances( 
        vec3a(-80.0, -80.0, -80.0),
        vec3a(80.0, 80.0, 80.0),
        particle_size,
        1000.0,
        &instances
    );
    let mut fluid = WCSPHSolver::new(
        0.01,
        50000.0,
        0.01,
        fluid_step_time,
        config
    );

    let frame_stop = (simulation_time * fps) as u32;
    let steps_per_frame = (1.0 / fluid_step_time / fps as f32).ceil() as u32;
    
    let mut simulation = Simulation::new(fps, frame_stop, instances.len() as u32);

    println!("Starting simulation");

    let total_time = Instant::now();
    for frame in 0..frame_stop {
        let start = Instant::now();
        for _step in 0..steps_per_frame {
            fluid.step();
        }
        for (particle_id, instance_id) in fluid.ps().ids.iter().enumerate() {
            let index = frame as usize * instances.len() + *instance_id;
            simulation.frames[index] = fluid.ps().x[particle_id];
        }
        println!("progress: {}/{} {}%, {}s", frame, frame_stop, frame*100/frame_stop, start.elapsed().as_millis() as f32 / 1000.0);
    }

    println!("Done {}s", total_time.elapsed().as_millis() as f32 / 1000.0);

    simulation.save(path).unwrap();
}

