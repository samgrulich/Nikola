use std::{thread::sleep, time::Duration};

use fluid_renderer::*;
use fluid_renderer::winit::event::*;
use glam::Vec3A;
use nikola::{setup_fluid_sim, step_fluid_sim};

// use nikola::*;



fn main() {
    let InitOutput{event_loop, window, aspect_ratio} = init(); 

    let shader_source = fluid_renderer::wgpu::ShaderSource::Wgsl(std::fs::read_to_string("libs/fluid-renderer/src/shader.wgsl").unwrap().into());
    let vertices = Quad.scale(fluid_renderer::PARTICLE_SIZE);
    let indices = Quad::INDICES;
    // let instances = create_grid((3, 2), (2, 2), (-0.0, -0.0, 0.0));
    let instances = vec![
        Instance::default(),
        Instance::default(),
    ];
    let camera = Camera {
        aspect: aspect_ratio,
        fovy: 45.0,
        ..Default::default()
    };

    let mut fluid = setup_fluid_sim(&instances);

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

    let mut is_playing = false;
    state.update_instances();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                match event {
                    WindowEvent::KeyboardInput { 
                        input: KeyboardInput {
                            state: ElementState::Pressed, 
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            ..
                        },
                        ..
                    } => is_playing = true,
                    WindowEvent::KeyboardInput { 
                        input: KeyboardInput {
                            state: ElementState::Released, 
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            ..
                        },
                        ..
                    } => is_playing = false,
                    _ => crate::hadnle_windowing(&mut state, event, control_flow),
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                
                if is_playing {
                    dbg!("-------------------");
                    dbg!("STEP-----------STEP");
                    dbg!("-------------------");

                    // dbg!(&fluid.table);
                    // fluid.table.get_neighborhood_2d(0).neighbors.iter().for_each(|neighbor| {
                    //     dbg!(neighbor, unsafe{&**neighbor}); 
                    // });

                    fluid.table.particles.iter().for_each(|particle| {
                        dbg!(particle); 
                        // let neighborhood = fluid.table.get_neighborhood_2d(particle.id);
                    });

                    step_fluid_sim(&mut state, &mut fluid);
                    state.update_instances();

                    sleep(Duration::from_millis(100));
                }


                fluid_renderer::handle_rendering(&mut state, control_flow);
            }
            Event::MainEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}
