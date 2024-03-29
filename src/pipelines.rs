use crate::resources::Resources;

pub struct Pipelines {
    pub agent_sense_move_deposit: wgpu::ComputePipeline,
    pub diffuse_and_decay: wgpu::ComputePipeline,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: &Resources,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline-layout"),
            bind_group_layouts: &[
                &resources.shader_context.bind_group_layout,
                &resources.data_layer.bind_group_layout,
                &resources.trail_layer.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let agent_sense_move_deposit =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("data-layer-compute-pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "agent_sense_move_deposit",
            });

        let diffuse_and_decay = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("trail-layer-compute-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "diffuse_and_decay",
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fragment_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            agent_sense_move_deposit,
            diffuse_and_decay,
            render_pipeline,
        }
    }
}
