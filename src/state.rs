use std::rc::Rc;

use crate::binding;
use crate::binding::Visibility;
use crate::Entries;
use crate::Shader;

pub struct StateData {
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub struct State {
    state: Rc<StateData>,
}

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm; 

/// init wgpu surface with my default values
fn config_surface(
    surface: &wgpu::Surface, 
    device: &wgpu::Device, 
    size: winit::dpi::PhysicalSize<u32>
) {
        surface.configure(device, &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: FORMAT, // could request supported from adapter
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        })
}

impl StateData {
    /// Initialize new backend logic devices
    pub async fn new(window: &winit::window::Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor { 
                    label: Some("main device"),
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None
             )
            .await
            .unwrap();
         
        config_surface(&surface, &device, window.inner_size());
        
        StateData { 
            surface,
            adapter,
            device,
            queue,
        }
    }

    /// Resize window surface to the new size
    pub fn resize(&self, size: winit::dpi::PhysicalSize<u32>) {
        config_surface(&self.surface, &self.device, size);
    }
}


impl State {
    pub fn new(state: StateData) -> Self {
        let state = Rc::new(state);
        
        State {
            state
        }
    }

    /// Resize window surface to the new size
    pub fn resize(&self, size: winit::dpi::PhysicalSize<u32>) {
        self.state.resize(size)
    }

    /// Create new shader 
    pub fn create_shader(
        &self,
        path: &str,
        entry: &str,
        visibility: Visibility,
        entries: Entries
    ) -> Shader {
        Shader::new(
            &self, 
            path, 
            entry, 
            visibility, 
            entries
        )
    }

    pub fn create_texture(&self) -> binding::Texture {
        todo!()
    }

    pub fn create_sampler(&self) -> binding::Sampler {
        todo!()
    }

    pub fn create_buffer(&self) -> binding::Buffer {
        todo!()
    }

    pub fn get_state(&self) -> Rc<StateData> {
        self.state.clone()
    }
}
