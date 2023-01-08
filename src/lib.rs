use std::rc::Rc;
use window::init_window;
use winit::{event::Event, event::WindowEvent};
use std::time;

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
    let compute_texture = render_pipeline.get_texture(binding::Access::Write, true);

    shader.add_entry(Box::new(compute_texture));

    // fluid setup
    let fluid_shader = Shader::new(&state, "./res/shaders/fluid_shader.wgsl", "main", Visibility::COMPUTE);
    let mut water = Fluid::new(&state, fluid_shader, Size::new(6, 5));

    shader.add_entry(Box::new(water.particles_in.get_binding(Some((Access::Read, )))));
    
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
                water.update();

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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    position: [f32; 2],
    velocity: [f32; 2],
    mass: f32,
    density: f32,
}

impl Particle {
    pub fn new(x: f32, y: f32) -> Self {
        Particle { 
            position: [x, y], 
            velocity: [0f32, 0f32],
            mass: 10f32,
            density: 10f32,
        }
    }
}

pub struct Fluid {
    computer: ComputePipeline,
    state: Rc<StateData>,

    particles_in: Buffer,
    particles_out: Buffer,
    particles_size: wgpu::BufferAddress,

    last_time: time::Instant,
    time_step: Buffer,
}

impl Fluid {
    fn create_particles(size: Size<u32>) -> Vec<Particle> {
        let mut particles = vec![];

        for y in 0..size.height {
            for x in 0..size.width {
                if y < 2 || (y >= 2 && x < 3) {
                    let particle = Particle::new(
                        (x) as f32, 
                        (y + 1) as f32
                    );
                    particles.push(particle)
                }
            }
        }

        particles
    }

    pub fn new(state: &State, mut shader: Shader, size: Size<u32>) -> Self {
        let start_time = time::Instant::now();
        let particles = Self::create_particles(size);
        let particles_size = std::mem::size_of_val(particles.as_slice()) as u64;
        let rest_density = 10f32;

        let particles_in = state.create_buffer_init(
            particles.as_slice(), 
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, 
            Access::Both
        );
        let particles_out = state.create_buffer(
            particles_size, 
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            Access::Write
        );
        let time_step = state.create_buffer_init(
            &[0f32], 
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,  
            Access::Read
        );

        shader.add_entry(Box::new(particles_in.get_binding(None)));
        shader.add_entry(Box::new(particles_out.get_binding(None)));
        shader.add_entry(Box::new(time_step.get_binding(None)));
        shader.create_storage_buffer_init(
            &[rest_density], 
            Access::Read
        );
        shader.refresh_binding();

        let computer = ComputePipeline::new(state, shader, size, None);
        let state = state.get_state();
        Fluid { 
            computer, 
            state, 
            particles_in, 
            particles_out, 
            particles_size, 
            time_step,
            last_time: start_time
        }
    }

    /// Update the state of fluid (run the shader)
    pub fn update(&mut self) {
        let mut encoder = self.computer.start_execute();
        encoder.copy_buffer_to_buffer(&self.particles_out, 0, &self.particles_in, 0, self.particles_size);

        let time_step = self.last_time.elapsed().as_secs_f32();
        let instance  = time::Instant::now();
        self.state.queue.write_buffer(&self.time_step, 0, bytemuck::cast_slice(&[time_step]));

        self.last_time = instance;
        self.state.queue.submit(std::iter::once(encoder.finish()));
    }
}
