use wgpu::util::DeviceExt;

// ULMITMATE GOAL: render a texture onto screen
//
// suggested solution:
//  add texture parameter to the fragment shader (mby vertex too, for the UV coordinations)
//  aand load a picture into the texture

use crate::general::State;


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1, 
                    format: wgpu::VertexFormat::Float32x2,
                },
            ]
        }
    }
}

unsafe impl bytemuck::Zeroable for Vertex { } 
unsafe impl bytemuck::Pod for Vertex { }


pub struct Renderer{
    vertex_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    num_vertices: u32,
    bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub async fn new(state: &State, mesh_slice: &[Vertex], texture: &wgpu::Texture) -> Self {
        let vertex_buffer = setup_vertex_buffer(&state.device, mesh_slice);
        let num_vertices = mesh_slice.len() as u32;
        let (bind_group, texture_layout) = bind_from_texture(texture, &state);
        let render_pipeline = setup_render_pipeline(state, &texture_layout);

        Renderer {
            vertex_buffer,
            render_pipeline,
            num_vertices,
            bind_group,
        }
    }

    pub fn render(&self, state: &State) -> Result<(), wgpu::SurfaceError>{
        // load the window data
        let output = state.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // encode the commands and send them to the GPU
        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { 
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
                label: Some("Render Pass"), 
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.3,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }
                            ),
                            store: true,
                        },
                    })
                ],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }

        state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}


pub const SQUARE: &[Vertex] = &[
    Vertex { position: [-1.0,  1.0, 0.0], uv: [0., 1.] },
    Vertex { position: [-1.0, -1.0, 0.0], uv: [0., 0.] },
    Vertex { position: [ 1.0, -1.0, 0.0], uv: [1., 0.] },

    Vertex { position: [-1.0,  1.0, 0.0], uv: [0., 1.] },
    Vertex { position: [ 1.0, -1.0, 0.0], uv: [1., 0.] },
    Vertex { position: [ 1.0,  1.0, 0.0], uv: [1., 1.] },
];

/// Create vertex buffer from vertex array
pub fn setup_vertex_buffer(device: &wgpu::Device, mesh_slice: &[Vertex]) -> wgpu::Buffer {
    let vertex_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(mesh_slice),
            usage: wgpu::BufferUsages::VERTEX
        }
    );

    vertex_buffer
}

/// Returns configured render pipeline, with basic shader inplace and 
/// configured to use triangle list
pub fn setup_render_pipeline(State {device, config, ..}: &State, texture_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { 
        label: Some("Vertex(render) shader"), 
        source: wgpu::ShaderSource::Wgsl(include_str!("../res/shaders/shader.wgsl").into()),
    });

    let render_pipeline_layout = 
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[texture_bind_group_layout],
            push_constant_ranges: &[],
        });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                Vertex::desc(),
            ],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[
                Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }
            )],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1, 
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    render_pipeline
}

/// Get bind group and bind layout from texture
pub fn bind_from_texture(texture: &wgpu::Texture, state: &State) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    // define sampler & view
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let texture_sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
   
    // configure how to pass them to the pipeline
    let texture_bind_group_layout = state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { 
                    sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                    view_dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false, 
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            }
        ],
        label: Some("texture bind group layout"),
    });

    let texture_bind_group = state.device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                }
            ],
            label: Some("texture bind group"),
        }
    );

    (texture_bind_group, texture_bind_group_layout)
}
