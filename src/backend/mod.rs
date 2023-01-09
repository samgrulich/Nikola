pub mod binding;
pub use binding::*;
pub mod pipelines;
pub use pipelines::*;
pub mod shader;
pub use shader::*;
pub mod state;
pub use state::*;

pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm; 

#[derive(Copy, Clone, Debug)]
/// Specify 2D size (width, height)
pub struct Size<T> 
where T: num_traits::Unsigned
{
    pub width: T,
    pub height: T
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
