use wgpu::*;

use crate::{
    common::{Shape, Triangles, Vertex},
    render_env::RenderEnv,
};

pub struct Screen {
    pipeline: RenderPipeline,
    screen_quad: Triangles,
}

impl Screen {
    pub fn new(render_env: &RenderEnv, sampler_bind_layout: &BindGroupLayout) -> Self {
        let device = &render_env.device;
        let shader = device.create_shader_module(include_wgsl!("./shaders/screen_shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Screen Pipeline Layout"),
            bind_group_layouts: &[sampler_bind_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Screen pipeline"),
            layout: Some(&pipeline_layout),
            depth_stencil: None,
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: render_env.surface_config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multisample: MultisampleState::default(),
            multiview: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::VERTEX_LAYOUT],
            },
            primitive: wgpu::PrimitiveState {
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
        });

        let screen_quad = Triangles::new(render_env, &[Shape::unit_square()]);

        Self {
            pipeline,
            screen_quad,
        }
    }

    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        current_texture: &SurfaceTexture,
        sampler_bind_group: &BindGroup,
    ) {
        let view = current_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("My render pass"),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.0,
                            b: 0.25,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            ..Default::default()
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, sampler_bind_group, &[]);
        self.screen_quad.draw(&mut render_pass);
    }
}
