pub use winit::{dpi::PhysicalSize, window::Window};

pub struct State {
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    dimensions: PhysicalSize<u32>,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let dimensions = window.inner_size();
        let PhysicalSize{width, height} = dimensions;

        // adapters (devices) setup
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe {instance.create_surface(window)};
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to load adapter");

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::default(),
            limits: wgpu::Limits::downlevel_defaults(),
        }, None)
        .await
        .expect("Failed to load queu and logical device");

        // surface config
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        State { 
            surface, 
            adapter, 
            device, 
            queue, 
            dimensions, 
            config, 
        }
    }

    pub fn raw_dimensions(&self) -> PhysicalSize<u32> {
        self.dimensions
    }

    pub fn resize(&mut self, new_dims: PhysicalSize<u32>) {
        if new_dims.width > 0 && new_dims.height > 0 {
            self.dimensions = new_dims;
            self.config.width = new_dims.width;
            self.config.height = new_dims.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, _event: &winit::event::WindowEvent) -> bool {
        false 
    }

    pub fn update(&mut self) {
        // remove `todo!()`
    }
}

