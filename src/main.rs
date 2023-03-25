use fluid_renderer::*;
use fluid_renderer::winit::event::*;
use glam::vec3a;
use nikola::{Config, WCSPHSolver, Solver};

// use nikola::*;



fn main() {
    let InitOutput{event_loop, window, aspect_ratio} = init(); 

    let shader_source = fluid_renderer::wgpu::ShaderSource::Wgsl(std::fs::read_to_string("libs/fluid-renderer/src/shader.wgsl").unwrap().into());
    let vertices = Quad.scale(fluid_renderer::PARTICLE_SIZE);
    let indices = Quad::INDICES;
    let particle_offset = (
        1.2 * fluid_renderer::PARTICLE_SIZE,
        1.2 * fluid_renderer::PARTICLE_SIZE,
        1.2 * fluid_renderer::PARTICLE_SIZE,
    );
    let instances = create_cube((40, 40, 40), None, (-1.0, -1.0, -2.0));
    let config = Config::from_instances( 
        vec3a(-1.0, -1.0, -2.0),
        vec3a(1.0, 1.0, 0.0),
        fluid_renderer::PARTICLE_SIZE,
        1000.0,
        &instances
    );
    
    let mut fluid = WCSPHSolver::new(
        0.01,
        1,
        50000.0,
        0.01,
        0.004,
        config
    );


    let camera = Camera {
        aspect: aspect_ratio,
        fovy: 45.0,
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
