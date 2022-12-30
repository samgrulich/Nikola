use window::init_window;

mod backend;
pub use crate::backend::*;

pub async fn run() {
    let (event_loop, window) = init_window();
    
    let state = State::new(&window).await;

    let vertex = Shader::new(&state, "../res/shaders/screen_shader.wgsl", "vert_main", Visibility::VERTEX, vec![]);
    let fragment = Shader::new(&state, "../res/shaders/screen_shader.wgsl", "frag_main", Visibility::FRAGMENT, vec![]);
    let render_pipeline = RenderPipeline::new(&state, vertex, fragment);

    event_loop.run(move |event, _, control_flow| {
        

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

// execute
