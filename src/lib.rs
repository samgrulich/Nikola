use window::init_window;
use winit::{event::Event, event::WindowEvent};

mod backend;
pub use crate::backend::*;

pub async fn run() {
    let (event_loop, window) = init_window();
    
    let state = State::new(&window).await;

    // shader setup
    let vertex = Shader::new(&state, "./res/shaders/screen_shader.wgsl", "vert_main", Visibility::VERTEX);
    let fragment = vertex.new_from("frag_main", binding::Visibility::FRAGMENT);

    // renderer setup 
    let mut render_pipeline = RenderPipeline::new(&state, vertex, fragment);

    // compute setup
    let mut shader = Shader::new(&state, "./res/shaders/render_shader.wgsl", "main", Visibility::COMPUTE);

    let particles = create_particle_list(4);
    let compute_texture = render_pipeline.get_texture(binding::Access::Write, true);

    shader.add_entry(Box::new(compute_texture));
    shader.create_buffer_init(wgpu::BufferUsages::STORAGE, particles.as_slice(), binding::Access::Read);

    let mut compute = ComputePipeline::new(&state, shader, Size::from_physical(window.inner_size()), Some(Size::new(1, 1)));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { window_id, event } 
                if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => control_flow.set_exit(),
                            // todo: implement shader resizing (down)
                        WindowEvent::Resized( new_size ) => state.resize(new_size), 
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => state.resize(*new_inner_size),
                        _ => {}
                    }
            },
            Event::MainEventsCleared => {
                // update app
                compute.execute();

                window.request_redraw();
            },
            Event::RedrawRequested(
                window_id 
            ) if window_id == window.id() => {
                let render_result = render_pipeline.render();

                match render_result {
                    Ok(_) => {},
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(window.inner_size()),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => control_flow.set_exit(),
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        }
    });
}

pub mod window {
    pub fn init_window() -> (winit::event_loop::EventLoop<()>, winit::window::Window) {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title("Nikola - prototype")
            .build(&event_loop)
            .unwrap();

        (event_loop, window)
    }
}

fn create_particle_list(count: u32) -> Vec<[f32; 4]> {
    let mut particles = vec![];

    for y in 0..count {
        for x in 0..count {
            particles.push([
               x as f32, 
               y as f32,
               0f32,
               0f32,
            ]);
        }
    }

    particles
}
