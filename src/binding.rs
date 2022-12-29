use std::rc::Rc;

use crate::backend::FORMAT;

pub enum Access {
    Write,
    Read,
    Both,
}

impl Access {
    /// get wgpu equivalent of contents
    pub fn to_wgpu(&self) -> wgpu::StorageTextureAccess {
        match *self {
            Access::Write => wgpu::StorageTextureAccess::WriteOnly,
            Access::Read  => wgpu::StorageTextureAccess::ReadOnly,
            Access::Both  => wgpu::StorageTextureAccess::ReadWrite,
        }
    }
     
    /// get boolean which states if the access is read_only
    pub fn to_bool_read(&self) -> bool {
        match *self {
            Access::Read => true,
            _ => false,
        }
    }
}

/// Describe how many dimensions is your array structured in
pub enum Dimension {
    D1,
    D2,
    D3
}

impl Dimension {
    /// get wgpu equivalent for texture
    pub fn to_texture(&self) -> wgpu::TextureDimension {
        match *self {
            Dimension::D1 => wgpu::TextureDimension::D1,
            Dimension::D2 => wgpu::TextureDimension::D2,
            Dimension::D3 => wgpu::TextureDimension::D3,
        }
    }

    /// get wgpu equivalent for texture view
    pub fn to_view(&self) -> wgpu::TextureViewDimension {
        match *self {
            Dimension::D1 => wgpu::TextureViewDimension::D1,
            Dimension::D2 => wgpu::TextureViewDimension::D2,
            Dimension::D3 => wgpu::TextureViewDimension::D3,
        }
    }
}

/// Describe what shader stage is able to access this data
pub enum Visibility {
    VERTEX,
    FRAGMENT,
    COMPUTE,
}

impl Visibility {
    /// get the wgpu equivalent
    pub fn to_wgpu(&self) -> wgpu::ShaderStages {
        match *self {
            Visibility::VERTEX   => wgpu::ShaderStages::VERTEX,
            Visibility::FRAGMENT => wgpu::ShaderStages::FRAGMENT,
            Visibility::COMPUTE  => wgpu::ShaderStages::COMPUTE,
        }
    }
}

pub trait Resource {
    /// get bind group layout entry of this resource
    fn get_layout(&self, binding: u32, visibility: Visibility) -> wgpu::BindGroupLayoutEntry;

    /// get binding resource of this resource
    fn get_resource(&self, binding: u32) -> wgpu::BindingResource;
}

fn get_layout_entry(binding: u32, visibility: Visibility, ty: wgpu::BindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry { 
        binding,
        visibility: visibility.to_wgpu(), 
        ty, 
        count: None
    }
}



/// Contains texture and additional data
pub struct Texture {
    texture: Box<Rc<wgpu::Texture>>,
    access: Access,
    dimension: Dimension,
    is_storage: bool,
}

impl Texture {
    pub fn new(texture: wgpu::Texture, access: Access, is_storage: bool) -> Self {
        let texture = Box::new(Rc::new(texture));
        Texture { 
            texture, 
            access, 
            dimension: Dimension::D2, 
            is_storage,
        }
    }

    /// Get separate view of this texture data, and you can specify texture access data 
    pub fn get_view(&self, data: Option<(Access, Dimension, bool)>) -> Texture {
        let data = data.unwrap_or((self.access, self.dimension, self.is_storage));

        Texture { 
            texture: self.texture.clone(), 
            access: data.0, 
            dimension: data.1,
            is_storage: data.2,
        }
    }
}

impl Resource for Texture {
    fn get_layout(&self, binding: u32, visibility: Visibility) -> wgpu::BindGroupLayoutEntry {
        let ty = if self.is_storage {
                wgpu::BindingType::StorageTexture { 
                    access: self.access.to_wgpu(), 
                    format: FORMAT, 
                    view_dimension: self.dimension.to_view(),
                }
            } else {
                wgpu::BindingType::Texture { 
                    sample_type: wgpu::TextureSampleType::Float { filterable: true }, // todo: parametrize
                    view_dimension: self.dimension.to_view(), 
                    multisampled: false
                }
            };

        get_layout_entry(binding, visibility, ty)
    }
    
    fn get_resource(&self, binding: u32) -> wgpu::BindingResource {
        let view = self.texture.create_view(&wgpu::TextureViewDescriptor::default());
        wgpu::BindingResource::TextureView(&view) // possible early free of memory
    }
}



/// Trait that signifies the data is referencing buffer
pub trait BufferData {
    fn get_binding(&self) -> wgpu::BufferBinding;
}

impl BufferData for wgpu::Buffer {
    fn get_binding(&self) -> wgpu::BufferBinding {
        self.as_entire_buffer_binding()
    }
}

impl<'a> BufferData for wgpu::BufferBinding<'a> {
    fn get_binding(&self) -> wgpu::BufferBinding {
        self.clone()
    }
}

pub struct Buffer {
    buffer: Box<dyn BufferData>,
    access: Access,
}

impl Buffer {
    pub fn new(buffer: wgpu::Buffer, access: Access) -> Self {
        let buffer = Box::new(buffer);

        Buffer { buffer, access }
    }

    /// Get buffer binding of this buffer data and specify additional access data
    pub fn get_binding(&self, data: Option<(Access,)>) -> Buffer {
        let binding = Box::new(self.buffer.get_binding()); 
        let data = data.unwrap_or((self.access, ));

        Buffer { buffer: binding, access: data.0}
    }
}

impl Resource for Buffer {
    fn get_layout(&self, binding: u32, visibility: Visibility) -> wgpu::BindGroupLayoutEntry {
        let ty =  wgpu::BindingType::Buffer { 
            ty: wgpu::BufferBindingType::Storage { read_only: self.access.to_bool_read() }, 
            has_dynamic_offset: false, 
            min_binding_size: None 
        };

        get_layout_entry(binding, visibility, ty)
    }

    fn get_resource(&self, binding: u32) -> wgpu::BindingResource {
        wgpu::BindingResource::Buffer(self.buffer.get_binding())
    }
}



pub struct Sampler {
    sampler: wgpu::Sampler,
    access: Access,
}

impl Sampler {
    pub fn new(sampler: wgpu::Sampler, access: Access) -> Self {
        Sampler { sampler, access }
    }
}

impl Resource for Sampler {
    fn get_layout(&self, binding: u32, visibility: Visibility) -> wgpu::BindGroupLayoutEntry {
        let ty = wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering);
        get_layout_entry(binding, visibility, ty)
    }

    fn get_resource(&self, binding: u32) -> wgpu::BindingResource {
        wgpu::BindingResource::Sampler(&self.sampler)
    }
}
