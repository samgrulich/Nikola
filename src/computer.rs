use crate::general::{State, PhysicalSize};
use wgpu::util::DeviceExt;
use std::fs;

// ULTIMATE GOAL: upload data (structs) onto GPU -> Compute some algorithms on the GPU -> Retrive
// the data from the GPU - either as a buffer or a texture :)
//
// Suggested solution: 
//  Optional - Rewrite the old compute shader code from glsl -> wgsl

/// Shader data 
///  -- path: String 
///  -- entry_point: String
pub struct Shader {
    pub path: String, 
    pub entry_point: String,
}

impl Shader { 
    pub fn get_module(&self, device: &wgpu::Device) -> wgpu::ShaderModule {
        let path = &self.path;
        let src = fs::read_to_string(path).unwrap();

        device.create_shader_module(wgpu::ShaderModuleDescriptor { 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(src.into())
        }) 
    }
}

// move this into the general module
pub struct Dimensions {
    width: u32,
    height: u32,
}

impl Dimensions {
    pub fn from_size(size: PhysicalSize<u32>) -> Self {
        Dimensions { width: size.width, height: size.height }
    }

    pub fn as_tuple(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

pub struct Computer {
    out_texture: wgpu::Texture,
    pipeline: wgpu::ComputePipeline,
    dimensions: Dimensions,
    bind_group: wgpu::BindGroup,
}

impl Computer {
    pub async fn new<'a>(
        state: &State, 
        dimensions: Dimensions, 
        shader: Shader,
        bg_resources: Vec<wgpu::BindingResource<'a>>,
    ) -> Computer {
        let Dimensions{width, height} = dimensions;
        let out_texture = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Compute out texture"),
            size: wgpu::Extent3d{width, height, depth_or_array_layers: 1},
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let pipeline = state.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None, 
            module: &shader.get_module(&state.device),
            entry_point: "main",
        });

        let texture_view = out_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut resources = bg_resources.as_slice().to_vec();
        
        resources
            .push(
                wgpu::BindingResource::TextureView(&texture_view)
            ); 
        resources.rotate_right(1); // move texture to the 1st place

        let entries = resources
            .iter()
            .enumerate()
            .map(|(index, resource)| {
                wgpu::BindGroupEntry {
                    binding: index as u32,
                    resource: resource.to_owned(),
                }
            });
        let entries = entries.collect::<Vec<_>>();

        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: entries.as_slice(),
        });

        Computer { out_texture, pipeline, dimensions, bind_group }
    }

    pub fn execute(&self, state: &State) -> &wgpu::Texture {
        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let (dispatch_width, dispatch_height) = compute_work_group_count(self.dimensions.as_tuple(), (8, 8));
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
            
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(dispatch_width, dispatch_height, 1);
        }
        
        state.queue.submit(Some(encoder.finish()));
        // return the texture for render loading
        &self.out_texture
    }
}

pub fn compute_work_group_count(
    (width, height): (u32, u32),
    (workgroup_width, workgroup_height): (u32, u32),
) -> (u32, u32) {
    let x = (width + workgroup_width - 1) / workgroup_width;
    let y = (height + workgroup_height - 1) / workgroup_height;

    (x, y)
}

pub async fn compute_config(state: &State, dimensions: PhysicalSize<u32>) -> Computer {
    let color = [0.9, 0.8, 0.2, 0.8];
    let color_buff = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
        label: Some("Color write buffer"),
        contents: bytemuck::cast_slice(color.as_slice()),
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
    });

    let computer = Computer::new(
        state,
        Dimensions::from_size(dimensions), 
        Shader { path: "./res/shaders/compute.wgsl".to_string(), entry_point: "main".to_string() }, 
        vec![color_buff.as_entire_binding()]
    ).await;

    computer
}

// pub async fn compute_config(state: &State) -> wgpu::Texture {
//     let PhysicalSize { width, height } = state.raw_dimensions();
//
//     // render setup  
//     let color = [0.9, 0.8, 0.2, 0.8];
//     let color_buff = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
//         label: Some("Color write buffer"),
//         contents: bytemuck::cast_slice(color.as_slice()),
//         usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
//     });
//
//     let out_texture = state.device.create_texture(&wgpu::TextureDescriptor {
//         label: Some("Compute out texture"),
//         size: wgpu::Extent3d{width, height, depth_or_array_layers: 1},
//         mip_level_count: 1,
//         sample_count: 1,
//         dimension: wgpu::TextureDimension::D2,
//         format: wgpu::TextureFormat::Rgba8Unorm,
//         usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
//     });
//
//     let shader = state.device.create_shader_module(wgpu::ShaderModuleDescriptor { 
//         label: None,
//         source: wgpu::ShaderSource::Wgsl(include_str!("../res/shaders/compute.wgsl").into()),
//     });
//     
//     let pipeline = state.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
//         label: None,
//         layout: None, 
//         module: &shader,
//         entry_point: "main",
//     });
//
//     let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor { 
//         label: None,
//         layout: &pipeline.get_bind_group_layout(0),
//         entries: &[
//             wgpu::BindGroupEntry {
//                 binding: 0,
//                 resource: color_buff.as_entire_binding(),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 1,
//                 resource: wgpu::BindingResource::TextureView(&out_texture.create_view(&wgpu::TextureViewDescriptor::default())),
//             }
//         ],
//     });
//
//     // the data transfer + execution
//     let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
//     {
//         let (dispatch_width, dispatch_height) = compute_work_group_count((width, height), (8, 8));
//         let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
//         
//         compute_pass.set_pipeline(&pipeline);
//         compute_pass.set_bind_group(0, &bind_group, &[]);
//         compute_pass.dispatch_workgroups(dispatch_width, dispatch_height, 1);
//     }
//     
//     state.queue.submit(Some(encoder.finish()));
//     // return the texture for render loading
//     out_texture
// }
//
