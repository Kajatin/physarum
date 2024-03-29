use std::sync::Arc;

use wgpu::{Backends, Instance, InstanceDescriptor, Surface, SurfaceConfiguration};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

mod agent;
mod device;
mod parameters;
mod pipelines;
mod resources;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
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
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

struct State<'window> {
    params: parameters::Parameters,
    surface: Surface<'window>,
    device: device::Device,
    config: SurfaceConfiguration,
    resources: resources::Resources,
    pipelines: pipelines::Pipelines,
    window: Arc<Window>,
    gpu_mutex: Arc<std::sync::Mutex<()>>,
    exiting: bool,
}

impl<'window> State<'window> {
    async fn new(window: Window) -> State<'window> {
        let size = window.inner_size();

        let window = Arc::new(window);

        let mut params = parameters::Parameters::builder()
            .shader_parameters(
                parameters::ShaderParameters::builder()
                    .canvas_width(size.width)
                    .canvas_height(size.height)
                    .build(),
            )
            .build();

        // params.shader_parameters.randomize();

        // Context for all other wgpu objects.
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let device = device::Device::new(&instance, Some(&surface)).await;

        let config = configure_surface(&device, &surface, size);

        let resources = resources::Resources::new(&device.device, &params);

        let pipelines = pipelines::Pipelines::new(&device.device, config.format, &resources);

        Self {
            params,
            surface,
            device,
            config,
            resources,
            pipelines,
            window,
            gpu_mutex: Arc::new(std::sync::Mutex::new(())),
            exiting: false,
        }
    }

    fn update(&self) {
        let gpu_lock = self.gpu_mutex.lock().unwrap();

        // Start a new command encoder
        let mut command_encoder =
            self.device
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("command-encoder"),
                });

        // Diffuse and decay
        {
            let mut compute_pass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("diffuse-and-decay-cp"),
                    timestamp_writes: None,
                });

            compute_pass.set_pipeline(&self.pipelines.diffuse_and_decay);
            compute_pass.set_bind_group(0, &self.resources.shader_context.bind_group, &[]);
            compute_pass.set_bind_group(1, &self.resources.data_layer.bind_group, &[]);
            compute_pass.set_bind_group(2, &self.resources.trail_layer.bind_group, &[]);

            compute_pass.dispatch_workgroups(
                self.params.shader_parameters.canvas_width / 8,
                self.params.shader_parameters.canvas_height / 8,
                1,
            );
        }

        // Move agents and deposit
        {
            let mut compute_pass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("agent-sense-move-deposit-cp"),
                    timestamp_writes: None,
                });

            compute_pass.set_pipeline(&self.pipelines.agent_sense_move_deposit);
            compute_pass.set_bind_group(0, &self.resources.shader_context.bind_group, &[]);
            compute_pass.set_bind_group(1, &self.resources.data_layer.bind_group, &[]);
            compute_pass.set_bind_group(2, &self.resources.trail_layer.bind_group, &[]);

            // Lay agents out in x and y so they can be mapped to shader workgroups
            let number_of_active_agents = self.params.shader_parameters.number_of_active_agents;

            // Must match what is in the shader code
            const WORKGROUP_SIZE_X: u32 = 8;
            const WORKGROUP_SIZE_Y: u32 = 8;
            const WORKGROUP_SIZE_Z: u32 = 1;

            let threads_per_workgroup = WORKGROUP_SIZE_X * WORKGROUP_SIZE_Y * WORKGROUP_SIZE_Z;

            // Hacked integer division: number_of_active_agents / threads_per_workgroup but hacked so it rounds up
            let workgroups_needed =
                (number_of_active_agents + (threads_per_workgroup - 1)) / threads_per_workgroup;

            const NUMBER_OF_WORKGROUPS_X: u32 = 32;
            // Hacked integer division: workgroups_needed / 32 but hacked so it rounds up
            let number_of_workgroups_y = (workgroups_needed + 31) / 32;
            let number_of_workgroups_z = 1;

            compute_pass.dispatch_workgroups(
                NUMBER_OF_WORKGROUPS_X,
                number_of_workgroups_y,
                number_of_workgroups_z,
            );
        }

        let command_buffer = command_encoder.finish();
        self.device.queue.submit(Some(command_buffer));

        drop(gpu_lock);
    }

    fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let gpu_lock = self.gpu_mutex.lock().unwrap();

        let mut command_encoder =
            self.device
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render-command-encoder"),
                });

        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipelines.render_pipeline);
            render_pass.set_bind_group(0, &self.resources.shader_context.bind_group, &[]);
            render_pass.set_bind_group(1, &self.resources.data_layer.bind_group, &[]);
            render_pass.set_bind_group(2, &self.resources.trail_layer.bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.device
            .queue
            .submit(std::iter::once(command_encoder.finish()));

        drop(gpu_lock);

        surface_texture.present();

        Ok(())
    }

    fn exit(&mut self) {
        self.exiting = true;
    }
}

fn configure_surface(
    device: &device::Device,
    surface: &Surface,
    size: PhysicalSize<u32>,
) -> SurfaceConfiguration {
    let caps = surface.get_capabilities(&device.adapter);

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

    surface.configure(&device.device, &config);

    config
}

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();

    let window_builder = WindowBuilder::new()
        .with_title("Physarum")
        .with_inner_size(PhysicalSize::new(1400, 1400))
        .with_resizable(false);

    let window = window_builder.build(&event_loop).unwrap();

    let state = Arc::new(State::new(window).await);

    // Spawn thread to drive the simulation forward by dispatching GPU commands at e.g. 60 FPS
    let state_tick = Arc::clone(&state);
    let ticker = tokio::spawn(async move {
        loop {
            if state_tick.exiting {
                break;
            }

            state_tick.update();

            tokio::time::sleep(std::time::Duration::from_nanos(
                1_000_000_000 / state_tick.params.target_ticks_per_second as u64,
            ))
            .await;
        }
    });

    event_loop
        .run(move |event, elwt| {
            match event {
                Event::NewEvents(cause) => match cause {
                    winit::event::StartCause::ResumeTimeReached { .. } => {
                        state.window.request_redraw();
                    }
                    _ => {}
                },
                Event::WindowEvent { window_id, event } => {
                    // println!("Window event: {:?}, {:?}", window_id, event);

                    if window_id == state.window.id() {
                        match event {
                            WindowEvent::CloseRequested => {
                                elwt.exit();
                            }
                            WindowEvent::RedrawRequested => {
                                let time_per_frame = std::time::Duration::from_micros(
                                    1_000_000 / state.params.target_ticks_per_second as u64,
                                );
                                let next_frame = std::time::Instant::now() + time_per_frame;
                                elwt.set_control_flow(ControlFlow::WaitUntil(next_frame));

                                match state.render() {
                                    Ok(_) => {}
                                    Err(wgpu::SurfaceError::Lost) => {
                                        state
                                            .surface
                                            .configure(&state.device.device, &state.config);
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
                                            KeyCode::KeyR => {
                                                // state.params.shader_parameters.randomize();
                                            }
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
                Event::LoopExiting => {
                    // state.exit();
                    println!("The event loop is exiting; stopping");
                }
                _ => (),
            }
        })
        .unwrap_or_else(|e| {
            eprintln!("An error occurred: {}", e);
        });

    ticker.await.unwrap();
}
