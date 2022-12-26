use backend::{resize_surface, render_texture};
use wgpu::include_wgsl;
use winit::event::{WindowEvent, Event};

pub async fn run() {
    let (event_loop, window) = window::init_window();
    let (surface, device, queue) = backend::init_backend(&window).await;

    let (vertex_buffer, index_buffer) = backend::initialize_quad(&device);

    let (out_texture, compute_bind_group, compute_bind_group_layout) = backend::initialize_compute_data(&device, window.inner_size());
    let (texture_bind_group, texture_bind_group_layout) = backend::get_texure_render_data(&device, &out_texture.create_view(&wgpu::TextureViewDescriptor::default()));
    let compute_shader = device.create_shader_module(include_wgsl!("../res/shaders/render_shader.wgsl"));
    let compute_pipeline = backend::init_compute_unit(&device, &compute_shader, &compute_bind_group_layout);

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

    const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
    const QUAD: &[[f32; 5]] = &[
        [ 1.,  1., 0., 1., 1.],
        [-1., -1., 0., 0., 0.],
        [ 1., -1., 0., 1., 0.],
        [-1.,  1., 0., 0., 1.]
    ];
    const QUAD_INDICES: &[u16; 6] = &[
        0, 1, 2,
        0, 3, 1
    ];

    pub async fn init_backend(window: &winit::window::Window) -> (wgpu::Surface, wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor { 
                    label: Some("main device"),
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None
             )
            .await
            .unwrap();

        init_surface(&surface, &device, window.inner_size());
        (surface, device, queue)
    }
    
    pub fn init_surface(surface: &wgpu::Surface, device: &wgpu::Device, size: winit::dpi::PhysicalSize<u32>) {
        surface.configure(device, &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: FORMAT, // could request supported from adapter
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        })
    }

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

    pub fn resize_surface(surface: &wgpu::Surface, device: &wgpu::Device, size: winit::dpi::PhysicalSize<u32>) {
        init_surface(surface, device, size)
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

    fn create_particle_list(count: u32) -> Vec<[f32; 2]> {
        let mut particles = vec![];

        for y in 0..count {
            for x in 0..count {
                particles.push([
                   x as f32, 
                   y as f32
                ]);
            }
        }

        particles
    }

    pub fn initialize_compute_data(device: &wgpu::Device, size: winit::dpi::PhysicalSize<u32>) -> (wgpu::Texture, wgpu::BindGroup, wgpu::BindGroupLayout) {
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

        let particles_list = create_particle_list(4);
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

        (out_texture, bind_group, bind_group_layout)
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
