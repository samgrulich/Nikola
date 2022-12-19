use crate::general::{State, PhysicalSize};
use std::fs;



pub fn compute_work_group_count(
    (width, height): (u32, u32),
    (workgroup_width, workgroup_height): (u32, u32),
) -> (u32, u32) {
    let x = (width + workgroup_width - 1) / workgroup_width;
    let y = (height + workgroup_height - 1) / workgroup_height;

    (x, y)
}


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
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn from_size(size: PhysicalSize<u32>) -> Self {
        Dimensions { width: size.width, height: size.height }
    }

    pub fn as_tuple(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}


pub struct ComputeUnit {
    pipeline: wgpu::ComputePipeline,
    dimensions: Dimensions,
    bind_group: wgpu::BindGroup,
}

impl ComputeUnit {
    /// Create new compute unit
    pub async fn new<'a>(
        state: &State, 
        dimensions: Dimensions, 
        shader: Shader,
        bg_resources: Vec<wgpu::BindingResource<'a>>,
    ) -> Self {
        let pipeline = state.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None, 
            module: &shader.get_module(&state.device),
            entry_point: "main",
        });

        let entries = bg_resources
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

        ComputeUnit { pipeline, dimensions, bind_group }
    }

    pub fn execute(&self, state: &State) {
        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let (dispatch_width, dispatch_height) = compute_work_group_count(self.dimensions.as_tuple(), (8, 8));
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
            
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(dispatch_width, dispatch_height, 1);
        }
        
        state.queue.submit(Some(encoder.finish()));
    }
}

