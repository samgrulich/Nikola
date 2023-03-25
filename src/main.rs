use fluid_renderer::*;
use fluid_renderer::winit::event::*;
use glam::vec3a;
use nikola::{Config, WCSPHSolver, Solver};


fn main() {
    let InitOutput{event_loop, window, aspect_ratio} = init(); 

    let shader_source = fluid_renderer::wgpu::ShaderSource::Wgsl(std::fs::read_to_string("libs/fluid-renderer/src/shader.wgsl").unwrap().into());
    let particle_size = 0.08;
    let vertices = Quad.scale(particle_size);
    let indices = Quad::INDICES;
    let particle_offset = (
       particle_size * 1.1, 
       particle_size * 1.1, 
       particle_size * 1.1, 
    );
    let instances = create_cube(0.0, (40, 40, 40), Some(particle_offset), (-1.0, -1.0, -2.0));
    let config = Config::from_instances( 
        vec3a(-40.0, -20.0, -40.0),
        vec3a(40.0, 20.0, 0.0),
        particle_size,
        1000.0,
        &instances
    );
    
    let fluid_step_time = 0.00004;
    let mut fluid = WCSPHSolver::new(
        0.01,
        50000.0,
        0.01,
        fluid_step_time,
        config
    );

    let camera = Camera {
        aspect: aspect_ratio,
        fovy: 45.0,
        eye: vec3a(0.0, 0.0, 100.0),
        zfar: 1000.0,
        ..Default::default()
    };


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

    // let frames = 60.0; 
    // let steps_per_frame = frames / fluid_step_time;
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

                fluid.step(&mut state.instances);
                state.update_instances();

                fluid_renderer::handle_rendering(&mut state, control_flow);
            }
            Event::MainEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}
