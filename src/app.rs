use crate::common::{Shape, Time, Triangles, Vertex};
use crate::raytracing::Raytracer;
use crate::render_env::RenderEnv;
use crate::scene::Scene;
use crate::screen::Screen;
use wgpu::{ColorTargetState, CommandEncoderDescriptor, RenderPipeline, TextureViewDescriptor};
use winit::event::{KeyEvent, ElementState};
use winit::keyboard::{PhysicalKey, KeyCode};

pub struct App {
    time: Time,
    ray_tracer: Raytracer,
    screen: Screen,
    scene: Scene,
}

impl App {
    pub fn new(render_env: &RenderEnv) -> anyhow::Result<Self> {
        let time = Time::new(render_env);
        let scene = Scene::new(render_env);
        let ray_tracer = Raytracer::new(render_env, &scene.bind_group_layout, &time.bind_layout);
        let screen = Screen::new(
            render_env,
            &ray_tracer.sampler_bind_layout,
        );

        Ok(Self {
            time,
            ray_tracer,
            screen,
            scene,
        })
    }

    pub fn on_input(&mut self, render_env: &RenderEnv, input: winit::event::KeyEvent) {
        match input {
            KeyEvent { physical_key: PhysicalKey::Code(key), state: ElementState::Pressed, .. } => {
                if key == KeyCode::KeyR {
                    self.scene.reload(render_env);
                }
            }
            _ => (),
        }
    }

    pub fn update(&mut self, render_env: &RenderEnv) {
        self.time.add_delta(render_env, 0.01);
    }

    pub fn render(&mut self, render_env: &RenderEnv) -> anyhow::Result<()> {
        let device = &render_env.device;

        let current_texture = render_env.surface.get_current_texture()?;
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());

        self.ray_tracer.compute(&mut encoder, &self.scene.bind_group, &self.time.bind_group);
        self.screen.render(&mut encoder, &current_texture, &self.ray_tracer.sampler_bind_group);

        render_env.queue.submit(Some(encoder.finish()));

        // this needs to be after the submit
        self.scene.increase_frame(render_env);
        current_texture.present();
        Ok(())
    }
}
