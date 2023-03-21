use fluid_renderer::*;
use fluid_renderer::winit::event::*;
use nikola::{setup_fluid_sim, step_fluid_sim, setup_boundary};

// use nikola::*;



fn main() {
    let InitOutput{event_loop, window, aspect_ratio} = init(); 

    let shader_source = fluid_renderer::wgpu::ShaderSource::Wgsl(std::fs::read_to_string("libs/fluid-renderer/src/shader.wgsl").unwrap().into());
    let vertices = Quad.scale(fluid_renderer::PARTICLE_SIZE);
    let indices = Quad::INDICES;
    let fluid_instances = create_grid(fluid_renderer::GRID_DIMENSIONS, (2, 2), (-1.0, -1.0, 0.0));
    let camera = Camera {
        aspect: aspect_ratio,
        fovy: 45.0,
        ..Default::default()
    };

    let mut fluid = setup_fluid_sim(&fluid_instances);
    let mut boundary_instances = setup_boundary();

    let mut instances = fluid_instances;
    instances.append(&mut boundary_instances);

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
                
                step_fluid_sim(&mut state, &mut fluid);
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
