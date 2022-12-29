use std::rc::Rc;
use wgpu::util::DeviceExt;

use crate::Shader;

use crate::state::*;
use crate::binding;
use crate::Size;


#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

unsafe impl bytemuck::Zeroable for Vertex { }
unsafe impl bytemuck::Pod for Vertex { }

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
    texture: binding::Texture,
    vertex: Shader,
    fragment: Shader,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    pipeline: wgpu::RenderPipeline,
    state: Rc<StateData>,
}

impl RenderPipeline {
    pub fn new(state: &State, vertex: Shader, fragment: Shader) -> Self {
        // setup the inputs
        let texture = state.create_texture(
            state.size, 
            wgpu::TextureUsages::STORAGE_BINDING | 
            wgpu::TextureUsages::TEXTURE_BINDING, 
            binding::Access::Read,
            true
        );

        let sampler = fragment.create_sampler();

        let vertex_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(RECT.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(RECT.indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        let state = state.get_state();

        // bind the generic inputs
        fragment.add_entry(Box::new(texture.get_view(None)));

        // setup the pipeline 
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

        RenderPipeline { texture, vertex, fragment, vertex_buffer, index_buffer, pipeline, state }
    }

    /// Creates render pass with instructions to clear display in place
    fn begin_render_pass<'a>(encoder: &'a mut wgpu::CommandEncoder, view: &'a wgpu::TextureView) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
            label: Some("Render pipeline pass"), 
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment { 
                    view, 
                    resolve_target: None, 
                    ops: wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true, 
                    } 
                }),
            ], 
            depth_stencil_attachment: None,
        })
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> { 
        let output = self.state.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render pipeline command encoder"),
        });

        {
            let mut render_pass = RenderPipeline::begin_render_pass(&mut encoder, &view);

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, 0..2);
        }

        Ok(())
    }
}



pub struct ComputePipeline {
    pipeline: wgpu::ComputePipeline,

    shader: Shader,

    size: Size<u32>,
    size_z: Option<u32>,
    state: Rc<StateData>
}

impl ComputePipeline {
    pub fn new(shader: Shader) -> Self {
        todo!()
    }

    pub fn resize() { }

    pub fn execute() { }
}
