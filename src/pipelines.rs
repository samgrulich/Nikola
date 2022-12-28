use std::rc::Rc;
use crate::Shader;

use crate::state::*;
use crate::binding;
use crate::Size;


pub struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    pub fn new(data: [f32; 5]) -> Self {
        Vertex {
            position: data[0..3].try_into().unwrap(),
            uv: data[3..5].try_into().unwrap(),
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 3]> as wgpu::BufferAddress,
                    shader_location: 1,
                }
            ],
        }
    }
}

pub struct Rect {
    vertices: [Vertex; 4],
    indices: [i32; 6]
}

const RECT: Rect = Rect {
    vertices: [
        Vertex::new([ 1.,  1., 0., 1., 1.]),
        Vertex::new([-1., -1., 0., 0., 0.]),
        Vertex::new([ 1., -1., 0., 1., 0.]),
        Vertex::new([-1.,  1., 0., 0., 1.])
    ],
    indices: [
        0, 1, 2,
        0, 3, 1
    ],
};

pub struct RenderPipeline {
    texture: wgpu::Texture,
    sampler: wgpu::Sampler,

    pipeline: wgpu::RenderPipeline,
    state: Rc<StateData>,
}

impl RenderPipeline {
    pub fn new(state: &State, vertex: Shader, fragment: Shader) -> Self {
        let state = state.get_state();

        let texture = fragment.create_texture(
            state.size, 
            wgpu::TextureUsages::STORAGE_BINDING | 
            wgpu::TextureUsages::TEXTURE_BINDING, 
            binding::Access::Read, // todo implement multiple access types, use this for writing in
                                   // compute shader
            true
        );

        let layout = state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
            label: None, 
            bind_group_layouts: &[
                &fragment.get_layout().unwrap(),
            ], 
            push_constant_ranges: &[]
        });
        let pipeline = state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
            label: None, 
            layout: Some(&layout), 
            vertex: wgpu::VertexState { 
                module: &vertex.module, 
                entry_point: vertex.entry_point, 
                buffers: &[Vertex::desc()] // vertex description
            }, 
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None, 
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(
                wgpu::FragmentState { 
                    module: &fragment.module, 
                    entry_point: fragment.entry_point, 
                    targets: &[Some(
                        wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba8UnormSrgb, // todo FORMAT?
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }
                    )]
                },
            ),
            multiview: None,
        });

        todo!()
    }
}



pub struct ComputePipeline {
    pipeline: wgpu::ComputePipeline,
    size: Size<u32>
}

impl ComputePipeline {
    pub fn new() -> Self {
        todo!()
    }

    pub fn resize() { }
}
