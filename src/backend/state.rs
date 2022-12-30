use std::ops::Deref;
use std::rc::Rc;

use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

use crate::backend::{FORMAT, Size, binding::{self, Visibility}, Entries, Shader};

pub struct StateData {
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub size: Size<u32>,
}

pub struct State {
    state: Rc<StateData>,
}


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
            size: Size::from_physical(window.inner_size())
        }
    }

    /// Resize window surface to the new size
    pub fn resize(&self, size: winit::dpi::PhysicalSize<u32>) {
        config_surface(&self.surface, &self.device, size);
    }

    /// Create new raw texture with my custom default params
    pub fn create_raw_texture(
        &self,
        size: Size<u32>,
        usage: wgpu::TextureUsages,
    ) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: size.into_extent(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: binding::Dimension::D2.to_texture(),
            format: FORMAT,
            usage,
        })
    }
}

impl Deref for State {
    type Target = StateData;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl State {
    pub async fn new(window: &winit::window::Window) -> Self {
        let state = StateData::new(window).await;
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
        path: &'static str,
        entry: &'static str,
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

    pub fn get_state(&self) -> Rc<StateData> {
        self.state.clone()
    }

    /// Create generic texture 
    pub fn create_texture(
        &self, 
        size: Size<u32>, 
        usage: wgpu::TextureUsages,
        access: binding::Access,
        is_storage: bool,
    ) -> binding::Texture {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor{
            label: None,
            size: size.into_extent(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: binding::Dimension::D2.to_texture(),
            format: FORMAT,
            usage,
        });

        binding::Texture::new(texture, access, is_storage)
    }

    /// Create new empty unmapped buffer
    pub fn create_buffer(
        &self, 
         size: u64, 
         usage: wgpu::BufferUsages,
         access: binding::Access,
    ) -> binding::Buffer {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor{
            label: None,
            size: size as wgpu::BufferAddress,
            usage,
            mapped_at_creation: false,
        });

        binding::Buffer::new(buffer, access)
    }

    /// Create new buffer initialized with data
    pub fn create_buffer_init<T>(
        &self, 
        contents: &[T], 
        usage: wgpu::BufferUsages,
        access: binding::Access,
    )-> binding::Buffer 
        where T: NoUninit 
    {
        let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(contents),
            usage,
        });

        binding::Buffer::new(buffer, access)
    }
}
