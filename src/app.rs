use wgpu::{ ColorTargetState, CommandEncoderDescriptor, RenderPipeline,  TextureViewDescriptor};
use crate::common::{Time, Vertex, Triangles, Shape};
use crate::render_env::RenderEnv;

pub struct App {
    pipeline: RenderPipeline,
    triangles: Triangles,
    time: Time,
}

impl App {
    pub fn new(render_env: &RenderEnv) -> anyhow::Result<Self> {
        let device = &render_env.device;
        let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/raytrace.wgsl"));
        let time = Time::new(render_env);

        let triangles = Triangles::new(render_env, &[Shape::unit_square()]);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("My pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("My pipeline"),
            layout: Some(&pipeline_layout),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: render_env.surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            multiview: None,
        });

        Ok(Self {
            pipeline,
            time,
            triangles,
        })
    }

    pub fn update(&mut self, render_env: &RenderEnv) {
        self.time.add_delta(render_env, 0.01);
    }

    pub fn render(&self, render_env: &RenderEnv) -> anyhow::Result<()> {
        let device = &render_env.device;
        let current_texture = render_env.surface.get_current_texture()?;
        let view = current_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("My render pass"),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        self.triangles.draw(&mut render_pass);
        drop(render_pass);

        render_env.queue.submit(Some(encoder.finish()));
        current_texture.present();
        Ok(())
    }
}
