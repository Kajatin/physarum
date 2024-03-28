use wgpu::{
    Backends, Color, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits,
    PowerPreference, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

struct State<'window> {
    size: PhysicalSize<u32>,
    surface: Surface<'window>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    clear_color: Color,
    window: &'window Window,
}

impl<'window> State<'window> {
    // Creating some of the wgpu types requires async code
    async fn new(window: &'window Window) -> State<'window> {
        let size = window.inner_size();

        // Context for all other wgpu objects.
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        // Handle to the physical graphics and/or compute device.
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let caps = surface.get_capabilities(&adapter);

        let surface_format = caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let clear_color = wgpu::Color::BLACK;

        Self {
            size,
            surface,
            device,
            queue,
            config,
            clear_color,
            window,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.x as f64 / self.size.width as f64,
                    g: position.y as f64 / self.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                };
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();

    let window_builder = WindowBuilder::new()
        .with_title("Physarum")
        .with_inner_size(PhysicalSize::new(1400, 1400))
        .with_resizable(false);

    let window = window_builder.build(&event_loop).unwrap();

    let mut state = State::new(&window).await;

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent { window_id, event } => {
                    // println!("Window event: {:?}, {:?}", window_id, event);

                    if window_id == state.window().id() {
                        if !state.input(&event) {
                            match event {
                                WindowEvent::CloseRequested => {
                                    elwt.exit();
                                }
                                WindowEvent::RedrawRequested => {
                                    state.update();

                                    match state.render() {
                                        Ok(_) => {}
                                        Err(wgpu::SurfaceError::Lost) => {
                                            state.surface.configure(&state.device, &state.config);
                                        }
                                        Err(wgpu::SurfaceError::OutOfMemory) => {
                                            eprintln!("Out of memory");
                                            elwt.exit();
                                        }
                                        Err(e) => eprintln!("{:?}", e),
                                    }
                                }
                                WindowEvent::KeyboardInput { event, .. } => {
                                    if event.state == winit::event::ElementState::Pressed {
                                        match event.physical_key {
                                            PhysicalKey::Code(code) => match code {
                                                KeyCode::Escape => elwt.exit(),
                                                _ => (),
                                            },
                                            _ => (),
                                        }
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
                Event::AboutToWait => {
                    state.window().request_redraw();
                }
                Event::LoopExiting => {
                    println!("The event loop is exiting; stopping");
                }
                _ => (),
            }
        })
        .unwrap_or_else(|e| {
            eprintln!("An error occurred: {}", e);
        });
}
