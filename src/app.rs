use std::collections::HashSet;

use crate::common::{Shape, Time, Triangles, Vertex};
use crate::raytracing::Raytracer;
use crate::render_env::RenderEnv;
use crate::world::World;
use crate::screen::Screen;
use glam::Vec2;
use wgpu::{ColorTargetState, CommandEncoderDescriptor, RenderPipeline, TextureViewDescriptor};
use winit::dpi::PhysicalPosition;
use winit::event::{KeyEvent, ElementState, MouseButton, WindowEvent, Modifiers};
use winit::keyboard::{PhysicalKey, KeyCode};

pub struct App {
    time: Time,
    ray_tracer: Raytracer,
    screen: Screen,
    world: World,
    modifiers: Modifiers,
    cursor_pos: Vec2,
    keys_held: HashSet<KeyCode>
}

impl App {
    pub fn new(render_env: &RenderEnv) -> anyhow::Result<Self> {
        let time = Time::new(render_env);
        let world = World::new(render_env);
        let ray_tracer = Raytracer::new(render_env, &world.bind_group_layout, &time.bind_layout);
        let screen = Screen::new(
            render_env,
            &ray_tracer.sampler_bind_layout,
        );
        let modifiers = Modifiers::default();

        Ok(Self {
            time,
            ray_tracer,
            screen,
            world,
            modifiers,
            cursor_pos: Vec2::ZERO,
            keys_held: HashSet::new(),
        })
    }

    pub fn on_key_input(&mut self, render_env: &RenderEnv, input: winit::event::KeyEvent) {
        match input {
            KeyEvent { physical_key: PhysicalKey::Code(key), state, .. } => {
                if !state.is_pressed() {
                    self.keys_held.remove(&key);
                    return;
                }

                self.keys_held.insert(key);
                self.world.on_key_press(render_env, key, &self.keys_held);
                if key == KeyCode::KeyR {
                    self.world.reload(render_env);
                }
            }
            _ => (),
        }
    }

    pub fn on_event(&mut self, render_env: &RenderEnv, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event: key_input, .. } => self.on_key_input(render_env, key_input),
            WindowEvent::MouseWheel { delta, .. } => {
                self.world.on_scroll(render_env, delta, &self.modifiers); }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = Vec2 {
                    x: position.x as f32,
                    y: render_env.window.inner_size().height as f32 - position.y as f32,
                };
                self.world.on_mouse_input(render_env, self.cursor_pos, None, None);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.world.on_mouse_input(render_env, self.cursor_pos, Some(state), Some(button));
            }
            _ => (),
        }
    }


    pub fn update(&mut self, render_env: &RenderEnv) {
        self.time.add_delta(render_env, 0.01);
        // TODO: Fix this awful solution lol
        self.world.on_key_press(render_env, KeyCode::F35, &self.keys_held);
    }

    pub fn render(&mut self, render_env: &RenderEnv) -> anyhow::Result<()> {
        let device = &render_env.device;
        let queue = &render_env.queue;

        let current_texture = render_env.surface.get_current_texture()?;
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());

        self.ray_tracer.compute(&mut encoder, &self.world.bind_group, &self.time.bind_group);
        self.screen.render(&mut encoder, &current_texture, &self.ray_tracer.sampler_bind_group);

        queue.submit(Some(encoder.finish()));

        // this needs to be after the submit
        self.world.increase_frame(render_env);
        current_texture.present();
        Ok(())
    }

    fn performance(&mut self) {

    }
}
