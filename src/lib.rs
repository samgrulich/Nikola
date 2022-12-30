use std::{rc::Rc, borrow::Cow, fs};
use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

mod binding;
use crate::binding::*;
mod pipelines;
// use crate::pipelines::*;
mod state;
use crate::state::*;

pub async fn run() {
    todo!()
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

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm; 
pub type Entries = Vec<Box<dyn Resource>>;

/// Group all shader metadata with the module
///     use new method to create new
pub struct Shader {
    module: wgpu::ShaderModule,
    entry_point: &'static str,
    path: &'static str,
    visibility: Visibility,
    
    entries: Entries,

    bind_group: Option<wgpu::BindGroup>,
    layout: Option<wgpu::BindGroupLayout>,
    
    state: Rc<StateData>,
}

impl Shader {
    /// Create new shader object
    pub fn new(
        state: &State, 
        path: &'static str, 
        entry: &'static str, 
        visibility: Visibility, 
        entries: Entries,
    ) -> Self {
        let binding = fs::read_to_string(path).unwrap();
        let source = binding.as_str();
        let state = state.get_state();
        let module = state.device.create_shader_module(wgpu::ShaderModuleDescriptor { 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
        });

        let mut this = Shader {
            module,
            entry_point: entry,
            path,
            visibility,
            entries,
            state,
            bind_group: None,
            layout: None,
        };

        this.refresh_binding();
        this
    }

    pub fn add_entry(&mut self, entry: Box<dyn Resource>) {
        self.entries.push(entry);
        self.refresh_binding();
    }

    /// Create shader specific texture
    pub fn create_texture(&mut self, size: Size<u32>, usage: wgpu::TextureUsages, access: binding::Access, is_storage: bool) {
        let texture_data = self.state.device.create_texture(&wgpu::TextureDescriptor { 
            label: None,
            size: size.into_extent(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: Dimension::D2.to_texture(),
            format: FORMAT,
            usage,
        });

        let texture = binding::Texture::new(texture_data, access, is_storage);
        self.add_entry(Box::new(texture));
    }

    /// Create shader specific empty unmapped buffer
    pub fn create_buffer(&mut self, usage: wgpu::BufferUsages, size: u64, access: binding::Access) {
        let buffer_data = self.state.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size as wgpu::BufferAddress,
            usage,
            mapped_at_creation: false,
        });

        let buffer = binding::Buffer::new(buffer_data, access);
        self.add_entry(Box::new(buffer));
    }

    /// Create shader specific buffer with data in it
    pub fn create_buffer_init<T>(&mut self, usage: wgpu::BufferUsages, contents: &[T], access: binding::Access)
        where T: NoUninit
    {
        let buffer_data = self.state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(contents),
            usage
        });

        let buffer = binding::Buffer::new(buffer_data, access);
        self.add_entry(Box::new(buffer));
    }

    /// Create shader specific sampler (Linear filtering)
    pub fn create_sampler(&mut self) {
        let address_mode = wgpu::AddressMode::ClampToEdge;
        let filter_mode = wgpu::FilterMode::Linear;

        let sampler_data = self.state.device.create_sampler(&wgpu::SamplerDescriptor { 
            label: None, 
            address_mode_u: address_mode, 
            address_mode_v: address_mode, 
            address_mode_w: address_mode, 
            mag_filter: filter_mode, 
            min_filter: filter_mode, 
            mipmap_filter: filter_mode, 
            ..Default::default()
        });

        let sampler = binding::Sampler::new(sampler_data);
        self.add_entry(Box::new(sampler));
    }

    fn refresh_binding(&mut self) {
        let layouts = self.entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                entry.get_layout(index as u32, self.visibility)
            })
            .collect::<Vec<wgpu::BindGroupLayoutEntry>>();

        let layout = self.state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: None, 
            entries: layouts.as_slice(),
        });


        let resources = self.entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let binding = index as u32;
                
                wgpu::BindGroupEntry { 
                    binding, 
                    resource: entry.get_resource(),
                }
            })
            .collect::<Vec<wgpu::BindGroupEntry>>();

        let bind_group = self.state.device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: None, 
            layout: &layout, 
            entries: resources.as_slice() 
        });

        self.layout     = Some(layout);
        self.bind_group = Some(bind_group);
    }

    pub fn get_binding(&mut self) 
        -> (Option<&wgpu::BindGroup>, Option<&wgpu::BindGroupLayout>) {
        if let None = self.bind_group {
            self.refresh_binding();
        }

        (self.bind_group.as_ref(), self.layout.as_ref())
    }

    pub fn get_bind_group(&mut self) -> Option<&wgpu::BindGroup> {
        if let None = self.bind_group {
            self.refresh_binding();
        }

        self.bind_group.as_ref()
    }
    
    pub fn get_layout(&mut self) -> Option<&wgpu::BindGroupLayout> {
        if let None = self.layout { 
            self.refresh_binding();
        }
        
        self.layout.as_ref()
    }
}


#[derive(Copy, Clone)]
/// Specify 2D size (width, height)
pub struct Size<T> 
where T: num_traits::Unsigned
{
    width: T,
    height: T
}

impl<T> Size<T>
where T: num_traits::Unsigned + Copy, u32: From<T> 
{
    pub fn new(width: T, height: T) -> Self {
        Size { width, height }
    }

    pub fn from_physical(size: winit::dpi::PhysicalSize<T>) -> Self {
        Size { width: size.width, height: size.height }
    }

    pub fn from_tuple(tuple: (T, T)) -> Self {
        Size { width: tuple.0, height: tuple.1 }
    }

    pub fn into_tuple(&self) -> (T, T) {
        (self.width, self.height)
    }

    pub fn into_u32_tuple(&self) -> (u32, u32) {
        (self.width.into(), self.height.into())
    }

    pub fn into_extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d { 
            width: self.width.into(), 
            height: self.height.into(), 
            depth_or_array_layers: 1
        }
    }
    
    /// Compute how many times will the other fit into this size 
    /// ceiled to nearest integer
    pub fn fit_other(&self, other: Size<u32>) -> Size<u32> {
        let this = self.into_u32_tuple();
        let other = other.into_u32_tuple();

        let width  = (this.0 + other.0 - 1) / other.0;
        let height  = (this.1 + other.1 - 1) / other.1;

        Size { width, height }
    }
}

// swap chain object 
    
// most of the time pipelines:
// 1. I pipeline - I bind group
// 2. pipelines are either compute or render and follow simmilar pattern
// 3. one type visibilities are bound to one pipeline
//
// setup 
// execute
