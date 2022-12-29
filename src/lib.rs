use std::rc::Rc;
use std::borrow::Cow;
use std::fs;
use backend::FORMAT;
use backend::render_texture;
use bytemuck::NoUninit;
use num_traits::Float;
use wgpu::util::DeviceExt;
use winit::event::{WindowEvent, Event};

mod binding;
use crate::binding::*;
mod pipelines;
//use crate::pipelines::*;
mod state;
use crate::state::*;

pub async fn run() {
    let (event_loop, window) = window::init_window();
    let (surface, device, queue) = backend::init_backend(&window).await;

    let (vertex_buffer, index_buffer) = backend::initialize_quad(&device);

    let (out_texture, compute_bind_group, compute_bind_group_layout, particles) = backend::initialize_compute_data(&device, window.inner_size());
    let (fluid_bind_group, fluid_bind_group_layout) = backend::init_fluid_data(&device, &particles);
    let (texture_bind_group, texture_bind_group_layout) = backend::get_texure_render_data(&device, &out_texture.create_view(&wgpu::TextureViewDescriptor::default()));
    
    // computes init
    let compute_shader = device.create_shader_module(include_wgsl!("../res/shaders/render_shader.wgsl"));
    let compute_pipeline = backend::init_compute_unit(&device, &compute_shader, &compute_bind_group_layout);

    let fluid_shader = device.create_shader_module(include_wgsl!("../res/shaders/fluid_shader.wgsl"));
    let fluid_pipeline = backend::init_compute_unit(&device, &fluid_shader, &fluid_bind_group_layout);

    // render init
    let screen_shader = device.create_shader_module(include_wgsl!("../res/shaders/screen_shader.wgsl"));
    let screen_pipeline = backend::init_texture_render_pipeline(&device, &screen_shader, &texture_bind_group_layout);

    // rendering should be separated from event handling
    event_loop.run(move |event, _, control_flow| {
        // handle input here 
        match event {
            // handling of window events
            Event::WindowEvent { 
                window_id, 
                event
            } if window_id == window.id() => { 
                match event {
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    WindowEvent::Resized( new_size ) => backend::resize_surface(&surface, &device, new_size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => backend::resize_surface(&surface, &device, *new_inner_size),
                    _ => {},
                }
            }
            Event::MainEventsCleared => {
                backend::execute_compute_unit(&device, &queue, &fluid_pipeline, &fluid_bind_group, winit::dpi::PhysicalSize { width: 15, height: 15 });
                backend::execute_compute_unit(&device, &queue, &compute_pipeline, &compute_bind_group, window.inner_size());

                window.request_redraw();
            },
            Event::RedrawRequested( 
                window_id 
            ) if window_id == window.id() => { 
                match render_texture(&surface, &device, &queue, &screen_pipeline, &texture_bind_group, &vertex_buffer, &index_buffer) {
                    Ok(_) => {},
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => resize_surface(&surface, &device, window.inner_size()),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => control_flow.set_exit(),
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => (),
        }
    });
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

pub mod backend {
    use wgpu::util::DeviceExt; 

    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
    const QUAD: &[[f32; 5]] = &[
        Vertex::new([ 1.,  1., 0., 1., 1.]),
        Vertex::new([-1., -1., 0., 0., 0.]),
        Vertex::new([ 1., -1., 0., 1., 0.]),
        Vertex::new([-1.,  1., 0., 0., 1.])
    ];
    const QUAD_INDICES: &[u16; 6] = &[
        0, 1, 2,
        0, 3, 1
    ];

    pub fn get_texure_render_data<'a>(device: &wgpu::Device, view: &'a wgpu::TextureView) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        let address_mode = wgpu::AddressMode::ClampToEdge;
        let filter_opt = wgpu::FilterMode::Nearest;
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { 
            label: Some("Render texutre sampler"), 
            address_mode_u: address_mode, 
            address_mode_v: address_mode, 
            address_mode_w: address_mode, 
            mag_filter: wgpu::FilterMode::Linear, 
            min_filter: filter_opt, 
            mipmap_filter: filter_opt, 
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: Some("Texture (Fragment) bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                }
            ]
        });
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("Texture (Fragment) bind group"), 
            layout: &bind_group_layout, 
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler)
                }
            ]
        });

        (bind_group, bind_group_layout)
    }
    
    pub fn init_texture_render_pipeline(device: &wgpu::Device, screen_shader: &wgpu::ShaderModule, bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Texutre render pipeline layout"), 
            bind_group_layouts: &[
                bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { 
                module: screen_shader,
                entry_point: "vert_main",
                buffers: &[get_vertex_desc()],
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
                    module: screen_shader, 
                    entry_point: "frag_main", 
                    targets: &[Some(
                        wgpu::ColorTargetState { 
                            format: FORMAT, 
                            blend: Some(wgpu::BlendState::REPLACE), 
                            write_mask: wgpu::ColorWrites::ALL, 
                        }
                    )]
                }
            ),
            multiview: None,
        });

        pipeline
    }

    pub fn initialize_quad(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Quad vertex buffer"),
                contents: bytemuck::cast_slice(QUAD),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Quad index buffer"),
                contents: bytemuck::cast_slice(QUAD_INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        (vertex_buffer, index_buffer)
    }

    pub fn get_vertex_desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: (4 * 5) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[ // wgpu::vertex_attr_array![0 => Float32x3];
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: (4 * 3) as wgpu::BufferAddress,
                    shader_location: 1,
                }
            ],
        }
    }

    fn create_particle_list(count: u32) -> Vec<[f32; 4]> {
        let mut particles = vec![];

        for y in 0..count {
            for x in 0..count {
                particles.push([
                   x as f32, 
                   y as f32,
                   0f32,
                   0f32,
                ]);
            }
        }

        particles
    }

    pub fn initialize_compute_data(device: &wgpu::Device, size: winit::dpi::PhysicalSize<u32>) -> (wgpu::Texture, wgpu::BindGroup, wgpu::BindGroupLayout, wgpu::Buffer) {
        // data initialization
        let out_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Compute unit output texture"),
            size: wgpu::Extent3d { 
                width: size.width, 
                height: size.height, 
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: 
                wgpu::TextureUsages::STORAGE_BINDING |
                wgpu::TextureUsages::TEXTURE_BINDING
        });
        let out_view = out_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let particles_list = create_particle_list(15);
        let particles = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle buffer"),
            contents: bytemuck::cast_slice(particles_list.as_slice()),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // data layout specs
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: Some("Compute unit bind group layout"), 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture { 
                        access: wgpu::StorageTextureAccess::WriteOnly, 
                        format: FORMAT, 
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }, 
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: true }, 
                        has_dynamic_offset: false, 
                        min_binding_size: None
                    },
                    count: None,
                }
            ]
        }); // or this can be automatically extracted from the shader (by pipeline)

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("Compute unit bind group"),
            layout: &bind_group_layout, 
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&out_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particles.as_entire_binding(),
                }
            ] 
        });

        (out_texture, bind_group, bind_group_layout, particles)
    }

    pub fn init_fluid_data<'a>(device: &wgpu::Device, particles: &wgpu::Buffer) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: Some("Fluid bind group layout"), 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: None
                    },
                    count: None,
                }
            ]
        }); // or this can be automatically extracted from the shader (by pipeline)

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("Compute unit bind group"),
            layout: &bind_group_layout, 
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particles.as_entire_binding(),
                }
            ]
        });

        (bind_group, bind_group_layout)
    }

    pub fn init_compute_unit(device: &wgpu::Device, shader: &wgpu::ShaderModule, bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::ComputePipeline {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
            label: Some("Compute unit pipeline layout"), 
            bind_group_layouts: &[
                bind_group_layout
            ], 
            push_constant_ranges: &[] 
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute unit pipeline"),
            layout: Some(&layout), // don't have to specify layout, because I am using only one bind group
            module: shader,
            entry_point: "main",
        });

        pipeline
    }

    pub fn execute_compute_unit(device: &wgpu::Device, queue: &wgpu::Queue, pipeline: &wgpu::ComputePipeline, bind_group: &wgpu::BindGroup, size: winit::dpi::PhysicalSize<u32>) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute unit encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Compute unit pass") });

            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, bind_group, &[]);
            compute_pass.dispatch_workgroups(size.width, size.height, 1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
    
    /// Begin render pass with clear color instructions 
    fn begin_clear_render_pass<'a>(encoder: &'a mut wgpu::CommandEncoder, view: &'a wgpu::TextureView) -> wgpu::RenderPass<'a> {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear color render pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            wgpu::Color {
                                r: 0.1,
                                g: 0.1, 
                                b: 0.1,
                                a: 1.,
                            },
                        ),
                        store: true,
                    }

                })
            ],
            depth_stencil_attachment: None,
        });

        render_pass
    }

    pub fn render(
        surface: &wgpu::Surface, 
        device: &wgpu::Device, 
        queue: &wgpu::Queue, 
        pipeline: &wgpu::RenderPipeline, 
        vertex_buffer: &wgpu::Buffer,
        index_buffer: &wgpu::Buffer
    ) -> Result<(), wgpu::SurfaceError>{
        let output = surface.get_current_texture()?; 
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Clear color encoder"),
        });

        {
            let mut render_pass = begin_clear_render_pass(&mut encoder, &view);

            render_pass.set_pipeline(&pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, 0..2);
        }
        
        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn render_texture(surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue, pipeline: &wgpu::RenderPipeline, bind_group: &wgpu::BindGroup, vertex_buffer: &wgpu::Buffer, index_buffer: &wgpu::Buffer) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?; 
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Texture render encoder"),
        });

        {
            let mut render_pass = begin_clear_render_pass(&mut encoder, &view);

            render_pass.set_pipeline(&pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, 0..2);
        }
        
        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn clear_color(surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?; 
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Clear color encoder"),
        });

        begin_clear_render_pass(&mut encoder, &view);

        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

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
        path: &str, 
        entry: &str, 
        visibility: Visibility, 
        entries: Entries,
    ) -> Self {
        let source = fs::read_to_string(path).unwrap().as_str();
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

    pub fn create_texture(&mut self, size: Size<u32>, usage: wgpu::TextureUsages, access: binding::Access, is_storage: bool) -> &binding::Texture {
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

        &texture
    }

    pub fn create_buffer(&mut self, usage: wgpu::BufferUsages, size: u64, access: binding::Access) -> &binding::Buffer {
        let buffer_data = self.state.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size as wgpu::BufferAddress,
            usage,
            mapped_at_creation: false,
        });

        let buffer = binding::Buffer::new(buffer_data, access);
        self.add_entry(Box::new(buffer));

        &buffer
    }

    pub fn create_buffer_init<T>(&mut self, usage: wgpu::BufferUsages, contents: &[T], access: binding::Access) -> &binding::Buffer 
        where T: NoUninit
    {
        let buffer_data = self.state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(contents),
            usage
        });

        let buffer = binding::Buffer::new(buffer_data, access);
        self.add_entry(Box::new(buffer));

        &buffer
    }

    pub fn create_sampler(&mut self) -> &binding::Sampler {
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

        &sampler
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
                    resource: entry.get_resource(binding),
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

    pub fn get_binding(&self) 
        -> (Option<&wgpu::BindGroup>, Option<&wgpu::BindGroupLayout>) {
        if let None = self.bind_group {
            self.refresh_binding();
        }

        (self.bind_group.as_ref(), self.layout.as_ref())
    }

    pub fn get_bind_group_(&self) -> Option<&wgpu::BindGroup> {
        if let None = self.bind_group {
            self.refresh_binding();
        }

        self.bind_group.as_ref()
    }
    
    pub fn get_layout(&self) -> Option<&wgpu::BindGroupLayout> {
        if let None = self.layout { 
            self.refresh_binding();
        }
        
        self.layout.as_ref()
    }
}


/// Specify 2D size (width, height)
pub struct Size<T> 
where T: num_traits::Unsigned
{
    width: T,
    height: T
}

impl<T> Size<T>
where T: num_traits::Unsigned, u32: From<T> 
{
    pub fn new(width: T, height: T) -> Self {
        Size { width, height }
    }

    pub fn from_physical(size: winit::dpi::PhysicalSize<T>) -> Self {
        Size { width: size.width, height: size.height }
    }

    pub fn into_tuple(&self) -> (T, T) {
        (self.width, self.height)
    }

    pub fn into_extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d { 
            width: self.width.into(), 
            height: self.height.into(), 
            depth_or_array_layers: 1
        }
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
