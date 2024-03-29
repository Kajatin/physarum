use wgpu::util::DeviceExt;

use crate::{agent::{initial_agent_distribution, Agent}, parameters::{Parameters, ShaderParameters}};

pub struct Resource {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub struct Resources {
    pub shader_context: Resource,
    pub data_layer: Resource,
    pub trail_layer: Resource,
}

impl Resources {
    pub fn new(device: &wgpu::Device, params: &Parameters) -> Self {
        let shader_context = create_shader_context(device, params);
        let data_layer = create_data_layer(device, params);
        let trail_layer = create_trail_layer(device, params);

        Self {
            shader_context,
            data_layer,
            trail_layer,
        }
    }
}

fn create_shader_context(device: &wgpu::Device, params: &Parameters) -> Resource {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("shader-context"),
        contents: bytemuck::cast_slice(&[params.shader_parameters]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("shader-context-bind-group-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(
                    std::mem::size_of::<ShaderParameters>() as u64
                ),
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("shader-context-bind-group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });

    Resource {
        buffer,
        bind_group,
        bind_group_layout,
    }
}

fn create_data_layer(device: &wgpu::Device, params: &Parameters) -> Resource {
    let agents = initial_agent_distribution(params);

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("data-layer"),
        contents: bytemuck::cast_slice(&agents),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("data-layer-bind-group-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(
                    u64::from(params.number_of_agents) * std::mem::size_of::<Agent>() as u64,
                ),
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("data-layer-bind-group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });

    Resource {
        buffer,
        bind_group,
        bind_group_layout,
    }
}

fn create_trail_layer(device: &wgpu::Device, params: &Parameters) -> Resource {
    let canvas_resolution =
        params.shader_parameters.canvas_width * params.shader_parameters.canvas_height;

    // Start with a black canvas
    let init: Vec<f32> = vec![0.0; canvas_resolution as usize];

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("trail-layer"),
        contents: bytemuck::cast_slice(&init),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("trail-layer-bind-group-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(
                    (usize::try_from(canvas_resolution).unwrap() * std::mem::size_of::<f32>())
                        as u64,
                ),
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("trail-layer-bind-group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });

    Resource {
        buffer,
        bind_group,
        bind_group_layout,
    }
}
