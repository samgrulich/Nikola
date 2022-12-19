mod general;
mod computer;
mod renderer;

use winit::{
    event_loop::EventLoop,
    event::{Event, WindowEvent},
    window::Window,
};

fn main() {
    let event_loop = EventLoop::new();    
    let window = winit::window::WindowBuilder::new()
        .with_title("Test window")
        .with_visible(true)
        .build(&event_loop)
        .unwrap();

   pollster::block_on(run(event_loop, window));
}

async fn run(event_loop: EventLoop<()>, window: Window) {
 
    // gpu setup
    let mut state = general::State::new(&window).await;

    // compute shaders setup
    let computer = computer::compute_config(&state, window.inner_size()).await;
    let texture = computer.execute(&state);
            
    // render setup
    let renderer = renderer::Renderer::new(&state, renderer::SQUARE, texture).await;
  
    // main loop
    event_loop.run( move |event, _, control_flow| { 
        // control_flow.set_poll();
        control_flow.set_wait();

        match event {
            Event::WindowEvent { window_id, ref event }
            if window_id == window.id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                    WindowEvent::ScaleFactorChanged {  new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => { }
                }
            }
            Event::MainEventsCleared => {
                // app update code 
                window.request_redraw();
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                // Redraw
                state.update();

                match renderer.render(&state) {
                    Ok(_) => {}
                    // Reconfigure surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.raw_dimensions()),
                    // The system is out of memory -> quitting
                    Err(wgpu::SurfaceError::OutOfMemory) => control_flow.set_exit_with_code(1),
                    // Others 
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => { }
        }
    });
}
