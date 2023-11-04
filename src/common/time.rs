use wgpu::{*, util::{BufferInitDescriptor, DeviceExt}};
use wgpu::PipelineLayout;
use crate::render_env::RenderEnv;
use crate::common::UNIFORM_BUFFER_BINDING;

pub struct Time {
    time: f32,
    pub time_uniform: Buffer,
    pub delta_uniform: Buffer,
    pub bind_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl Time {
    pub fn new(render_env: &RenderEnv) -> Self {
        let device = &render_env.device;
        let time = 0;
        let time_uniform = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("delta uniform"),
            contents: &0.0f32.to_ne_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let delta_uniform = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("delta uniform"),
            contents: &0.0f32.to_ne_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Time bind layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
                    ty: UNIFORM_BUFFER_BINDING,
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::all(),
                    ty: UNIFORM_BUFFER_BINDING,
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Time bind group"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: time_uniform.as_entire_binding(),
            }, wgpu::BindGroupEntry {
                binding: 1,
                resource: delta_uniform.as_entire_binding(),
            }],
        });

        Self {
            time: 0.0,
            time_uniform,
            delta_uniform,
            bind_layout,
            bind_group,
        }
    }

    /// Adds the "delta" time value to the current time. This way you
    /// can use a fixed delta for each frame or the use time since the
    /// last frame
    pub fn add_delta(&mut self, render_env: &RenderEnv, delta: f32) {
        let queue = &render_env.queue;
        self.time += delta;
        queue.write_buffer(&self.time_uniform, 0, &self.time.to_ne_bytes());
        queue.write_buffer(&self.delta_uniform, 0, &delta.to_ne_bytes());
    }
}
