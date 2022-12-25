use backend::{clear_color, resize_surface};
use winit::event::{WindowEvent, Event};

pub async fn run() {
    let (event_loop, window) = window::init_window();
    let (surface, device, queue) = backend::init_backend(&window).await;

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
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested( 
                window_id 
            ) if window_id == window.id() => { 
                match clear_color(&surface, &device, &queue) {
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
    const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

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

    pub fn init_render_pipeline(device: &wgpu::Device, screen_shader: &wgpu::ShaderModule) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"), 
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { 
                module: screen_shader,
                entry_point: "vert_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
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
                                r: 0.,
                                g: 0., 
                                b: 0.,
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

    pub fn render(surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue, pipeline: &wgpu::RenderPipeline) -> Result<(), wgpu::SurfaceError>{
        let output = surface.get_current_texture()?; 
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Clear color encoder"),
        });

        {
            let mut render_pass = begin_clear_render_pass(&mut encoder, &view);

            render_pass.set_pipeline(&pipeline);
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
