use std::{rc::Rc, fs};
use std::io::{Error, ErrorKind};
use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

use crate::backend::binding::*;
use crate::backend::State;
use crate::backend::*;


pub type Entries = Vec<Box<dyn Resource>>;

/// Group all shader metadata with the module
///     use new method to create new
pub struct Shader {
    module: Rc<wgpu::ShaderModule>,
    pub entry_point: &'static str,
    pub path: &'static str,
    pub visibility: Visibility,
    
    entries: Entries,
    entry_layout: Option<Vec<usize>>,

    bind_group: Option<wgpu::BindGroup>,
    bind_layout: Option<wgpu::BindGroupLayout>,
    
    state: Rc<StateData>,
}

impl Shader {
    /// Create new shader object
    pub fn new(
        state: &State, 
        path: &'static str, 
        entry: &'static str, 
        visibility: Visibility, 
    ) -> Self {
        let binding = fs::read_to_string(path).unwrap();
        let source = binding.as_str();
        let state = state.get_state();
        let module = Rc::new(state.device.create_shader_module(wgpu::ShaderModuleDescriptor { 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(source.into()),
        }));

        Shader {
            module,
            entry_point: entry,
            path,
            visibility,
            entries: vec![],
            entry_layout: None,
            state,
            bind_group: None,
            bind_layout: None,
        }
    }

    pub fn new_from(
        &self, 
        entry: &'static str, 
        visibility: Visibility, 
    ) -> Self {
        Shader {
            module: self.module.clone(),
            entry_point: entry,
            path: self.path,
            visibility,
            entries: vec![],
            entry_layout: None,
            state: self.state.clone(),
            bind_group: None,
            bind_layout: None
        }
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

    /// Create a shader specific empty unmapped buffer with 
    /// STORAGE usage
    pub fn create_storage_buffer(&mut self, size: u64, access: binding::Access) {
        self.create_buffer(
            wgpu::BufferUsages::STORAGE,
            size, 
            access
        )
    }

    pub fn create_storage_buffer_init<T>(&mut self, contents: &[T], access: binding::Access)
        where T: NoUninit 
    {
        self.create_buffer_init(
            wgpu::BufferUsages::STORAGE, 
            contents, 
            access
        )
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

    /// Swap resources at two positions (positions are refferencing the index in constant 
    /// entries array)
    pub fn swap_resources(&mut self, first: usize, second: usize) -> Result<(), Error> {
        let first_type  = self.entries[first].get_type();
        let second_type = self.entries[second].get_type();

        if !first_type.equals(second_type) {
            return Err(Error::new(ErrorKind::Unsupported, "the two types aren't equal"));
        }

        if let Some(layout) = &mut self.entry_layout {
            let first_idx = layout
                .iter()
                .position(|elem| *elem == first)
                .unwrap();

            let second_idx = layout
                .iter()
                .position(|elem| *elem == second)
                .unwrap();

            layout[first_idx] = second;
            layout[second_idx] = first;
            self.entry_layout = Some(layout.to_vec());
        }
        else {
            let mut layout = (0..self.entries.len()).collect::<Vec<usize>>();
            layout[first] = second;
            layout[second] = first;
            self.entry_layout = Some(layout);
        }

        Ok(())
    }

    /// refresh the bind group layout and bind group of this shader
    pub fn refresh_binding(&mut self) {
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


        let entries = self.entries.iter();
        let resources = if self.entry_layout.is_none() {
                entries
                .enumerate()
                .map(|(index, entry)| {
                    let binding = index as u32;
                    
                    wgpu::BindGroupEntry { 
                        binding, 
                        resource: entry.get_resource(),
                    }
                })
                .collect::<Vec<wgpu::BindGroupEntry>>()
            } else {
                entries
                .zip(self.entry_layout.as_ref().unwrap().iter())
                .map(|(entry, index)| {
                    let binding = *index as u32;

                    wgpu::BindGroupEntry { 
                        binding, 
                        resource: entry.get_resource() 
                    }

                })
                .collect::<Vec<wgpu::BindGroupEntry>>()
            };

        let bind_group = self.state.device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: None, 
            layout: &layout, 
            entries: resources.as_slice() 
        });

        self.bind_layout = Some(layout);
        self.bind_group  = Some(bind_group);
    }

    pub fn get_binding(&mut self) 
        -> (Option<&wgpu::BindGroup>, Option<&wgpu::BindGroupLayout>) {
        if self.entries.len() == 0 {
            return (None, None);
        }
        
        if let None = self.bind_group {
            self.refresh_binding();
        }

        (self.bind_group.as_ref(), self.bind_layout.as_ref())
    }

    pub fn get_bind_group(&mut self) -> Option<&wgpu::BindGroup> {
        if self.entries.len() == 0 {
            return None;
        }

        if let None = self.bind_group {
            self.refresh_binding();
        }

        self.bind_group.as_ref()
    }
    
    pub fn get_layout(&mut self) -> Option<&wgpu::BindGroupLayout> {
        if self.entries.len() == 0 {
            return None;
        }

        if let None = self.bind_layout { 
            self.refresh_binding();
        }
        
        self.bind_layout.as_ref()
    }

    pub fn get_module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
}
