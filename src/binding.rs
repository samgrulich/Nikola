use std::{rc::Rc, ops::Deref};

use crate::FORMAT;

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
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
    fn get_resource(&self) -> wgpu::BindingResource;
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
    texture: Rc<wgpu::Texture>,
    view: wgpu::TextureView,

    access: Access,
    dimension: Dimension,
    is_storage: bool,
}

impl Texture {
    pub fn new(texture: wgpu::Texture, access: Access, is_storage: bool) -> Self {
        let texture = Rc::new(texture);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture { 
            texture, 
            view,
            access, 
            dimension: Dimension::D2, 
            is_storage,
        }
    }

    /// Swap texture
    pub unsafe fn swap_texture(&mut self, mut new_texture: wgpu::Texture) {
        let texture_ptr: *const wgpu::Texture = &*self.texture;
        let texture_ptr = texture_ptr.cast_mut();
        
        let new_texture_ptr: *mut wgpu::Texture = &mut new_texture;
        
        // swap the textures
        texture_ptr.swap(new_texture_ptr);

        // clean the old texture
        new_texture_ptr.drop_in_place();
    }

    /// Get separate view of this texture data, and you can specify texture access data 
    pub fn get_view(&self, data: Option<(Access, Dimension, bool)>) -> Texture {
        let data = data.unwrap_or((self.access, self.dimension, self.is_storage));
        let view = self.texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture { 
            texture: self.texture.clone(), 
            view,
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
    
    fn get_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(&self.view) 
    }
}



/// Trait that signifies the data is referencing buffer
pub struct Buffer {
    buffer: Rc<wgpu::Buffer>,
    access: Access,
}

impl Deref for Buffer {
    type Target = Rc<wgpu::Buffer>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl Buffer {
    pub fn new(buffer: wgpu::Buffer, access: Access) -> Self {
        let buffer = Rc::new(buffer);

        Buffer { buffer, access }
    }

    /// Get buffer binding of this buffer data and specify additional access data
    pub fn get_binding(&self, data: Option<(Access,)>) -> Buffer {
        let binding = self.buffer.clone(); 
        let data = data.unwrap_or((self.access, ));

        Buffer { buffer: binding, access: data.0 }
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

    fn get_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::Buffer(self.buffer.as_entire_buffer_binding())
    }
}



pub struct Sampler {
    sampler: wgpu::Sampler,
}

impl Deref for Sampler {
    type Target = wgpu::Sampler;

    fn deref(&self) -> &Self::Target {
        &self.sampler
    }
}

impl Sampler {
    pub fn new(sampler: wgpu::Sampler) -> Self {
        Sampler { sampler }
    }
}

impl Resource for Sampler {
    fn get_layout(&self, binding: u32, visibility: Visibility) -> wgpu::BindGroupLayoutEntry {
        let ty = wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering);
        get_layout_entry(binding, visibility, ty)
    }

    fn get_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::Sampler(&self.sampler)
    }
}
